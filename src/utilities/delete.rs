use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::{Change, ChangeSet, Operation}
};

//TODO: can this function and backspace be combined?...

/// Deletes text inside each [`Selection`] in [`Selections`], or if [`Selection`] not extended, the next character, and pushes changes to undo stack.
pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();
    let mut cannot_delete = false;
    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        //handles cursor at doc end
        if selection.anchor() == app.buffer.len_chars() && selection.cursor(&app.buffer, semantics.clone()) == app.buffer.len_chars(){
            cannot_delete = true; //don't modify text buffer here...
            let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
            changes.push(change);
        }
        else{   //apply the delete
            let change = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
            if let Operation::Insert{inserted_text} = change.inverse(){
                app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
            }
            changes.push(change);
        }
    }

    if app.selections.count() == 1 && cannot_delete{return Err(ApplicationError::SelectionAtDocBounds);}
    else{
        // push change set to undo stack
        app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

        // clear redo stack. new actions invalidate the redo history
        app.redo_stack.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests{
    use crate::utilities::delete;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::View,
    };

    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, expected_text: &str, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));

        let expected_buffer = crate::buffer::Buffer::new(expected_text, None, false);
        let mut vec_expected_selections = Vec::new();
        for tuple in tuple_expected_selections{
            vec_expected_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &expected_buffer, semantics.clone()));
        }
        let expected_selections = Selections::new(vec_expected_selections, expected_primary, &expected_buffer, semantics.clone());
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        let result = delete::application_impl(&mut app, semantics);
        assert!(!result.is_err());
        
        assert_eq!(expected_buffer, app.buffer);
        assert_eq!(expected_selections, app.selections);
        //println!("expected: {:?}\ngot: {:?}", expected_buffer, app.buffer);
        //assert!(app.buffer.is_modified());    //is modified doesn't work with tests, because it now checks against a persistent file, which tests don't have
    }
    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        assert!(delete::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_non_extended_selections(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
            "dk\nome\nshit\n", 
            vec![
                (0, 1, Some(0)),
                (3, 4, Some(0))
            ], 0
        );
    }

    #[test] fn with_extended_selections(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 2, None),
                (4, 6, None)
            ], 0, 
            "k\nme\nshit\n", 
            vec![
                (0, 1, Some(0)),
                (2, 3, Some(0))
            ], 0
        );
    }
    //TODO: maybe test direction backward too?...

    #[test] fn with_valid_selection_and_cursor_at_doc_end(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 10, None),
                (14, 15, None)
            ], 0, 
            "idk\nsome\nhit\n", 
            vec![
                (9, 10, Some(0)),
                (13, 14, None)
            ], 0
        );
    }

    #[test] fn errors_if_single_cursor_at_doc_end(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 15, None)
            ], 0
        );
    }
}

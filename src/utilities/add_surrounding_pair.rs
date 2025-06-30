use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::{Change, ChangeSet, Operation}
};

//TODO: i think all edit actions + apply replace/insert/delete should prob be made purely functional...

//had to make the following public
    //Document.text
    //Document.selections
    //Document.undo_stack
    //Document.redo_stack
    //Document::apply_replace
//is this easing of encapsulation acceptable?...
pub fn application_impl(app: &mut Application, leading_char: char, trailing_char: char, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();
    let mut cannot_add_surrounding_pair = false;  //to handle cursor at doc end...
    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        //handles cursor at doc end
        if selection.anchor() == app.buffer.len_chars() && selection.cursor(&app.buffer, semantics.clone()) == app.buffer.len_chars(){
            cannot_add_surrounding_pair = true; //don't modify text buffer here...
            let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
            changes.push(change);
        }
        else{   //replace each selection with its text contents + leading and trailing char added
            //let mut contents = selection.contents_as_string(&document.text);
            let mut contents = selection.to_string(&app.buffer);
            contents.insert(0, leading_char);
            contents.push(trailing_char);
            let change = Application::apply_replace(&mut app.buffer, &contents, selection, CursorSemantics::Block);
            changes.push(change);
            app.selections.shift_subsequent_selections_forward(i, 2);  //TODO: could this be handled inside apply_replace and similar functions?...
        }
    }

    if app.selections.count() == 1 && cannot_add_surrounding_pair{return Err(ApplicationError::InvalidInput);}
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
    use crate::utilities::add_surrounding_pair;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::DisplayArea,
    };

    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, leading_char: char, trailing_char: char, expected_text: &str, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));

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
        
        let result = add_surrounding_pair::application_impl(&mut app, leading_char, trailing_char, semantics);
        assert!(!result.is_err());
        
        assert_eq!(expected_buffer, app.buffer);
        assert_eq!(expected_selections, app.selections);
        //println!("expected: {:?}\ngot: {:?}", expected_buffer, app.buffer);
        //assert!(app.buffer.is_modified());    //is modified doesn't work with tests, because it now checks against a persistent file, which tests don't have
    }
    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, leading_char: char, trailing_char: char){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        assert!(add_surrounding_pair::application_impl(&mut app, leading_char, trailing_char, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_single_selection(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 3, None)
            ], 0, 
            '{', '}', 
            "{idk}\nsome\nshit\n", 
            vec![
                (5, 6, Some(5))
            ], 0
        );
    }

    //TODO: test multiple selections

    //TODO: test with selection over newline(should be the same, but worth verifying...)

    #[test] fn with_valid_selection_and_cursor_at_end_of_doc(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 11, None),
                (14, 15, None)
            ], 0, 
            '<', '>', 
            "idk\nsome\n<sh>it\n", 
            vec![
                (13, 14, Some(4)),
                (16, 17, None)
            ], 0
        );
    }

    #[test] fn errors_when_single_cursor_at_end_of_document(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 15, None)
            ], 0, 
            '{', '}'
        );
    }

    //TODO?: should resultant selection after adding surrounding pair be a selection over the content and pair?...
    //i think this is a much deeper question than this single function...
    //this relates to all replacement text  (if we use the default Document::apply_replace...)
}

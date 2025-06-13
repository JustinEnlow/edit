use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::{Change, ChangeSet, Operation}
};

//TODO: combine backspace with delete (make delete take a direction::Forward/Backward)

/// Deletes the previous character, or deletes selection if extended.
/// #### Invariants:
/// - will not delete past start of doc
/// - at start of line, appends current line to end of previous line
/// - removes previous soft tab, if `TAB_WIDTH` spaces are before cursor
/// - deletes selection if selection extended
pub fn application_impl(app: &mut Application, use_hard_tab: bool, tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::with_capacity(app.selections.count());
    let mut cannot_delete = false;

    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        if selection.is_extended(){
            let change = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
            if let Operation::Insert{inserted_text} = change.inverse(){
                app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
            }
            changes.push(change);
        }else{
            if selection.anchor() == 0 && selection.cursor(&app.buffer, semantics.clone()) == 0{
                cannot_delete = true; //don't modify text buffer here...
                let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
                changes.push(change);
            }
            else{
                let offset_from_line_start = app.buffer.offset_from_line_start(selection.cursor(&app.buffer, semantics.clone()));
                //let line = app.buffer.inner.line(app.buffer.char_to_line(selection.cursor(&app.buffer, semantics.clone())));
                let is_deletable_soft_tab = !use_hard_tab && offset_from_line_start >= tab_width
                // handles case where user adds a space after a tab, and wants to delete only the space
                && offset_from_line_start % tab_width == 0
                // if previous 4 chars are spaces, delete 4. otherwise, use default behavior
                && app.buffer.slice_is_all_spaces(offset_from_line_start.saturating_sub(tab_width), offset_from_line_start);

                if is_deletable_soft_tab{
                    selection.shift_and_extend(tab_width, &app.buffer, semantics.clone());
                    changes.push(Application::apply_delete(&mut app.buffer, selection, semantics.clone()));
                    app.selections.shift_subsequent_selections_backward(i, tab_width);
                }
                else{
                    //if let Ok(new_selection) = selection.move_left(&document.text, semantics){
                    if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(selection, &app.buffer, semantics.clone()){
                        *selection = new_selection;
                    }   //TODO: handle error    //first for loop guarantees no selection is at doc bounds, so this should be ok to ignore...
                    changes.push(Application::apply_delete(&mut app.buffer, selection, semantics.clone()));
                    app.selections.shift_subsequent_selections_backward(i, 1);
                }
            }
        }
    }

    if app.selections.count() == 1 && cannot_delete{return Err(ApplicationError::SelectionAtDocBounds);}
    else{
        // push changes to undo stack
        app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

        // clear redo stack. new actions invalidate the redo history
        app.redo_stack.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests{
    use crate::utilities::backspace;
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
        
        let result = backspace::application_impl(&mut app, false, 4, semantics);
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
        
        assert!(backspace::application_impl(&mut app, false, 4, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn common_use(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (1, 2, None),
                (5, 6, None)
            ], 0, 
            "dk\nome\nshit\n", 
            vec![
                (0, 1, Some(0)),
                (3, 4, Some(0))
            ], 0
        );
    }

    #[test] fn when_at_line_start_appends_current_line_to_previous_line(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 5, None)
            ], 0, 
            "idksome\nshit\n", 
            vec![
                (3, 4, Some(3))
            ], 0
        );
    }

    #[test] fn with_valid_selection_and_cursor_at_doc_start(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
            "idksome\nshit\n", 
            vec![
                (0, 1, None),
                (3, 4, Some(3))
            ], 0
        );
    }

    #[test] fn errors_if_single_cursor_at_doc_start(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 0
        );
    }

    #[test] fn with_extended_selection_deletes_selection(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 4, None)
            ], 0, 
            "some\nshit\n", 
            vec![
                (0, 1, Some(0))
            ], 0
        );
    }

    //TODO: test tab deletion with soft tabs
    //TODO: test tab deletion with hard tabs
    //TODO: test with various tab widths


    //TODO: test error described in range.rs:15:9
}

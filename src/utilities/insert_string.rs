use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::{Change, ChangeSet},
};

/// Inserts provided string into text at each selection.
pub fn application_impl(app: &mut Application, string: &str, use_hard_tab: bool, tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    if app.buffer.read_only{return Err(ApplicationError::ReadOnlyBuffer);}
    if string.is_empty(){return Err(ApplicationError::InvalidInput);}
    
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();

    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        let change = match string{
            //"\n" => {}    //handle behavior specific to pressing "enter". auto-indent, etc... //TODO: create tests for newline behavior...
            "\t" => {   //handle behavior specific to pressing "tab".
                if use_hard_tab{
                    if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), "\t")}
                    else{handle_insert(app, "\t", i, semantics.clone())}
                }
                else{
                    let tab_distance = app.buffer.distance_to_next_multiple_of_tab_width(selection, semantics.clone(), tab_width);
                    let modified_tab_width = if tab_distance > 0 && tab_distance < tab_width{tab_distance}else{tab_width};
                    let soft_tab = " ".repeat(modified_tab_width);

                    if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), &soft_tab)}
                    else{handle_insert(app, &soft_tab, i, semantics.clone())}
                }
            }
            //handle any other inserted string
            _ => {
                if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), string)}
                else{handle_insert(app, string, i, semantics.clone())}
            }
        };

        changes.push(change);
    }

    // push change set to undo stack
    app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

    // clear redo stack. new actions invalidate the redo history
    app.redo_stack.clear();

    Ok(())
}
fn handle_insert_replace(app: &mut Application, current_selection_index: usize, semantics: CursorSemantics, new_text: &str) -> Change{
    use std::cmp::Ordering;
    let selection = app.selections.nth_mut(current_selection_index);
    let change = Application::apply_replace(&mut app.buffer, new_text, selection, semantics);
    if let crate::history::Operation::Replace{replacement_text} = change.inverse(){
        match replacement_text.len().cmp(&new_text.len()){    //old selected text vs new text
            Ordering::Greater => {app.selections.shift_subsequent_selections_backward(current_selection_index, replacement_text.len().saturating_sub(new_text.len()));}
            Ordering::Less => {app.selections.shift_subsequent_selections_forward(current_selection_index, new_text.len().saturating_sub(replacement_text.len()));}
            Ordering::Equal => {}   // no change to subsequent selections
        }
    }
    change
}
fn handle_insert(app: &mut Application, string: &str, current_selection_index: usize, semantics: CursorSemantics) -> Change{
    let selection = app.selections.nth_mut(current_selection_index);
    let change = Application::apply_insert(&mut app.buffer, string, selection, semantics);
    app.selections.shift_subsequent_selections_forward(current_selection_index, string.len());
    change
}

#[cfg(test)]
mod tests{
    use crate::utilities::insert_string;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::View,
    };

    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, string: &str, expected_text: &str, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
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
        
        let result = insert_string::application_impl(&mut app, string, false, 4, semantics);
        assert!(!result.is_err());
        
        assert_eq!(expected_buffer, app.buffer);
        assert_eq!(expected_selections, app.selections);
        //println!("expected: {:?}\ngot: {:?}", expected_buffer, app.buffer);
        //assert!(app.buffer.is_modified());    //is modified doesn't work with tests, because it now checks against a persistent file, which tests don't have
    }
    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, string: &str){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        assert!(insert_string::application_impl(&mut app, string, false, 4, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn insert_single_char_with_multi_selection_block_semantics(){
        test(
            CursorSemantics::Block, 
            "some\nshit\n", 
            vec![
                (0, 1, None),
                (5, 6, None)
            ], 0, 
            "x", 
            "xsome\nxshit\n", 
            vec![
                (1, 2, Some(1)),
                (7, 8, Some(1))
            ], 0
        );
    }

    #[test] fn insert_single_char_with_multi_selection_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "some\nshit\n", 
            vec![
                (0, 0, None),
                (5, 5, None)
            ], 0, 
            "x", 
            "xsome\nxshit\n", 
            vec![
                (1, 1, Some(1)),
                (7, 7, Some(1))
            ], 0
        );
    }
    
    // TODO: insert multi-char with multi selection bar/block semantics
    //TODO: test insert tab (hard/soft/tab width)
    //TODO: test insert newline
    
    #[test] fn errors_if_empty_insert_string(){
        test_error(
            CursorSemantics::Block, 
            "some\nshit\n", 
            vec![
                (0, 1, None),
                (5, 6, None)
            ], 0, 
            ""
        );
    
        test_error(
            CursorSemantics::Bar, 
            "some\nshit\n", 
            vec![
                (0, 0, None),
                (5, 5, None)
            ], 0, 
            ""
        );
    }
}

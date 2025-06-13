use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, Movement},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
    }
    Ok(())
}

/// Returns a new instance of [`Selection`] with cursor moved right to the nearest word boundary.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
    
    let goal_index = buffer.next_word_boundary(selection.head());
    match semantics{
        CursorSemantics::Bar => {
            selection.put_cursor(goal_index, buffer, Movement::Move, semantics, true)
        }
        CursorSemantics::Block => {
            if goal_index == buffer.len_chars(){
                selection.put_cursor(goal_index, buffer, Movement::Move, semantics, true)
            }else{
                selection.put_cursor(buffer.previous_grapheme_boundary_index(goal_index), buffer, Movement::Move, semantics, true)
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::utilities::move_cursor_word_boundary_forward;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::View,
    };

    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));

        let mut vec_expected_selections = Vec::new();
        for tuple in tuple_expected_selections{
            vec_expected_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let expected_selections = Selections::new(vec_expected_selections, expected_primary, &app.buffer, semantics.clone());
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        let result = move_cursor_word_boundary_forward::application_impl(&mut app, semantics.clone());
        assert!(!result.is_err());
        
        assert_eq!(expected_selections, app.selections);
        assert!(!app.buffer.is_modified());
    }
    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        assert!(move_cursor_word_boundary_forward::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_multiple_valid_selections_bar_semantics(){
        //                    1                   2
        //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        // u s e _ e r r o r : : E r r o r ; _ _ _ _
        test(
            CursorSemantics::Bar, 
            "use error::Error;    ",    //len 21    text end: (21, 21)    doc end: (21, 21)
            vec![
                (0, 0, None),   //common use
                (3, 3, None),   //skips whitespace and moves to next ending word boundary
                (9, 9, None),   //non alpha_numeric or whitespace jumps to next non whitespace
                (11, 16, None), //extended collapses then moves normally
                (17, 17, None)  //skips whitespace and moves to doc end if no other alphanumeric
            ], 0, 
            vec![
                (3, 3, Some(3)),
                (9, 9, Some(9)),
                (10, 10, Some(10)),
                (17, 17, Some(17)),
                (21, 21, Some(21))
            ], 0
        );
    }
    #[test] fn with_multiple_valid_selections_block_semantics(){
        //                    1                   2
        //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        // u s e _ e r r o r : : E r r o r ; _ _ _ _
        test(
            CursorSemantics::Block, 
            "use error::Error;    ",    //len 21    text end: (20, 21)    doc end: (21, 22)
            vec![
                (0, 1, None),   //common use
                (2, 3, None),   //skips whitespace and moves to next ending word boundary
                (8, 9, None),   //non alpha_numeric or whitespace jumps to next non whitespace
                (11, 16, None), //extended collapses then moves normally
                (16, 17, None)  //skips whitespace and moves to doc end if no other alphanumeric
            ], 0, 
            vec![
                (2, 3, Some(2)),
                (8, 9, Some(8)),
                (9, 10, Some(9)),
                (16, 17, Some(16)),
                (21, 22, Some(21))
            ], 0
        );
    }
    
    #[test] fn with_mixed_valid_and_invalid_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 3, None),   //valid + line to line updates stored line position
                (14, 14, None)  //invalid
            ], 0, 
            vec![
                (8, 8, Some(4)),
                (14, 14, None)
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 4, None),   //valid + line to line updates stored line position
                (14, 15, None)  //invalid
            ], 0, 
            vec![
                (7, 8, Some(3)),
                (14, 15, None)
            ], 0
        );
    }
    
    #[test] fn errors_when_single_selection_at_doc_end_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 14, None)
            ], 0
        );
    }
    #[test] fn errors_when_single_selection_at_doc_end_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 15, None)
            ], 0
        );
    }
}

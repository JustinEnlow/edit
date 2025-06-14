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

/// Returns a new instance of [`Selection`] with cursor moved left to the nearest word boundary.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    
    let goal_index = buffer.previous_word_boundary(selection.cursor(buffer, semantics.clone()));
    selection.put_cursor(goal_index, buffer, Movement::Move, semantics, true)
}

#[cfg(test)]
mod tests{
    use crate::utilities::move_cursor_word_boundary_backward;
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
        
        let result = move_cursor_word_boundary_backward::application_impl(&mut app, semantics.clone());
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
        
        assert!(move_cursor_word_boundary_backward::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_multiple_valid_selections_bar_semantics(){
        //                    1                   2
        //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        // _ _ _ _ u s e _ e r r o r : : E r r o r ;
        test(
            CursorSemantics::Bar, 
            "    use error::Error;",    //len 21    text end: (21, 21)  doc end: (21, 21)
            vec![
                (4, 4, None),   //skips whitespace and moves to doc start if no other alphanumeric
                (8, 8, None),   //skips whitespace and moves to next starting word boundary
                (14, 14, None), //non alpha_numeric or whitespace jumps to previous non whitespace
                (20, 15, None), //extended collapses then moves normally
                (21, 21, None)  //common use
            ], 0, 
            vec![
                (0, 0, Some(0)),
                (4, 4, Some(4)),
                (13, 13, Some(13)),
                (14, 14, Some(14)),
                (20, 20, Some(20))
            ], 0
        );
    }
    #[test] fn with_multiple_valid_selections_block_semantics(){
        //                    1                   2
        //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        // _ _ _ _ u s e _ e r r o r : : E r r o r ;
        test(
            CursorSemantics::Block, 
            "    use error::Error;",    //len 21    text end: (20, 21)  doc end: (21, 22)
            vec![
                (4, 5, None),   //skips whitespace and moves to doc start if no other alphanumeric
                (8, 9, None),   //skips whitespace and moves to next starting word boundary
                (14, 15, None), //non alpha_numeric or whitespace jumps to previous non whitespace
                (20, 15, None), //extended collapses then moves normally
                (21, 22, None)  //common use
            ], 0, 
            vec![
                (0, 1, Some(0)),
                (4, 5, Some(4)),
                (13, 14, Some(13)),
                (14, 15, Some(14)),
                (20, 21, Some(20))
            ], 0
        );
    }

    #[test] fn with_mixed_valid_and_invalid_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),   //invalid
                (9, 9, None)    //valid + line to line updates stored line position
            ], 0, 
            vec![
                (0, 0, None),
                (4, 4, Some(0))
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),   //invalid
                (9, 10, None)   //valid + line to line updates stored line position
            ], 0, 
            vec![
                (0, 1, None),
                (4, 5, Some(0))
            ], 0
        );
    }

    #[test] fn errors_when_single_selection_at_doc_end_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None)
            ], 0
        );
    }
    #[test] fn errors_when_single_selection_at_doc_end_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 0
        );
    }
}

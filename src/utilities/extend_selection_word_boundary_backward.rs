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

/// Returns a new instance of [`Selection`] with cursor extended left to the nearest word boundary.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    
    let goal_index = buffer.previous_word_boundary(selection.cursor(buffer, semantics.clone()));
    selection.put_cursor(goal_index, buffer, Movement::Extend, semantics, true)
}

#[cfg(test)]
mod tests{
    use crate::utilities::extend_selection_word_boundary_backward;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::View,
    };

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
        
        let result = extend_selection_word_boundary_backward::application_impl(&mut app, semantics.clone());
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
        
        assert!(extend_selection_word_boundary_backward::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_multiple_valid_selections(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (2, 3, None),
                (7, 8, None)
            ], 0, 
            vec![
                (3, 0, Some(0)),
                (8, 4, Some(0))
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (7, 8, None)
            ], 0, 
            vec![
                (0, 1, None),
                (8, 4, Some(0))
            ], 0
        );
    }
    
    #[test] fn extends_to_doc_start_if_no_other_word_boundaries(){
        test(
            CursorSemantics::Block, 
            "    idk\nsome\nshit\n", 
            vec![
                (4, 5, None)
            ], 0, 
            vec![
                (5, 0, Some(0))
            ], 0
        );
    }
    
    #[test] fn shrinks_previously_forward_extended_selection(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 14, None)
            ], 0, 
            vec![
                (0, 10, Some(0))
            ], 0
        );
    }
    
    #[test] fn errors_if_single_selection_at_doc_start(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 0
        );
    }
    #[test] fn errors_if_already_extended_backwards_to_doc_start(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 0, None)
            ], 0
        );
    }

    //#[test] fn extend_left_word_boundary(){
    //    let text = Rope::from("use std::error::Error;");
    //    assert_eq!(Selection::with_stored_line_position(Range::new(0, 4), Direction::Backward, 0), Selection::new(Range::new(3, 4), Direction::Forward).extend_left_word_boundary(&text, CursorSemantics::Block).unwrap());
    //    assert_eq!(Selection::with_stored_line_position(Range::new(0, 3), Direction::Backward, 0), Selection::new(Range::new(3, 3), Direction::Forward).extend_left_word_boundary(&text, CursorSemantics::Bar).unwrap());
    //}
    //#[test] fn extend_left_word_boundary_error(){
    //    let text = Rope::from("idk\nsome\nshit\n");
    //    assert!(Selection::new(Range::new(0, 1), Direction::Forward).extend_left_word_boundary(&text, CursorSemantics::Block).is_err());
    //    assert!(Selection::new(Range::new(0, 0), Direction::Forward).extend_left_word_boundary(&text, CursorSemantics::Bar).is_err());
    //}
}

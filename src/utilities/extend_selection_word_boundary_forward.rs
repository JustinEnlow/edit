use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, Movement},
    selections::SelectionsError,
    view::DisplayArea
};

pub fn application_impl(app: &mut Application, count: usize, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    //match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
    match app.selections.move_selection(count, &app.buffer, display_area, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
    }
    Ok(())
}

//TODO: this seems to be misbehaving when selection already extend left word boundary, and then extend right word boundary triggered.
//only when cursor over character that can be a beginning or ending word boundary...
/// Returns a new instance of [`Selection`] with cursor extended right to the nearest word boundary.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{  //TODO: ensure this can't extend past doc text end
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());

    selection.assert_invariants(buffer, semantics.clone());
    if selection.range.start == buffer.len_chars()
    || selection.range.end == buffer.len_chars()
    || selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
        
    //let goal_index = buffer.next_word_boundary(selection.head());
    let mut goal_index = selection.head();
    for _ in 0..count{
        let next_word_boundary = buffer.next_word_boundary(selection.head());
        //goal_index = buffer.next_word_boundary(selection.head());
        if goal_index == next_word_boundary{break;} //break out of loop early if we are already on the last grapheme
        goal_index = next_word_boundary;
    }
    match semantics{
        CursorSemantics::Bar => {
            selection.put_cursor(goal_index, buffer, Movement::Extend, semantics, true)
        }
        CursorSemantics::Block => {
            if goal_index == buffer.len_chars(){
                //self.put_cursor(goal_index, text, Movement::Extend, semantics, true)
                selection.put_cursor(buffer.previous_grapheme_boundary_index(buffer.len_chars()), buffer, Movement::Extend, semantics, true)
            }else{
                selection.put_cursor(
                    buffer.previous_grapheme_boundary_index(goal_index), 
                    buffer, 
                    Movement::Extend, 
                    semantics, 
                    true
                )
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::utilities::extend_selection_word_boundary_forward;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::DisplayArea,
    };
    use crate::utilities::test;

    //fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
    //    let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
    //
    //    let mut vec_expected_selections = Vec::new();
    //    for tuple in tuple_expected_selections{
    //        vec_expected_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
    //    }
    //    let expected_selections = Selections::new(vec_expected_selections, expected_primary, &app.buffer, semantics.clone());
    //    
    //    let mut vec_selections = Vec::new();
    //    for tuple in tuple_selections{
    //        vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
    //    }
    //    let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
    //    
    //    app.selections = selections;
    //    
    //    let result = extend_selection_word_boundary_forward::application_impl(&mut app, semantics.clone());
    //    assert!(!result.is_err());
    //    
    //    assert_eq!(expected_selections, app.selections);
    //    assert!(!app.buffer.is_modified());
    //}
    //fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
    //    let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
    //    
    //    let mut vec_selections = Vec::new();
    //    for tuple in tuple_selections{
    //        vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
    //    }
    //    let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
    //    
    //    app.selections = selections;
    //    
    //    assert!(extend_selection_word_boundary_forward::application_impl(&mut app, semantics).is_err());
    //    assert!(!app.buffer.is_modified());
    //}

    //#[test] fn sanity_check(){
    //    let text = Rope::from("idk\nsome\nshit\n");
    //    assert_eq!(14, text.len_chars());
    //}
    //
    //#[test] fn extend_right_word_boundary(){
    //    test(
    //        CursorSemantics::Block, 
    //        "use std::error::Error;", 
    //        vec![
    //            Selection::new(Range::new(0, 1), Direction::Forward)
    //        ], 0, 
    //        vec![
    //            Selection::with_stored_line_position(Range::new(0, 3), Direction::Forward, 2)
    //        ], 0
    //    );
    //    test(
    //        CursorSemantics::Bar, 
    //        "use std::error::Error;", 
    //        vec![
    //            Selection::new(Range::new(0, 0), Direction::Forward)
    //        ], 0, 
    //        vec![
    //            Selection::with_stored_line_position(Range::new(0, 3), Direction::Forward, 3)
    //        ], 0
    //    );
    //}

    #[test] fn implement_tests_using_count(){
        unimplemented!()
    }

    #[test] fn with_multiple_valid_selections(){
        test::selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
            1,
            None,
            vec![
                (0, 3, Some(2)),
                (4, 8, Some(3))
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections(){
        test::selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (13, 14, None)
            ], 0, 
            1,
            None,
            vec![
                (0, 3, Some(2)),
                (13, 14, None)
            ], 0
        );
    }
    //should error if single selection at doc end
    //TODO: test with previously forward extended, with cursor over non word char

    #[test] fn normal_use_bar_semantics(){
        test::selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None)
            ], 0, 
            1,
            None,
            vec![
                (0, 3, Some(3))
            ], 0
        );
    }
    #[test] fn normal_use_block_semantics(){
        test::selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 0, 
            1,
            None,
            vec![
                (0, 3, Some(2))
            ], 0
        );
    }
    
    #[test] fn extends_to_doc_end_from_doc_text_end_bar_semantics(){
        test::selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (13, 13, None)
            ], 0, 
            1,
            None,
            vec![
                (13, 14, Some(0))
            ], 0
        );
    }
    #[test] fn extends_to_doc_end_from_doc_text_end_block_semantics(){
        test::selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (12, 13, None)
            ], 0, 
            1,
            None,
            vec![
                (12, 14, Some(4))
            ], 0
        );
    }

    #[test] fn errors_if_cursor_at_doc_end_bar_semantics(){
        test::error_selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 14, None)
            ], 0,
            1,
            None
        );
    }
    #[test] fn errors_if_cursor_at_doc_end_block_semantics(){
        test::error_selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (13, 14, None)
            ], 0,
            1,
            None
        );
    }

    #[test] fn errors_if_already_extended_forward_to_doc_end_bar_semantics(){
        test::error_selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 14, None)
            ], 0,
            1,
            None
        );
    }
    #[test] fn errors_if_already_extended_forward_to_doc_end_block_semantics(){
        test::error_selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 14, None)
            ], 0,
            1,
            None
        );
    }

    //TODO: actually, this should work... it should move the cursor from 0 to 3...
    #[test] fn errors_if_already_extended_backward_from_doc_end_bar_semantics(){
        test::error_selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 0, None)
            ], 0,
            1,
            None
        );
    }
    #[test] fn errors_if_already_extended_backward_from_doc_end_block_semantics(){
        test::error_selection_movement_with_count(
            extend_selection_word_boundary_forward::application_impl,
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 0, None)
            ], 0,
            1,
            None
        );
    }
}

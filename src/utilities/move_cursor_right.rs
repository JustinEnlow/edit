use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection/*, Movement */},
    selections::SelectionsError,
    view::DisplayArea,
};

pub fn application_impl(app: &mut Application, count: usize, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    //match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
    match app.selections.move_selection(count, &app.buffer, display_area, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
    }
    Ok(())
}

/// Returns a new instance of [`Selection`] with cursor moved down.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(!display_area.is_some());

//    let mut selection = selection.clone();
    selection.assert_invariants(buffer, semantics.clone());
    
    if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){
        return Err(SelectionError::ResultsInSameState);
    }
//
//    //let next = buffer.next_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
//    let mut next = selection.cursor(buffer, semantics.clone());
//    for _ in 0..count{
//        next = buffer.next_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
//    };
//    let new_position = next.min(buffer.len_chars()); //ensures this does not move past text end      //could match on semantics, and ensure extend does index.min(previous_grapheme_index(text.len_chars()))
//    
//    selection.range.start = new_position;
//    selection.range.end = match semantics.clone(){
//        CursorSemantics::Bar => new_position.min(buffer.len_chars()),
//        CursorSemantics::Block => buffer.next_grapheme_boundary_index(new_position).min(buffer.len_chars().saturating_add(1))
//    };
//    selection.direction = ExtensionDirection::None;
//    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
//    
//    selection.assert_invariants(buffer, semantics.clone());
//
//    Ok(selection)
    selection.move_horizontally(count, buffer, crate::selection::Movement::Move, ExtensionDirection::Forward, semantics)
    
}

//#[cfg(test)]
//mod tests{
//    use crate::utilities::move_cursor_right;
//    use crate::{
//        application::Application,
//        selections::Selections,
//        selection::{Selection, CursorSemantics},
//        view::DisplayArea,
//    };
//
//    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
//    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
//        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
//
//        let mut vec_expected_selections = Vec::new();
//        for tuple in tuple_expected_selections{
//            vec_expected_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let expected_selections = Selections::new(vec_expected_selections, expected_primary, &app.buffer, semantics.clone());
//        
//        let mut vec_selections = Vec::new();
//        for tuple in tuple_selections{
//            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
//        
//        app.selections = selections;
//        
//        let result = move_cursor_right::application_impl(&mut app, semantics.clone());
//        assert!(!result.is_err());
//        
//        assert_eq!(expected_selections, app.selections);
//        assert!(!app.buffer.is_modified());
//    }
//    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
//        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
//        
//        let mut vec_selections = Vec::new();
//        for tuple in tuple_selections{
//            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
//        
//        app.selections = selections;
//        
//        assert!(move_cursor_right::application_impl(&mut app, semantics).is_err());
//        assert!(!app.buffer.is_modified());
//    }
//
//    #[test] fn sanity_check(){
//        use crate::utilities::move_cursor_right::selection_impl;
//        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
//        let semantics = CursorSemantics::Block;
//        let result = selection_impl(&Selection::new_from_range(crate::range::Range::new(14, 15), crate::selection::ExtensionDirection::None, buffer, semantics.clone()), buffer, semantics.clone());
//        println!("{:?}", result);
//        assert!(result.is_err());
//    }
//
//    #[test] fn with_multiple_valid_selections_bar_semantics(){
//        test(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 0, None),   //common use
//                (8, 8, None),   //line to line updates stored line position
//                (10, 13, None)  //extended selection collapses then moves normally
//            ], 0, 
//            vec![
//                (1, 1, Some(1)),
//                (9, 9, Some(0)),
//                (14, 14, Some(0))
//            ], 0
//        );
//    }
//    #[test] fn with_multiple_valid_selections_block_semantics(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 1, None),   //common use
//                (8, 9, None),   //line to line updates stored line position
//                (10, 13, None)  //extended selection collapses then moves normally
//            ], 0, 
//            vec![
//                (1, 2, Some(1)),
//                (9, 10, Some(0)),
//                (13, 14, Some(4))
//            ], 0
//        );
//    }
//    
//    #[test] fn with_mixed_valid_and_invalid_selections_bar_semantics(){
//        test(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 0, None),   //valid
//                (14, 14, None)  //invalid
//            ], 0, 
//            vec![
//                (1, 1, Some(1)),
//                (14, 14, None)
//            ], 0
//        );
//    }
//    #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 1, None),   //valid
//                (14, 15, None)  //invalid
//            ], 0, 
//            vec![
//                (1, 2, Some(1)),
//                (14, 15, None)
//            ], 0
//        );
//    }
//    
//    #[test] fn with_single_selection_at_doc_end_bar_semantics(){
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (14, 14, None)
//            ], 0
//        );
//    }
//    #[test] fn with_single_selection_at_doc_end_block_semantics(){
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (14, 15, None)
//            ], 0
//        );
//    }
//}

use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection/*, Movement */},
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

pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(!display_area.is_some());

//    let mut selection = selection.clone();
    selection.assert_invariants(buffer, semantics.clone());
    
    if selection.range.start == buffer.len_chars()
    || selection.range.end == buffer.len_chars()
    || selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}

//    //let next = buffer.next_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
//    let mut next = selection.cursor(buffer, semantics.clone());
//    for _ in 0..count{
//        next = buffer.next_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
//    };
//    let new_position = next.min(buffer.len_chars()); //ensures this does not move past text end      //could match on semantics, and ensure extend does index.min(previous_grapheme_index(text.len_chars()))
//    
//    match semantics.clone(){
//        CursorSemantics::Bar => {
//            let to = Ord::min(new_position, buffer.len_chars());
//            let (start, end, direction) = if to < selection.anchor(){
//                (to, selection.anchor(), ExtensionDirection::Backward)
//            }else{
//                (selection.anchor(), to, ExtensionDirection::Forward)
//            };
//            selection.range.start = start;
//            selection.range.end = end;
//            selection.direction = direction;
//        }
//        CursorSemantics::Block => {
//            let to = Ord::min(new_position, buffer.previous_grapheme_boundary_index(buffer.len_chars()));
//            let new_anchor = match selection.direction{
//                ExtensionDirection::None |
//                ExtensionDirection::Forward => {
//                    if to < selection.anchor(){  //could also do self.range.start
//                        if let Some(char_at_cursor) = buffer.get_char(selection.cursor(buffer, semantics.clone())){
//                            if char_at_cursor == '\n'{selection.anchor()}
//                            else{buffer.next_grapheme_boundary_index(selection.anchor()).min(buffer.len_chars())}
//                        }else{buffer.next_grapheme_boundary_index(selection.anchor()).min(buffer.len_chars())}
//                    }else{selection.anchor()}
//                }
//                ExtensionDirection::Backward => {
//                    if to >= selection.anchor(){buffer.previous_grapheme_boundary_index(selection.anchor())} //could also do self.range.end
//                    else{selection.anchor()}
//                }
//            };
//
//            if new_anchor <= to{    //allowing one more char past text.len_chars() for block cursor
//                selection.range.start = new_anchor;
//                selection.range.end = Ord::min(buffer.next_grapheme_boundary_index(to), buffer.len_chars().saturating_add(1));
//                selection.direction = ExtensionDirection::Forward;
//            }else{
//                selection.range.start = to;
//                selection.range.end = new_anchor;
//                selection.direction = ExtensionDirection::Backward;
//            }
//        }
//    }
//
//    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
//    
//    selection.assert_invariants(buffer, semantics.clone());
//
//    Ok(selection)
    selection.move_horizontally(count, buffer, crate::selection::Movement::Extend, ExtensionDirection::Forward, semantics)
}

//#[cfg(test)]
//mod tests{
//    use crate::utilities::extend_selection_right;
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
//        let result = extend_selection_right::application_impl(&mut app, semantics.clone());
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
//        assert!(extend_selection_right::application_impl(&mut app, semantics).is_err());
//        assert!(!app.buffer.is_modified());
//    }
//
//    //use ropey::Rope;
//    //use crate::range::Range;
//    //use crate::selection::{Selection, CursorSemantics, Direction};
//
//    //#[test] fn sanity_check(){
//    //    let text = Rope::from("idk\nsome\nshit\n");
//    //    assert_eq!(14, text.len_chars());
//    //}
//
//    #[test] fn normal_use_bar_semantics(){
//        test(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 0, None)
//            ], 0, 
//            vec![
//                (0, 1, Some(1))
//            ], 0
//        );
//    }
//    #[test] fn normal_use_block_semantics(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 1, None)
//            ], 0, 
//            vec![
//                (0, 2, Some(1))
//            ], 0
//        );
//    }
//
//    #[test] fn extends_to_doc_text_end_bar_semantics(){
//        test(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (13, 13, None)
//            ], 0, vec![
//                (13, 14, Some(0))
//            ], 0
//        );
//    }
//    #[test] fn extends_to_doc_text_end_block_semantics(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (12, 13, None)
//            ], 0, 
//            vec![
//                (12, 14, Some(4))
//            ], 0
//        );
//    }
//
//    //TODO:
//    //#[test] fn with_previously_backward_extended_selection(){
//    //    test(
//    //        CursorSemantics::Bar, 
//    //        "idk\nsome\nshit\n", 
//    //        vec![Selection::new(Range::new(0, 14), Direction::Backward)], 0,
//    //        vec![Selection::with_stored_line_position(Range::new(1, 14), Direction::Backward, 1)], 0
//    //    );
//    //}
//
//    #[test] fn errors_if_cursor_at_doc_text_end_bar_semantics(){
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (14, 14, None)
//            ], 0
//        );
//    }
//    #[test] fn errors_if_cursor_at_doc_text_end_block_semantics(){
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (13, 14, None)
//            ], 0
//        );
//    }
//
//    #[test] fn errors_if_already_extended_forward_at_doc_text_end_bar_semantics(){
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 14, None)
//            ], 0
//        );
//    }
//    #[test] fn errors_if_already_extended_forward_at_doc_text_end_block_semantics(){
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 14, None)
//            ], 0
//        );
//    }
//}

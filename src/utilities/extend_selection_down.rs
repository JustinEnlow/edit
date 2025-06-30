use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection, Movement},
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

/// Returns a new instance of [`Selection`] with the [`Selection`] extended down.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());

    selection.assert_invariants(buffer, semantics.clone());
    let last_line = buffer.len_lines().saturating_sub(1);
    if buffer.char_to_line(selection.range.start) == last_line
    || buffer.char_to_line(selection.range.end) == last_line
    || buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == last_line{return Err(SelectionError::ResultsInSameState);}

    selection.move_vertically(count, buffer, Movement::Extend, ExtensionDirection::Forward, semantics)
}

//#[cfg(test)]
//mod tests{
//    use crate::utilities::extend_selection_down;
//    use crate::{
//        application::Application,
//        selections::Selections,
//        selection::{Selection, CursorSemantics},
//        view::DisplayArea,
//    };
//
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
//        let result = extend_selection_down::application_impl(&mut app, semantics.clone());
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
//        assert!(extend_selection_down::application_impl(&mut app, semantics).is_err());
//        assert!(!app.buffer.is_modified());
//    }
//
//    #[test] fn with_multiple_valid_selections_bar_semantics(){
//        test(
//            CursorSemantics::Bar, 
//            "some\nshit\nidk\n", 
//            vec![
//                (0, 0, None),   //common use
//                (9, 9, None)    //to shorter line
//            ], 0, 
//            vec![
//                (0, 5, Some(0)),
//                (9, 13, Some(4))
//            ], 0
//        );
//    }
//    #[test] fn with_multiple_valid_selections_block_semantics(){
//        test(
//            CursorSemantics::Block, 
//            "some\nshit\nidk\n", 
//            vec![
//                (0, 1, None),   //common use
//                (9, 10, None)   //to shorter line
//            ], 0, 
//            vec![
//                (0, 6, Some(0)),
//                (9, 14, Some(4))
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
//                (0, 4, Some(0)),
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
//                (0, 5, Some(0)),
//                (14, 15, None)
//            ], 0
//        );
//    }
//
//    #[test] fn errors_when_single_selection_on_bottom_line_bar_semantics(){
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (14, 14, None)
//            ], 0
//        );
//    }
//    #[test] fn errors_when_single_selection_on_bottom_line_block_semantics(){
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (14, 15, None)
//            ], 0
//        );
//    }
//}

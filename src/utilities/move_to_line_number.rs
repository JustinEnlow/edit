use crate::{
    //application::{Application, ApplicationError},
    selection::{Selection, SelectionError, /*Extension*/Direction, CursorSemantics, Movement},
    //selections::SelectionsError
};

//front end converts to zero-based
//pub fn application_impl(app: &mut Application, line_number: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    //assert!(line_number > 0);
//    //let line_number = line_number.saturating_sub(1);    //convert to zero based //should this conversion really be happening on the back end?
//    if line_number >= app.buffer.len_lines(){return Err(ApplicationError::InvalidInput);}
//    
//    //if let Ok(()) = document.clear_non_primary_selections(){};
//    if let Ok(()) = crate::utilities::clear_non_primary_selections::application_impl(app){};
//    match selection_impl(app.selections.primary(), line_number, &app.buffer, Movement::Move, semantics){
//        Ok(new_selection) => {*app.selections.primary_mut() = new_selection;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}    //should be same state error
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with the cursor set to specified 0-based line number.
pub fn selection_impl(selection: &Selection, line_number: usize, buffer: &crate::buffer::Buffer, movement: Movement, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    //selection.assert_invariants(buffer, semantics.clone());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    assert!(line_number < buffer.len_lines());

    if line_number == buffer.char_to_line(selection.cursor(buffer, semantics.clone())){return Err(SelectionError::ResultsInSameState);}
    
    let current_line = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let (amount, direction) = if line_number < current_line{
        (current_line.saturating_sub(line_number), /*Extension*/Direction::Backward)
    }else{
        (line_number.saturating_sub(current_line), /*Extension*/Direction::Forward)
    };
    selection.move_vertically(amount, buffer, movement, direction, semantics)
}

//#[cfg(test)]
//mod tests{
//    use crate::utilities::move_to_line_number;
//    use crate::{
//        application::Application,
//        selections::Selections,
//        selection::{Selection, CursorSemantics},
//        display_area::DisplayArea,
//    };
//
//    fn test(semantics: CursorSemantics, text: &str, line_number: usize, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
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
//        let result = move_to_line_number::application_impl(&mut app, line_number, semantics.clone());
//        assert!(!result.is_err());
//        
//        assert_eq!(expected_selections, app.selections);
//        assert!(!app.buffer.is_modified());
//    }
//    fn test_error(semantics: CursorSemantics, text: &str, line_number: usize, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
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
//        assert!(move_to_line_number::application_impl(&mut app, line_number, semantics).is_err());
//        assert!(!app.buffer.is_modified());
//    }
//
//    //TODO: restricts cursor to line end, when stored line position > line width
//
//    #[test] fn moves_to_line_number_bar_semantics(){
//        test(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            2, 
//            vec![
//                (0, 0, None),
//                (4, 4, None)
//            ], 0, 
//            vec![
//                (9, 9, Some(0))
//            ], 0
//        );
//    }
//    #[test] fn moves_to_line_number_block_semantics(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            2, 
//            vec![
//                (0, 1, None),
//                (4, 5, None)
//            ], 0, 
//            vec![
//                (9, 10, Some(0))
//            ], 0
//        );
//    }
//
//    #[test] fn errors_if_already_at_line_number_bar_semantics(){
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            1, 
//            vec![
//                (4, 4, None)
//            ], 0
//        );
//    }
//    #[test] fn errors_if_already_at_line_number_block_semantics(){
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            1, 
//            vec![
//                (4, 5, None)
//            ], 0
//        );
//    }
//    
//    #[test] fn errors_if_invalid_line_number_bar_semantics(){   //0 is valid, since backend line numbers are 0 based
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            500, 
//            vec![
//                (0, 0, None)
//            ], 0
//        );
//    }
//    #[test] fn errors_if_invalid_line_number_block_semantics(){
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            500, 
//            vec![
//                (0, 1, None)
//            ], 0
//        );
//    }
//}

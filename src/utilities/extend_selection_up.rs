use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Direction, Movement},
    display_area::DisplayArea
};

//pub fn application_impl(app: &mut Application, count: usize, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    //match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
//    match app.selections.move_selection(count, &app.buffer, display_area, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended up.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());

    selection.assert_invariants(buffer, semantics.clone());
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.move_vertically(count, buffer, Movement::Extend, /*Extension*/Direction::Backward, semantics)
}

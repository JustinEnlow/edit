use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection, Movement},
    selections::SelectionsError,
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

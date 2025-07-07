use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Movement}
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_non_overlapping(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with `cursor` aligned with anchor.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    if !selection.is_extended(){return Err(SelectionError::ResultsInSameState);}
    //selection.put_cursor(selection.cursor(buffer, semantics.clone()), buffer, Movement::Move, semantics, true)
    //if we want collapse to anchor:
    selection.put_cursor(selection.anchor(), buffer, Movement::Move, semantics, true)
}


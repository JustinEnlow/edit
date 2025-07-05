use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, Movement}
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_non_overlapping(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
//    }
//    Ok(())
//}

//TODO: we should allow collapsing to anchor, or collapse to anchor collapse(&self, text: &Rope, semantics: CursorSemantics, collapse_target: Anchor)
/// Returns a new instance of [`Selection`] with `anchor` aligned with cursor.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    if !selection.is_extended(){return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(selection.cursor(buffer, semantics.clone()), buffer, Movement::Move, semantics, true)
    //if we want collapse to anchor:
    //self.put_cursor(self.anchor, text, Movement::Move, semantics, true)
}

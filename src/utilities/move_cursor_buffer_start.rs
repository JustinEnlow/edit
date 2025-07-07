use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Movement},
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with the cursor moved to the start of the document.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(0, buffer, Movement::Move, semantics, true)
}

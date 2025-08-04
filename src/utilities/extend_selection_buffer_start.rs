use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Movement},
};

/// Returns a new instance of [`Selection`] with the selection extended to the start of the buffer.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(0, buffer, Movement::Extend, semantics, true)
}

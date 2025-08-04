use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Movement},
};

/// Returns a new instance of [`Selection`] with the selection extended to the end of the buffer.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(buffer.len_chars(), buffer, Movement::Extend, semantics, true)
}

use crate::{
    display_area::DisplayArea,
    selection::{Selection, SelectionError, CursorSemantics, Direction, Movement}
};

/// Returns a new instance of [`Selection`] with the selection extended down by the height of `client_view`.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let client_view = match display_area{
        Some(client_view) => client_view,
        None => return Err(SelectionError::ResultsInSameState), //maybe need a better error
    };
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}
    selection.move_vertically(
        count.saturating_mul(client_view.height.saturating_sub(1)),
        buffer, 
        Movement::Extend, 
        Direction::Forward, 
        semantics
    )
}

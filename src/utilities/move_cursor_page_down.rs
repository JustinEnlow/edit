use crate::{
    application::{Application, ApplicationError},
    view::View,
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection, Movement}
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match app.selections.move_cursor_page(&app.buffer, &app.view, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
    }
    Ok(())
}

/// Returns a new instance of [`Selection`] with the cursor moved down by the height of `client_view`.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, client_view: &View, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}
    selection.move_vertically(client_view.height().saturating_sub(1), buffer, Movement::Move, ExtensionDirection::Forward, semantics)
}
use crate::{
    application::{Application, ApplicationError},
    display_area::DisplayArea,
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection, Movement}
};

//pub fn application_impl(app: &mut Application, count: usize, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    //match app.selections.move_cursor_page(&app.buffer, &app.buffer_display_area, semantics, selection_impl){
//    match app.selections.move_selection(count, &app.buffer, display_area, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with the cursor moved down by the height of `client_view`.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let client_view = match display_area{
        Some(client_view) => client_view,
        None => return Err(SelectionError::ResultsInSameState), //maybe need a better error
    };
    selection.assert_invariants(buffer, semantics.clone());
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}
    //selection.move_vertically(client_view.height().saturating_sub(1), buffer, Movement::Move, ExtensionDirection::Forward, semantics)
    selection.move_vertically(
        count.saturating_mul(client_view.height().saturating_sub(1)),
        buffer, 
        Movement::Move, 
        ExtensionDirection::Forward, 
        semantics
    )
}
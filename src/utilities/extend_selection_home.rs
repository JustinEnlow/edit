use crate::{
    selection::{Selection, SelectionError, CursorSemantics},
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended to absolute start of line, or line text start, depending on [`Selection`] `head` position.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    use crate::utilities::extend_selection_line_text_start;
    use crate::utilities::extend_selection_line_start;
    
    selection.assert_invariants(buffer, semantics.clone());
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_start = buffer.line_to_char(line_number);
    let text_start_offset = buffer.first_non_whitespace_character_offset(line_number);
    let text_start = line_start.saturating_add(text_start_offset);  //nth_next_grapheme_index(line_start, text_start_offset, text)?

    if selection.cursor(buffer, semantics.clone()) == text_start{extend_selection_line_start::selection_impl(selection, buffer, semantics.clone())}
    else{extend_selection_line_text_start::selection_impl(selection, buffer, semantics)}
}

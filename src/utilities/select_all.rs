use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Movement},
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_clearing_non_primary(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with [`Selection`] extended to encompass all text.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{  //TODO: ensure this can't extend past doc text end
    selection.assert_invariants(buffer, semantics.clone());
    if selection.range.start == 0 
    && (
        selection.range.end == buffer.len_chars() || 
        selection.range.end == buffer.len_chars().saturating_add(1)
    ){return Err(SelectionError::ResultsInSameState);}
    
    let selection = selection.put_cursor(0, buffer, Movement::Move, semantics.clone(), true)?;
    selection.put_cursor(
        match semantics{
            CursorSemantics::Bar => buffer.len_chars(), 
            CursorSemantics::Block => buffer.previous_grapheme_boundary_index(buffer.len_chars())
        }, 
        buffer, 
        Movement::Extend, 
        semantics, 
        true
    )
}

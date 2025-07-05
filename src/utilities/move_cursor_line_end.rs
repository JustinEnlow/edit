use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection/*, Movement */},
    selections::SelectionsError
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with cursor moved down.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let mut selection = selection.clone();
    selection.assert_invariants(buffer, semantics.clone());
    
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_width = buffer.line_width(line_number, false);
    let line_start = buffer.line_to_char(line_number);
    let line_end = line_start.saturating_add(line_width);   //nth_next_grapheme_index(line_start, line_width, text)?

    if selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
    //selection.put_cursor(line_end, text, Movement::Move, semantics, true)
    
    selection.range.start = line_end;
    selection.range.end = match semantics.clone(){
        CursorSemantics::Bar => line_end.min(buffer.len_chars()),
        CursorSemantics::Block => buffer.next_grapheme_boundary_index(line_end).min(buffer.len_chars().saturating_add(1))
    };
    selection.direction = ExtensionDirection::None;
    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
    
    selection.assert_invariants(buffer, semantics.clone());

    Ok(selection)
}

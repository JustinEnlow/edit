use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Direction},
    display_area::DisplayArea
};

//pub fn application_impl(app: &mut Application, count: usize, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    //match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
//    match app.selections.move_selection(count, &app.buffer, display_area, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with cursor moved down.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());

//    let mut selection = selection.clone();
    //selection.assert_invariants(buffer, semantics.clone());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){
        return Err(SelectionError::ResultsInSameState);
    }
    
//    let current_line = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
//    let goal_line_number = core::cmp::Ord::min(current_line.saturating_add(count), buffer.len_lines().saturating_sub(1));
//    let start_of_line = buffer.line_to_char(goal_line_number);
//    let line_width = buffer.line_width(goal_line_number, false);
//    
//    // Use current stored line offset or calculate it if None
//    let stored_line_offset = selection.stored_line_offset.unwrap_or_else(|| {
//        buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone()))
//    });
//
//    // Calculate the new position based on line width
//    let mut new_position = if stored_line_offset < line_width{
//        start_of_line + stored_line_offset
//    }else{
//        start_of_line + line_width
//    };
//    new_position = new_position.min(buffer.len_chars());
//    
//    selection.range.start = new_position;
//    selection.range.end = match semantics{
//        CursorSemantics::Bar => new_position,
//        CursorSemantics::Block => buffer.next_grapheme_boundary_index(new_position)
//    };
//    selection.direction = ExtensionDirection::None;
//    selection.stored_line_offset = Some(stored_line_offset);
//
//    selection.assert_invariants(buffer, semantics);
//    Ok(selection)
    selection.move_vertically(count, buffer, crate::selection::Movement::Move, /*Extension*/Direction::Forward, semantics)
}

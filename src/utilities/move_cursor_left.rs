use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection/*, Movement */},
    selections::SelectionsError,
    display_area::DisplayArea
};

//pub fn application_impl(app: &mut Application, count: usize, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    //match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
//    match app.selections.move_selection(count, &app.buffer, display_area, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with cursor moved down.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(!display_area.is_some());

//    let mut selection = selection.clone();
    selection.assert_invariants(buffer, semantics.clone());
    
    if !selection.is_extended() && selection.cursor(buffer, semantics.clone()) == 0{
        return Err(SelectionError::ResultsInSameState);
    }

//    //let new_position = buffer.previous_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
//    let mut new_position = selection.cursor(buffer, semantics.clone());
//    for _ in 0..count{
//        new_position = buffer.previous_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
//    }
//    
//    selection.range.start = new_position;
//    selection.range.end = match semantics.clone(){
//        CursorSemantics::Bar => new_position.min(buffer.len_chars()),
//        CursorSemantics::Block => buffer.next_grapheme_boundary_index(new_position).min(buffer.len_chars().saturating_add(1))
//    };
//    selection.direction = ExtensionDirection::None;
//    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
//    
//    selection.assert_invariants(buffer, semantics.clone());
//
//    Ok(selection)
    selection.move_horizontally(count, buffer, crate::selection::Movement::Move, ExtensionDirection::Backward, semantics)
}

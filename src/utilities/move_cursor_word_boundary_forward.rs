use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Movement},
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

/// Returns a new instance of [`Selection`] with cursor moved right to the nearest word boundary.
pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());

    //selection.assert_invariants(buffer, semantics.clone());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
    
    //let goal_index = buffer.next_word_boundary(selection.head());
    let mut goal_index = selection.head();
    for _ in 0..count{
        let next_word_boundary = buffer.next_word_boundary(selection.head());
        //goal_index = buffer.next_word_boundary(selection.head());
        if goal_index == next_word_boundary{break;} //break out of loop early if we are already on the last grapheme
        goal_index = next_word_boundary;
    }
    match semantics{
        CursorSemantics::Bar => {
            selection.put_cursor(goal_index, buffer, Movement::Move, semantics, true)
        }
        CursorSemantics::Block => {
            if goal_index == buffer.len_chars(){
                selection.put_cursor(goal_index, buffer, Movement::Move, semantics, true)
            }else{
                selection.put_cursor(buffer.previous_grapheme_char_index(goal_index), buffer, Movement::Move, semantics, true)
            }
        }
    }
}

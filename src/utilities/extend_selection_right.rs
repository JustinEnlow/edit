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

pub fn selection_impl(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());

//    let mut selection = selection.clone();
    selection.assert_invariants(buffer, semantics.clone());

    // i d k \n s o m e \n s h i t \n
    // block            // block            // bar
    // i d k \n         // i d k \n         // i d k \n
    // s o m e \n       // s o m e \n       // s o m e \n
    // s h i t|\n>      // s h i t \n       // s h i t \n
    //                  //| >               //|
    //if selection.range.start == buffer.len_chars()
    //|| selection.range.end == buffer.len_chars()
    //|| selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
    if (
        selection.range.start == buffer.len_chars() || 
        selection.range.end == buffer.len_chars() || 
        selection.cursor(buffer, semantics.clone()) == buffer.len_chars()
    ) && (    //needs to be able to shrink selection if extension_direction is Backward
        selection.extension_direction.is_none() ||
        selection.extension_direction == Some(Direction::Forward)
    ){return Err(SelectionError::ResultsInSameState);}

//    //let next = buffer.next_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
//    let mut next = selection.cursor(buffer, semantics.clone());
//    for _ in 0..count{
//        next = buffer.next_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
//    };
//    let new_position = next.min(buffer.len_chars()); //ensures this does not move past text end      //could match on semantics, and ensure extend does index.min(previous_grapheme_index(text.len_chars()))
//    
//    match semantics.clone(){
//        CursorSemantics::Bar => {
//            let to = Ord::min(new_position, buffer.len_chars());
//            let (start, end, direction) = if to < selection.anchor(){
//                (to, selection.anchor(), ExtensionDirection::Backward)
//            }else{
//                (selection.anchor(), to, ExtensionDirection::Forward)
//            };
//            selection.range.start = start;
//            selection.range.end = end;
//            selection.direction = direction;
//        }
//        CursorSemantics::Block => {
//            let to = Ord::min(new_position, buffer.previous_grapheme_boundary_index(buffer.len_chars()));
//            let new_anchor = match selection.direction{
//                ExtensionDirection::None |
//                ExtensionDirection::Forward => {
//                    if to < selection.anchor(){  //could also do self.range.start
//                        if let Some(char_at_cursor) = buffer.get_char(selection.cursor(buffer, semantics.clone())){
//                            if char_at_cursor == '\n'{selection.anchor()}
//                            else{buffer.next_grapheme_boundary_index(selection.anchor()).min(buffer.len_chars())}
//                        }else{buffer.next_grapheme_boundary_index(selection.anchor()).min(buffer.len_chars())}
//                    }else{selection.anchor()}
//                }
//                ExtensionDirection::Backward => {
//                    if to >= selection.anchor(){buffer.previous_grapheme_boundary_index(selection.anchor())} //could also do self.range.end
//                    else{selection.anchor()}
//                }
//            };
//
//            if new_anchor <= to{    //allowing one more char past text.len_chars() for block cursor
//                selection.range.start = new_anchor;
//                selection.range.end = Ord::min(buffer.next_grapheme_boundary_index(to), buffer.len_chars().saturating_add(1));
//                selection.direction = ExtensionDirection::Forward;
//            }else{
//                selection.range.start = to;
//                selection.range.end = new_anchor;
//                selection.direction = ExtensionDirection::Backward;
//            }
//        }
//    }
//
//    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
//    
//    selection.assert_invariants(buffer, semantics.clone());
//
//    Ok(selection)
    selection.move_horizontally(count, buffer, crate::selection::Movement::Extend, /*Extension*/Direction::Forward, semantics)
}

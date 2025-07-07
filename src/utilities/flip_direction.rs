use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Direction},
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_non_overlapping(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
//    }
//    Ok(())
//}

pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    //use crate::selection::ExtensionDirection;
    selection.assert_invariants(buffer, semantics.clone());
    if !selection.is_extended(){return Err(SelectionError::ResultsInSameState)}
    //Ok(
    //    Selection::new(
    //        selection.range.clone(), 
    //        match selection.direction{
    //            Direction::Forward => {Direction::Backward}
    //            Direction::Backward => {Direction::Forward}
    //        }
    //    )
    //)
    let mut new_selection = selection.clone();
    new_selection.extension_direction = match selection.extension_direction{
        None/*ExtensionDirection::None*/ => return Err(SelectionError::ResultsInSameState),
        Some(Direction::Forward)/*ExtensionDirection::Forward*/ => Some(Direction::Backward)/*ExtensionDirection::Backward*/,
        Some(Direction::Backward)/*ExtensionDirection::Backward*/ => Some(Direction::Forward)/*ExtensionDirection::Forward*/
    };
    Ok(new_selection)
}

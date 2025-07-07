use crate::{
    selection::{Selection, CursorSemantics},
    display_area::{DisplayArea, DisplayAreaError},
};

//pub fn application_impl(app: &mut Application, display_area: &DisplayArea, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match view_impl(display_area, app.selections.primary(), &app.buffer, semantics){
//        Ok(view) => {
//            //app.buffer_display_area = view
//            let DisplayArea{horizontal_start, vertical_start, width: _width, height: _height} = view;
//            app.buffer_horizontal_start = horizontal_start;
//            app.buffer_vertical_start = vertical_start;
//        }
//        Err(e) => {
//            match e{
//                DisplayAreaError::InvalidInput => {return Err(ApplicationError::InvalidInput);}
//                DisplayAreaError::ResultsInSameState => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
//            }
//        }
//    }
//
//    Ok(())
//}

/// Returns an instance of [`View`] vertically centered around specified cursor.
/// # Errors
/// when function output would return a `View` with the same state.
/// # Panics
/// when `selection` is invalid.
/// when `text` is invalid.
pub fn view_impl(view: &DisplayArea, selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<DisplayArea, DisplayAreaError>{
    assert!(selection.cursor(buffer, semantics.clone()) <= buffer.len_chars());    //ensure selection is valid
    assert!(buffer.len_lines() > 0);  //ensure text is not empty
        
    let current_line = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    //let view_is_even_numbered = self.height % 2 == 0;
    let half_view_height = view.height / 2; //current impl will be biased towards the bottom of the view, if view is even numbered

    //TODO: consider how even numbered view heights should be handled...
    // maybe < half_view_height.saturating_sub(1)
    if current_line <= half_view_height{return Err(DisplayAreaError::ResultsInSameState);} //maybe return error cursor before doc_start + half the view height
    if current_line >= buffer.len_lines().saturating_sub(half_view_height){return Err(DisplayAreaError::ResultsInSameState);}    //maybe return error cursor after doc_end - half the view height

    // Calculate the new vertical start position
    let new_vertical_start = if current_line > half_view_height{
        current_line.saturating_sub(half_view_height)
    }else{
        0
    }.min(buffer.len_lines().saturating_sub(view.height));    //should self.height be half_view_height?

    // if view_is_even_numbered && (current_line == new_vertical_start || current_line == new_vertical_start.saturating_sub(1)){return Err(ViewError::ResultsInSameState);}
    //if current_line == new_vertical_start{return Err(ViewError::ResultsInSameState);}   //maybe return error already centered   //TODO: and test
    //

    let new_view = DisplayArea::new(view.horizontal_start, new_vertical_start, view.width, view.height);    
    if new_view == view.clone(){return Err(DisplayAreaError::ResultsInSameState);} //can we catch this condition any earlier?...
    Ok(new_view)
}

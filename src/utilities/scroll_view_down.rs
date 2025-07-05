use crate::{
    application::{Application, ApplicationError},
    display_area::{DisplayArea, DisplayAreaError},
    selections::SelectionsError
};

//pub fn application_impl(app: &mut Application, display_area: &DisplayArea, amount: usize) -> Result<(), ApplicationError>{
//    match view_impl(display_area, amount, &app.buffer){
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

/// Returns a new instance of [`View`] with `vertical_start` increased by specified amount.
/// # Errors
/// when `amount` is 0.
/// when function would return a `View` with the same state.
/// # Panics
/// when `text` is invalid.
pub fn view_impl(view: &DisplayArea, amount: usize, buffer: &crate::buffer::Buffer) -> Result<DisplayArea, DisplayAreaError>{
    assert!(buffer.len_lines() > 0);

    if amount == 0{return Err(DisplayAreaError::InvalidInput);}

    let max_scrollable_position = buffer.len_lines().saturating_sub(view.height);
    if view.vertical_start == max_scrollable_position{return Err(DisplayAreaError::ResultsInSameState);}
    
    let new_vertical_start = view.vertical_start.saturating_add(amount);

    if new_vertical_start <= max_scrollable_position{
        Ok(DisplayArea::new(view.horizontal_start, new_vertical_start, view.width, view.height))
    }else{
        Ok(DisplayArea::new(view.horizontal_start, max_scrollable_position, view.width, view.height))
    }
}

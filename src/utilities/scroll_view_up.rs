use crate::{
    display_area::{DisplayArea, DisplayAreaError},
};

//pub fn application_impl(app: &mut Application, amount: usize) -> Result<(), ApplicationError>{
//pub fn application_impl(app: &mut Application, display_area: &DisplayArea, amount: usize) -> Result<(), ApplicationError>{
//    match view_impl(display_area, amount){
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

/// Returns a new instance of [`View`] with `vertical_start` decreased by specified amount.
/// # Errors
/// when `amount` is 0.
/// when function would return a `View` with the same state.
pub fn view_impl(view: &DisplayArea, amount: usize) -> Result<DisplayArea, DisplayAreaError>{
    if amount == 0{return Err(DisplayAreaError::InvalidInput);}
    if view.vertical_start == 0{return Err(DisplayAreaError::ResultsInSameState);}
    Ok(DisplayArea::new(view.horizontal_start, view.vertical_start.saturating_sub(amount), view.width, view.height))
}

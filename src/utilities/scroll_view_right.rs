use crate::{
    display_area::{DisplayArea, DisplayAreaError},
};

//pub fn application_impl(app: &mut Application, display_area: &DisplayArea, amount: usize) -> Result<(), ApplicationError>{
//    match view_impl(&display_area, amount, &app.buffer){
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

/// Returns a new instance of [`View`] with `horizontal_start` increased by specified amount.
/// # Errors
/// when `amount` is 0.
/// when function would return a `View` with the same state.
pub fn view_impl(view: &DisplayArea, amount: usize, buffer: &crate::buffer::Buffer) -> Result<DisplayArea, DisplayAreaError>{
    if amount == 0{return Err(DisplayAreaError::InvalidInput);}

    // TODO: cache longest as a field in [`View`] struct to eliminate having to calculate this on each call
    // Calculate the longest line width in a single pass
    let longest = buffer./*inner.*/lines().enumerate()
        .map(|(i, _)| buffer.line_width_chars(i, false))
        .max()
        .unwrap_or(0); // Handle the case where there are no lines

    let new_horizontal_start = view.horizontal_start.saturating_add(amount);

    if new_horizontal_start + view.width <= longest{
        Ok(DisplayArea::new(new_horizontal_start, view.vertical_start, view.width, view.height))
    }else{
        //Ok(self.clone())
        Err(DisplayAreaError::ResultsInSameState)
    }
}

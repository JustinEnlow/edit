use crate::{
    application::{Application, ApplicationError},
    display_area::{DisplayArea, DisplayAreaError},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, display_area: &DisplayArea, amount: usize) -> Result<(), ApplicationError>{
    match view_impl(&display_area, amount, &app.buffer){
        Ok(view) => {
            //app.buffer_display_area = view
            let DisplayArea{horizontal_start, vertical_start, width: _width, height: _height} = view;
            app.buffer_horizontal_start = horizontal_start;
            app.buffer_vertical_start = vertical_start;
        }
        Err(e) => {
            match e{
                DisplayAreaError::InvalidInput => {return Err(ApplicationError::InvalidInput);}
                DisplayAreaError::ResultsInSameState => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
            }
        }
    }

    Ok(())
}

/// Returns a new instance of [`View`] with `horizontal_start` increased by specified amount.
/// # Errors
///     //if `amount` is 0.
///     //if function would return a `View` with the same state.
fn view_impl(view: &DisplayArea, amount: usize, buffer: &crate::buffer::Buffer) -> Result<DisplayArea, DisplayAreaError>{
    if amount == 0{return Err(DisplayAreaError::InvalidInput);}

    // TODO: cache longest as a field in [`View`] struct to eliminate having to calculate this on each call
    // Calculate the longest line width in a single pass
    let longest = buffer.inner.lines().enumerate()
        .map(|(i, _)| buffer.line_width(i, false))
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

#[cfg(test)]
mod tests{
    use crate::utilities::scroll_view_right;
    use crate::{
        application::Application,
        selection::CursorSemantics,
        display_area::DisplayArea,
    };

    fn test(_semantics: CursorSemantics, text: &str, view: DisplayArea, amount: usize, expected_text: &str, expected_view: DisplayArea){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));

        //app.buffer_display_area = view;
        
        let result = scroll_view_right::application_impl(&mut app, &view, amount);
        assert!(!result.is_err());
        
        //assert_eq!(expected_text, app.buffer_display_area.text(&app.buffer));
        //assert_eq!(expected_view, app.buffer_display_area);
        assert!(!app.buffer.is_modified());
    }
    fn test_error(_semantics: CursorSemantics, text: &str, view: DisplayArea, amount: usize){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
        
        //app.buffer_display_area = view;
        
        assert!(scroll_view_right::application_impl(&mut app, &view, amount).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn scroll_right(){
        test(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            DisplayArea::new(0, 0, 2, 2), 1, 
            "dk\nom\n", 
            DisplayArea::new(1, 0, 2, 2), 
        );
    }
    //TODO: test when amount > space left to scroll.    //does this saturate at doc bounds currently?

    #[test] fn errors_if_already_scrolled_right_all_the_way(){
        test_error(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            DisplayArea::new(2, 0, 2, 2), 1, 
        );
    }

    #[test] fn errors_if_amount_is_zero(){
        test_error(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            DisplayArea::new(1, 0, 2, 2), 0, 
        );
    }
}

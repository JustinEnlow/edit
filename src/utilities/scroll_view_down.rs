use crate::{
    application::{Application, ApplicationError},
    view::{DisplayArea, DisplayAreaError},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, display_area: &DisplayArea, amount: usize) -> Result<(), ApplicationError>{
    match view_impl(display_area, amount, &app.buffer){
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

/// Returns a new instance of [`View`] with `vertical_start` increased by specified amount.
/// # Errors
///     //if `amount` is 0.
///     //if function would return a `View` with the same state.
/// # Panics
///     //if `text` is invalid.
fn view_impl(view: &DisplayArea, amount: usize, buffer: &crate::buffer::Buffer) -> Result<DisplayArea, DisplayAreaError>{
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

#[cfg(test)]
mod tests{
    use crate::utilities::scroll_view_down;
    use crate::{
        application::Application,
        selection::CursorSemantics,
        view::DisplayArea,
    };

    fn test(_semantics: CursorSemantics, text: &str, view: DisplayArea, amount: usize, expected_text: &str, expected_view: DisplayArea){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));

        //app.buffer_display_area = view;
        
        let result = scroll_view_down::application_impl(&mut app, &view, amount);
        assert!(!result.is_err());
        
        //assert_eq!(expected_text, app.buffer_display_area.text(&app.buffer));
        //assert_eq!(expected_view, app.buffer_display_area);
        assert!(!app.buffer.is_modified());
    }
    fn test_error(_semantics: CursorSemantics, text: &str, view: DisplayArea, amount: usize){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
        
        //app.buffer_display_area = view;
        
        assert!(scroll_view_down::application_impl(&mut app, &view, amount).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn scroll_down(){
        test(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            DisplayArea::new(0, 0, 2, 2), 1, 
            "so\nsh\n", 
            DisplayArea::new(0, 1, 2, 2), 
        );
        test(
            CursorSemantics::Bar,
            "idk\nsome\nshit\n", 
            DisplayArea::new(0, 0, 2, 2), 1, 
            "so\nsh\n", 
            DisplayArea::new(0, 1, 2, 2), 
        );
    }
    //TODO: test when amount > space left to scroll.    //does this saturate at doc bounds currently?

    #[test] fn errors_if_already_scrolled_down_all_the_way(){
        test_error(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            DisplayArea::new(0, 2, 2, 2), 1, 
        );
        test_error(
            CursorSemantics::Bar,
            "idk\nsome\nshit\n", 
            DisplayArea::new(0, 2, 2, 2), 1, 
        );
    }

    #[test] fn errors_if_amount_is_zero(){
        test_error(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            DisplayArea::new(0, 1, 2, 2), 0, 
        );
        test_error(
            CursorSemantics::Bar,
            "idk\nsome\nshit\n", 
            DisplayArea::new(0, 1, 2, 2), 0, 
        );
    }
}

use crate::{
    application::{Application, ApplicationError},
    display_area::{DisplayArea, DisplayAreaError},
    selections::SelectionsError
};

//pub fn application_impl(app: &mut Application, amount: usize) -> Result<(), ApplicationError>{
pub fn application_impl(app: &mut Application, display_area: &DisplayArea, amount: usize) -> Result<(), ApplicationError>{
    match view_impl(display_area, amount){
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

/// Returns a new instance of [`View`] with `vertical_start` decreased by specified amount.
/// # Errors
///     //if `amount` is 0.
///     //if function would return a `View` with the same state.
fn view_impl(view: &DisplayArea, amount: usize) -> Result<DisplayArea, DisplayAreaError>{
    if amount == 0{return Err(DisplayAreaError::InvalidInput);}
    if view.vertical_start == 0{return Err(DisplayAreaError::ResultsInSameState);}
    Ok(DisplayArea::new(view.horizontal_start, view.vertical_start.saturating_sub(amount), view.width, view.height))
}

#[cfg(test)]
mod tests{
    use crate::utilities::scroll_view_up;
    use crate::{
        application::{Application, ViewAction, Mode},
        selection::CursorSemantics,
        display_area::DisplayArea,
        utilities::test::test_view_action,
        config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
    };

    //fn test(_semantics: CursorSemantics, text: &str, view: DisplayArea, amount: usize, expected_text: &str, expected_view: DisplayArea){
    //    let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
    //
    //    //app.buffer_display_area = view;
    //    
    //    let result = scroll_view_up::application_impl(&mut app, &view, amount);
    //    assert!(!result.is_err());
    //    
    //    //assert_eq!(expected_text, app.buffer_display_area.text(&app.buffer));
    //    //assert_eq!(expected_view, app.buffer_display_area);
    //    assert!(!app.buffer.is_modified());
    //}
    fn test_error(_semantics: CursorSemantics, text: &str, view: DisplayArea, amount: usize){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
        
        //app.buffer_display_area = view;
        
        assert!(scroll_view_up::application_impl(&mut app, &view, amount).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn scroll_up(){
        //test(
        //    CursorSemantics::Block,
        //    "idk\nsome\nshit\n", 
        //    DisplayArea::new(0, 2, 2, 2), 1, 
        //    "so\nsh\n", 
        //    DisplayArea::new(0, 1, 2, 2), 
        //);
        //
        //test(
        //    CursorSemantics::Bar,
        //    "idk\nsome\nshit\n", 
        //    DisplayArea::new(0, 2, 2, 2), 1, 
        //    "so\nsh\n", 
        //    DisplayArea::new(0, 1, 2, 2), 
        //);

        // i d k        |i d|k
        //|s o|m e      |s o|m e
        //|s h|i t       s h i t
        test_view_action(
            ViewAction::ScrollUp, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 1, width: 2, height: 2}, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 
            0, 
            Mode::View, 
            "id\nso\n",
            DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}
        );
    }
    //TODO: test when amount > space left to scroll.    //does this saturate at doc bounds currently?

    #[test] fn errors_if_already_scrolled_up_all_the_way(){
        //test_error(
        //    CursorSemantics::Block,
        //    "idk\nsome\nshit\n", 
        //    DisplayArea::new(0, 0, 2, 2), 1, 
        //);
        //test_error(
        //    CursorSemantics::Bar,
        //    "idk\nsome\nshit\n", 
        //    DisplayArea::new(0, 0, 2, 2), 1, 
        //);
        test_view_action(
            ViewAction::ScrollUp, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 
            0, 
            match SAME_STATE_DISPLAY_MODE{
                DisplayMode::Error => Mode::Error(SAME_STATE.to_string()),
                DisplayMode::Warning => Mode::Warning(SAME_STATE.to_string()),
                DisplayMode::Notify => Mode::Notify(SAME_STATE.to_string()),
                DisplayMode::Info => Mode::Info(SAME_STATE.to_string()),
                DisplayMode::Ignore => Mode::Insert,
            }, 
            "id\nso\n", 
            DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}
        );
    }

    #[test] fn errors_if_amount_is_zero(){  //idk if this can be represented in test_view_action
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

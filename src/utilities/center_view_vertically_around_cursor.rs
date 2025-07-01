use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, CursorSemantics},
    display_area::{DisplayArea, DisplayAreaError},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, display_area: &DisplayArea, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match view_impl(display_area, app.selections.primary(), &app.buffer, semantics){
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

/// Returns an instance of [`View`] vertically centered around specified cursor.
/// # Errors
///     //if function output would return a `View` with the same state.
/// # Panics
///     //if `selection` is invalid.
///     //if `text` is invalid.
fn view_impl(view: &DisplayArea, selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<DisplayArea, DisplayAreaError>{
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

#[cfg(test)]
mod tests{
    use crate::{
        application::{ViewAction, Mode},
        selection::CursorSemantics,
        display_area::DisplayArea,
        utilities::test::test_view_action,
        config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
    };

    #[test] fn works_when_cursor_in_valid_position_before_center(){
        // i d k                                        // i d k
        // y e t                                        //|y e t|
        //|s o m|e      //<-- primary cursor here -->   //|s o m|e
        //|m o r|e                                      //|m o r|e
        //|o t h|e r                                    // o t h e r
        // r a n d o m                                  // r a n d o m
        // s h i t                                      // s h i t
        test_view_action(
            ViewAction::CenterVerticallyAroundCursor, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 2, width: 3, height: 3}, 
            "idk\nyet\nsome\nmore\nother\nrandom\nshit\n", 
            vec![
                (8, 9, None)
            ], 
            0, 
            Mode::Insert, 
            "yet\nsom\nmor\n", 
            DisplayArea::new(0, 1, 3, 3)
        );
        // test with line numbers and status bar displayed...
        //test_view_action(
        //    ViewAction::CenterVerticallyAroundCursor, 
        //    CursorSemantics::Block, 
        //    true, 
        //    true, 
        //    DisplayArea{
        //        horizontal_start: 0, 
        //        vertical_start: 2, 
        //        width: 3 + 1 + (crate::ui::document_viewport::LINE_NUMBER_PADDING as usize),    //buffer display area width + line number display width + line number padding
        //        height: 3 + 2   //buffer display area height + status/util bar
        //    }, 
        //    "idk\nyet\nsome\nmore\nother\nrandom\nshit\n", 
        //    vec![
        //        (8, 9, None)
        //    ], 
        //    0, 
        //    Mode::Insert, 
        //    "yet\nsom\nmor\n", 
        //    DisplayArea::new(0, 1, 3, 3)
        //);
    }
    #[test] fn works_when_cursor_in_valid_position_after_center(){
        // i d k                                        // i d k
        // y e t                                        // y e t
        //|s o m|e                                      // s o m e
        //|m o r|e                                      //|m o r|e
        //|o t h|e r    //<-- primary cursor here -->   //|o t h|e r
        // r a n d o m                                  //|r a n|d o m
        // s h i t                                      // s h i t
        test_view_action(
            ViewAction::CenterVerticallyAroundCursor, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 2, width: 3, height: 3}, 
            "idk\nyet\nsome\nmore\nother\nrandom\nshit\n", 
            vec![
                (18, 19, None)
            ], 
            0, 
            Mode::Insert, 
            "mor\noth\nran\n", 
            DisplayArea::new(0, 3, 3, 3)
        );
    }

    #[test] fn errors_when_cursor_before_half_view_height(){
        //|i d k|       //<-- primary cursor here -->   //|i d k|
        //|s o m|e                                      //|s o m|e
        //|m o r|e                                      //|m o r|e
        // o t h e r                                    // o t h e r
        // s h i t                                      // s h i t
        test_view_action(
            ViewAction::CenterVerticallyAroundCursor, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 0, width: 3, height: 3}, 
            "idk\nsome\nmore\nother\nshit\n", 
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
            "idk\nsom\nmor\n", 
            DisplayArea{horizontal_start: 0, vertical_start: 0, width: 3, height: 3}
        );
    }
    
    #[test] fn errors_when_cursor_after_doc_end_minus_half_view_height(){
        // i d k                                        // i d k
        // s o m e                                      // s o m e
        //|m o r|e                                      //|m o r|e
        //|o t h|e r                                    //|o t h|e r
        //|s h i|t      //<-- primary cursor here -->   //|s h i|t
        test_view_action(
            ViewAction::CenterVerticallyAroundCursor, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 2, width: 3, height: 3}, 
            "idk\nsome\nmore\nother\nshit\n", 
            vec![
                (25, 26, None)
            ], 
            0, 
            match SAME_STATE_DISPLAY_MODE{
                DisplayMode::Error => Mode::Error(SAME_STATE.to_string()),
                DisplayMode::Warning => Mode::Warning(SAME_STATE.to_string()),
                DisplayMode::Notify => Mode::Notify(SAME_STATE.to_string()),
                DisplayMode::Info => Mode::Info(SAME_STATE.to_string()),
                DisplayMode::Ignore => Mode::Insert,
            },
            "mor\noth\nshi\n", 
            DisplayArea{horizontal_start: 0, vertical_start: 2, width: 3, height: 3}
        );
    }
    
    #[test] fn errors_when_cursor_already_centered_with_odd_num_lines(){
        // i d k                                        // i d k
        //|s o m|e                                      //|s o m|e
        //|m o r|e      //<-- primary cursor here -->   //|m o r|e
        //|o t h|e r                                    //|o t h|e r
        // s h i t                                      // s h i t
        test_view_action(
            ViewAction::CenterVerticallyAroundCursor, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 1, width: 3, height: 3}, 
            "idk\nsome\nmore\nother\nshit\n", 
            vec![
                (9, 10, None)
            ], 
            0, 
            match SAME_STATE_DISPLAY_MODE{
                DisplayMode::Error => Mode::Error(SAME_STATE.to_string()),
                DisplayMode::Warning => Mode::Warning(SAME_STATE.to_string()),
                DisplayMode::Notify => Mode::Notify(SAME_STATE.to_string()),
                DisplayMode::Info => Mode::Info(SAME_STATE.to_string()),
                DisplayMode::Ignore => Mode::Insert,
            },
            "som\nmor\noth\n", 
            DisplayArea{horizontal_start: 0, vertical_start: 1, width: 3, height: 3}
        );
    }
    #[test] fn errors_when_cursor_on_first_middle_line_with_even_num_lines(){
        // i d k                                        // i d k
        //|y e t|                                       //|y e t|
        //|s o m|e      //<-- primary cursor here -->   //|s o m|e
        //|m o r|e                                      //|m o r|e
        //|o t h|e r                                    //|o t h|e r
        // s h i t                                      // s h i t
        test_view_action(
            ViewAction::CenterVerticallyAroundCursor, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 1, width: 3, height: 4}, 
            "idk\nyet\nsome\nmore\nother\nshit\n", 
            vec![
                (8, 9, None)
            ], 
            0, 
            match SAME_STATE_DISPLAY_MODE{
                DisplayMode::Error => Mode::Error(SAME_STATE.to_string()),
                DisplayMode::Warning => Mode::Warning(SAME_STATE.to_string()),
                DisplayMode::Notify => Mode::Notify(SAME_STATE.to_string()),
                DisplayMode::Info => Mode::Info(SAME_STATE.to_string()),
                DisplayMode::Ignore => Mode::Insert,
            },
            "yet\nsom\nmor\noth\n", 
            DisplayArea{horizontal_start: 0, vertical_start: 1, width: 3, height: 4}
        );
    }
    #[test] fn errors_when_cursor_on_other_middle_line_with_even_num_lines(){
        // i d k                                        // i d k
        //|y e t|                                       //|y e t|
        //|s o m|e                                      //|s o m|e
        //|m o r|e      //<-- primary cursor here -->   //|m o r|e
        //|o t h|e r                                    //|o t h|e r
        // s h i t                                      // s h i t
        test_view_action(
            ViewAction::CenterVerticallyAroundCursor, 
            CursorSemantics::Block, 
            false, 
            false, 
            DisplayArea{horizontal_start: 0, vertical_start: 1, width: 3, height: 4}, 
            "idk\nyet\nsome\nmore\nother\nshit\n", 
            vec![
                (13, 14, None)
            ], 
            0, 
            match SAME_STATE_DISPLAY_MODE{
                DisplayMode::Error => Mode::Error(SAME_STATE.to_string()),
                DisplayMode::Warning => Mode::Warning(SAME_STATE.to_string()),
                DisplayMode::Notify => Mode::Notify(SAME_STATE.to_string()),
                DisplayMode::Info => Mode::Info(SAME_STATE.to_string()),
                DisplayMode::Ignore => Mode::Insert,
            },
            "yet\nsom\nmor\noth\n", 
            DisplayArea{horizontal_start: 0, vertical_start: 1, width: 3, height: 4}
        );
    }
}

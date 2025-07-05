use edit::{
    application::{ViewAction, Mode},
    selection::CursorSemantics,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
};
use crate::view_actions::test_view_action;


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
        Mode::View,
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
        Mode::View,
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
        Mode::View,
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
        Mode::View,
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
        Mode::View,
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
        Mode::View,
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
        Mode::View,
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

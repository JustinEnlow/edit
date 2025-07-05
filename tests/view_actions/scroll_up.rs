use edit::{
    application::{ViewAction, Mode},
    selection::CursorSemantics,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
};
use crate::view_actions::test_view_action;



#[test] fn stays_in_view_mode_when_successful_scroll_from_view_mode(){
    // i d k        |i d|k
    //|s o|m e      |s o|m e
    //|s h|i t       s h i t
    test_view_action(
        ViewAction::ScrollUp, 
        CursorSemantics::Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 1, width: 2, height: 2}, 
        Mode::View,
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

#[test] fn stays_in_insert_mode_when_successful_scroll_from_insert_mode(){
    // i d k        |i d|k
    //|s o|m e      |s o|m e
    //|s h|i t       s h i t
    test_view_action(
        ViewAction::ScrollUp, 
        CursorSemantics::Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 1, width: 2, height: 2}, 
        Mode::Insert,
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None)
        ], 
        0, 
        Mode::Insert, 
        "id\nso\n", 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}
    );
}

#[test] fn enters_correct_mode_when_unsuccessful_scroll_from_view_mode(){//scoll_view_up_errors_if_already_scrolled_up_all_the_way(){
    //|i d|k
    //|s o|m e
    // s h i t
    test_view_action(
        ViewAction::ScrollUp, 
        CursorSemantics::Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}, 
        Mode::View,
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

#[should_panic] #[test] fn should_panic_when_called_from_any_mode_but_insert_or_view(){
    test_view_action(
        ViewAction::ScrollUp, 
        CursorSemantics::Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}, 
        Mode::Command,
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

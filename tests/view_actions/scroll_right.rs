use edit::{
    application::{ViewAction::ScrollRight, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::view_actions::test_view_action;

#[test] fn stays_in_view_mode_when_successful_scroll_from_view_mode(){
    //|i d|k         i|d k|
    //|s o|m e       s|o m|e
    // s h i t       s h i t
    test_view_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        ScrollRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}, 
        Mode::View,
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None)
        ], 
        0, 
        Mode::View, 
        "dk\nom\n",
        DisplayArea{horizontal_start: 1, vertical_start: 0, width: 2, height: 2}
    );
}

#[test] fn stays_in_insert_mode_when_successful_scroll_from_insert_mode(){
    //|i d|k         i|d k|
    //|s o|m e       s|o m|e
    // s h i t       s h i t
    test_view_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        ScrollRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}, 
        Mode::Insert,
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None)
        ], 
        0, 
        Mode::Insert, 
        "dk\nom\n",
        DisplayArea{horizontal_start: 1, vertical_start: 0, width: 2, height: 2}
    );
}
//TODO: test when amount > space left to scroll.    //does this saturate at doc bounds currently?

#[test] fn errors_if_already_scrolled_right_all_the_way(){
    // i d|k  |
    // s o|m e|
    // s h i t
    test_view_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        ScrollRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 2, vertical_start: 0, width: 2, height: 2}, 
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
        "k\nme\n", 
        DisplayArea{horizontal_start: 2, vertical_start: 0, width: 2, height: 2}
    );
}

#[should_panic] #[test] fn should_panic_when_called_from_any_mode_but_insert_or_view(){
    test_view_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        ScrollRight, 
        //Block, 
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

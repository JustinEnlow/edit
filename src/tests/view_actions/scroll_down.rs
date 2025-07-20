use crate::{
    application::ViewAction::ScrollDown,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::tests::view_actions::test_view_action;

#[test] fn stays_in_view_mode_when_successful_scroll_from_view_mode(){
    //|i d|k         i d k
    //|s o|m e      |s o|m e
    // s h i t      |s h|i t
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
        ScrollDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}, 
        Mode::View,
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
        ], 
        0, 
        Mode::View, 
        "so\nsh\n",
        DisplayArea{horizontal_start: 0, vertical_start: 1, width: 2, height: 2}
    );
}

#[test] fn stays_in_insert_mode_when_successful_scroll_from_insert_mode(){
    //|i d|k         i d k
    //|s o|m e      |s o|m e
    // s h i t      |s h|i t
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
        ScrollDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}, 
        Mode::Insert,
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
        ], 
        0, 
        Mode::Insert, 
        "so\nsh\n", 
        DisplayArea{horizontal_start: 0, vertical_start: 1, width: 2, height: 2}
    );
}
//TODO: test when amount > space left to scroll.    //does this saturate at doc bounds currently?

#[test] fn errors_if_already_scrolled_down_all_the_way(){   //TODO: do another with display_area(0, 1, 2, 2). this is the more realistic situation...
    // i d k
    // s o m e
    //|s h|i t
    //|   |
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
        ScrollDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 2, width: 2, height: 2}, 
        Mode::View, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
        ], 
        0, 
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => Mode::Error(SAME_STATE.to_string()),
            DisplayMode::Warning => Mode::Warning(SAME_STATE.to_string()),
            DisplayMode::Notify => Mode::Notify(SAME_STATE.to_string()),
            DisplayMode::Info => Mode::Info(SAME_STATE.to_string()),
            DisplayMode::Ignore => Mode::Insert,
        }, 
        "sh\n\n", 
        DisplayArea{horizontal_start: 0, vertical_start: 2, width: 2, height: 2}
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
        ScrollDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 2, height: 2}, 
        Mode::Command,
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
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

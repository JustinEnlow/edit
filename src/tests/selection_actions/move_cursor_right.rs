use crate::{
    application::SelectionAction::MoveCursorRight,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn tab_with_hard_tab_true(){
    test_selection_action(
        Config{
            semantics: Block,
            use_full_file_path: false,
            use_hard_tab: true, //what about when file has existing hard tab, but use_hard_tab is false?...
            tab_width: 4,
            view_scroll_amount: 1,
            show_cursor_column: false,
            show_cursor_line: false
        }, 
        MoveCursorRight, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "\tidk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(Range::new(0, 1), None, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(Range::new(1, 2), None, Some(1))
        ], 
        0
    );
}
#[test] fn tab_with_hard_tab_false(){
    test_selection_action(
        Config{
            semantics: Block,
            use_full_file_path: false,
            use_hard_tab: false, 
            tab_width: 4,
            view_scroll_amount: 1,
            show_cursor_column: false,
            show_cursor_line: false
        }, 
        MoveCursorRight, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "\tidk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(Range::new(0, 1), None, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(Range::new(1, 2), None, Some(1))
        ], 
        0
    );
}

#[test] fn with_multiple_valid_selections_block_semantics(){
    test_selection_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        MoveCursorRight, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(Range::new(0, 1), None, None), //common use
            Selection::new_unchecked(Range::new(8, 9), None, None), //line to line updates stored line position
            Selection::new_unchecked(Range::new(10, 13), Some(Direction::Forward), None),   //extended selection collapses then moves normally
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(Range::new(1, 2), None, Some(1)),
            Selection::new_unchecked(Range::new(9, 10), None, Some(0)),
            Selection::new_unchecked(Range::new(13, 14), None, Some(4)),
        ], 
        0
    );
}
    
#[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
    test_selection_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        MoveCursorRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),   //valid
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(14, 15, None)  //invalid
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(1, 2, Some(1)),
            Selection::new_unchecked(Range::new(1, 2), None, Some(1)),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0
    );
}
    
#[test] fn with_single_selection_at_doc_end_block_semantics(){
    test_selection_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        MoveCursorRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0, 
        1, 
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
            DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
            DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
            DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0
    );
}

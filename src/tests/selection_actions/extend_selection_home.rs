use crate::{
    action::SelectionAction::ExtendSelectionHome,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn when_cursor_past_line_text_start_block_semantics(){
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
        ExtendSelectionHome, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    idk\n    something\n", 
        vec![
            //(6, 7, None),
            Selection::new_unchecked(Range::new(6, 7), None, None),
            //(16, 17, None)
            Selection::new_unchecked(Range::new(16, 17), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(7, 4, Some(4)),
            Selection::new_unchecked(Range::new(4, 7), Some(Direction::Backward), Some(4)),
            //(17, 12, Some(4))
            Selection::new_unchecked(Range::new(12, 17), Some(Direction::Backward), Some(4)),
        ], 
        0
    );
}
#[test] fn when_cursor_at_line_text_start_block_semantics(){
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
        ExtendSelectionHome, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    idk\n    something\n", 
        vec![
            //(4, 5, None),
            Selection::new_unchecked(Range::new(4, 5), None, None),
            //(12, 13, None)
            Selection::new_unchecked(Range::new(12, 13), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(5, 0, Some(0)),
            Selection::new_unchecked(Range::new(0, 5), Some(Direction::Backward), Some(0)),
            //(13, 8, Some(0))
            Selection::new_unchecked(Range::new(8, 13), Some(Direction::Backward), Some(0)),
        ], 
        0
    );
}
#[test] fn when_cursor_before_line_text_start_block_semantics(){
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
        ExtendSelectionHome, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    idk\n    something\n", 
        vec![
            //(2, 3, None),
            Selection::new_unchecked(Range::new(2, 3), None, None),
            //(10, 11, None)
            Selection::new_unchecked(Range::new(10, 11), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(2, 5, Some(4)),
            Selection::new_unchecked(Range::new(2, 5), Some(Direction::Forward), Some(4)),
            //(10, 13, Some(4))
            Selection::new_unchecked(Range::new(10, 13), Some(Direction::Forward), Some(4)),
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
        ExtendSelectionHome, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsomething\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(6, 7, None)
            Selection::new_unchecked(Range::new(6, 7), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(7, 4, Some(0))
            Selection::new_unchecked(Range::new(4, 7), Some(Direction::Backward), Some(0)),
        ], 
        0
    );
}

#[test] fn errors_when_line_start_and_line_text_start_and_cursor_position_all_equal_block_semantics(){
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
        ExtendSelectionHome, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsomething\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
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
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
        ], 
        0
    );
}

use crate::{
    application::{SelectionAction::SelectLine, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, SPANS_MULTIPLE_LINES_DISPLAY_MODE, SPANS_MULTIPLE_LINES, Config}
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn normal_use_block_semantics(){
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
        SelectLine, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 4, None),
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward), None),
            //(4, 9, None)
            Selection::new_unchecked(Range::new(4, 9), Some(Direction::Forward), None),
        ], 
        0
    );
}
#[test] fn should_succeed_if_mixed_selection_spanning_multiple_lines_and_valid_selection(){
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
        SelectLine, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(4, 12, None)
            Selection::new_unchecked(Range::new(4, 12), Some(Direction::Forward), None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 4, None),
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward), None),
            //(4, 12, None)
            Selection::new_unchecked(Range::new(4, 12), Some(Direction::Forward), None),
        ], 
        0
    );
}

#[test] fn errors_if_selection_spans_multiple_lines_block_semantics(){
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
        SelectLine, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(4, 12, None)
            Selection::new_unchecked(Range::new(4, 12), Some(Direction::Forward), None),
        ], 
        0, 
        1, 
        match SPANS_MULTIPLE_LINES_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(SPANS_MULTIPLE_LINES.to_string())},
            DisplayMode::Warning => {Mode::Warning(SPANS_MULTIPLE_LINES.to_string())},
            DisplayMode::Notify => {Mode::Notify(SPANS_MULTIPLE_LINES.to_string())},
            DisplayMode::Info => {Mode::Info(SPANS_MULTIPLE_LINES.to_string())},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            //(4, 12, None)
            Selection::new_unchecked(Range::new(4, 12), Some(Direction::Forward), None),
        ], 
        0
    );
}

//TODO: have test with mixed new state and same state selections. should succeed...
#[test] fn errors_if_results_in_same_state_block_semantics(){
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
        SelectLine, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 4, None)
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward), None),
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
            //(0, 4, None)
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward), None),
        ], 
        0
    );
}

//TODO: have test with mixed valid selection and selection at doc end and line empty. should succeed...
#[test] fn errors_if_at_doc_end_and_line_empty_block_semantics(){
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
        SelectLine, 
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

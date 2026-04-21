use crate::{
    action::SelectionAction::SelectLine,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SPANS_MULTIPLE_LINES_DISPLAY_MODE, Config},
    keybind::default_keybinds
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SelectLine, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
            //|i>d k ⏎|s>o m e ⏎ s h i t ⏎
            //0 1 2 3 4 5
            //|i>d k ⏎
            //|s>o m e ⏎
            // s h i t ⏎
            //

            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //0 1 2 3 4  5 6 7 8 9 0 1 2 3 4
            //|i d k:⏎>|s o m e:⏎>s h i t ⏎
            //0 1 2 3 4 5
            //|i d k:⏎>
            //|s o m e:⏎>
            // s h i t ⏎
            //

            Selection::new_unchecked(0..4, Some(Direction::Forward), 3),
            Selection::new_unchecked(4..9, Some(Direction::Forward), 4),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SelectLine, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..12, Some(Direction::Forward), 2),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..4, Some(Direction::Forward), 3),
            Selection::new_unchecked(4..12, Some(Direction::Forward), 2),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SelectLine, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(4..12, Some(Direction::Forward), 2),
        ], 
        0, 
        1, 
        match SPANS_MULTIPLE_LINES_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error},
            DisplayMode::Warning => {Mode::Warning},
            DisplayMode::Notify => {Mode::Notify},
            DisplayMode::Info => {Mode::Info},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            Selection::new_unchecked(4..12, Some(Direction::Forward), 2),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SelectLine, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..4, Some(Direction::Forward), 3),
        ], 
        0, 
        1, 
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error},
            DisplayMode::Warning => {Mode::Warning},
            DisplayMode::Notify => {Mode::Notify},
            DisplayMode::Info => {Mode::Info},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            Selection::new_unchecked(0..4, Some(Direction::Forward), 3),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SelectLine, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(14..15, None, 0),
        ], 
        0, 
        1, 
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error},
            DisplayMode::Warning => {Mode::Warning},
            DisplayMode::Notify => {Mode::Notify},
            DisplayMode::Info => {Mode::Info},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            Selection::new_unchecked(14..15, None, 0),
        ], 
        0
    );
}

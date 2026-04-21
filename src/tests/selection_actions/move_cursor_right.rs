use crate::{
    action::SelectionAction::MoveCursorRight,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, Config},
    keybind::default_keybinds
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        }, 
        MoveCursorRight, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "\tidk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(1..2, None, 1)
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        }, 
        MoveCursorRight, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "\tidk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(1..2, None, 1)
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        MoveCursorRight, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0), //common use
            Selection::new_unchecked(8..9, None, 4), //line to line updates stored line position
            Selection::new_unchecked(10..13, Some(Direction::Forward), 3),   //extended selection collapses then moves normally
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(1..2, None, 1),
            Selection::new_unchecked(9..10, None, 0),
            Selection::new_unchecked(13..14, None, 4),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        MoveCursorRight, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //valid
            Selection::new_unchecked(0..1, None, 0),
            //invalid
            Selection::new_unchecked(14..15, None, 0),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(1..2, None, 1),
            Selection::new_unchecked(14..15, None, 0),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        MoveCursorRight, 
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

use crate::{
    action::SelectionAction::MoveCursorLeft,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, Config},
    keybind::default_keybinds
};
use crate::tests::selection_actions::test_selection_action;



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
        MoveCursorLeft, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //common use
            Selection::new_unchecked(1..2, None, 1),
            //line to line updates stored line position
            Selection::new_unchecked(4..5, None, 0),
            //extended selection collapses to cursor then does regular move
            Selection::new_unchecked(10..13, Some(Direction::Forward), 3),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(3..4, None, 3),
            Selection::new_unchecked(11..12, None, 2),
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
        MoveCursorLeft, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //invalid
            Selection::new_unchecked(0..1, None, 0),
            //valid
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(3..4, None, 3),
        ], 
        0
    );
}
    
#[test] fn errors_if_single_selection_at_doc_start_block_semantics(){
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
        MoveCursorLeft, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
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
            Selection::new_unchecked(0..1, None, 0),
        ], 
        0
    );
}

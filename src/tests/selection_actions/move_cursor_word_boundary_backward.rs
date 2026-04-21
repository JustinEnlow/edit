use crate::{
    action::SelectionAction::MoveCursorWordBoundaryBackward,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, Config},
    keybind::default_keybinds
};
use crate::tests::selection_actions::test_selection_action;



#[ignore] #[test] fn implement_tests_using_count(){
    todo!()
}

#[test] fn with_multiple_valid_selections_block_semantics(){
    //                    1                   2
    //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    // _ _ _ _ u s e _ e r r o r : : E r r o r ;
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
        MoveCursorWordBoundaryBackward, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    use error::Error;",    //len 21    text end: (20, 21)  doc end: (21, 22), 
        vec![
            //skips whitespace and moves to doc start if no other alphanumeric
            Selection::new_unchecked(4..5, None, 4),
            //skips whitespace and moves to next starting word boundary
            Selection::new_unchecked(8..9, None, 8),
            //non alpha_numeric or whitespace jumps to previous non whitespace
            Selection::new_unchecked(14..15, None, 14),
            //extended collapses then moves normally
            Selection::new_unchecked(15..20, Some(Direction::Backward), 15),
            //common use
            Selection::new_unchecked(21..22, None, 21),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 4),
            Selection::new_unchecked(13..14, None, 13),
            Selection::new_unchecked(14..15, None, 14),
            Selection::new_unchecked(20..21, None, 20),
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
        MoveCursorWordBoundaryBackward, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n",
        vec![
            //invalid
            Selection::new_unchecked(0..1, None, 0),
            //valid + line to line updates stored line position
            Selection::new_unchecked(9..10, None, 0),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0
    );
}

#[test] fn errors_when_single_selection_at_doc_end_block_semantics(){
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
        MoveCursorWordBoundaryBackward, 
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

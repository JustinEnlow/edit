use crate::{
    action::SelectionAction::MoveCursorWordBoundaryForward,
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
    // u s e _ e r r o r : : E r r o r ; _ _ _ _
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
        MoveCursorWordBoundaryForward, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "use error::Error;    ",    //len 21    text end: (20, 21)    doc end: (21, 22)
        vec![
            //common use
            Selection::new_unchecked(0..1, None, 0),
            //skips whitespace and moves to next ending word boundary
            Selection::new_unchecked(2..3, None, 2),
            //non alpha_numeric or whitespace jumps to next non whitespace
            Selection::new_unchecked(8..9, None, 8),
            //extended collapses then moves normally
            Selection::new_unchecked(11..16, Some(Direction::Forward), 15),
            //skips whitespace and moves to doc end if no other alphanumeric
            Selection::new_unchecked(16..17, None, 16),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(2..3, None, 2),
            Selection::new_unchecked(8..9, None, 8),
            Selection::new_unchecked(9..10, None, 9),
            Selection::new_unchecked(16..17, None, 16),
            Selection::new_unchecked(21..22, None, 21),
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
        MoveCursorWordBoundaryForward, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //valid + line to line updates stored line position
            Selection::new_unchecked(3..4, None, 3),
            //invalid
            Selection::new_unchecked(14..15, None, 0),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(7..8, None, 3),
            Selection::new_unchecked(14..15, None, 0),
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
        MoveCursorWordBoundaryForward, 
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

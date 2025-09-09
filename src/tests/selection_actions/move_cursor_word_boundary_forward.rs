use crate::{
    action::SelectionAction::MoveCursorWordBoundaryForward,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, /*SAME_STATE, */Config},
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
            user_options: std::collections::HashMap::new(),
            user_commands: std::collections::HashMap::new(),
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
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "use error::Error;    ",    //len 21    text end: (20, 21)    doc end: (21, 22)
        vec![
            //(0, 1, None),   //common use
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(2, 3, None),   //skips whitespace and moves to next ending word boundary
            Selection::new_unchecked(Range::new(2, 3), None, None),
            //(8, 9, None),   //non alpha_numeric or whitespace jumps to next non whitespace
            Selection::new_unchecked(Range::new(8, 9), None, None),
            //(11, 16, None), //extended collapses then moves normally
            Selection::new_unchecked(Range::new(11, 16), Some(Direction::Forward), None),
            //(16, 17, None)  //skips whitespace and moves to doc end if no other alphanumeric
            Selection::new_unchecked(Range::new(16, 17), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(2, 3, Some(2)),
            Selection::new_unchecked(Range::new(2, 3), None, Some(2)),
            //(8, 9, Some(8)),
            Selection::new_unchecked(Range::new(8, 9), None, Some(8)),
            //(9, 10, Some(9)),
            Selection::new_unchecked(Range::new(9, 10), None, Some(9)),
            //(16, 17, Some(16)),
            Selection::new_unchecked(Range::new(16, 17), None, Some(16)),
            //(21, 22, Some(21))
            Selection::new_unchecked(Range::new(21, 22), None, Some(21)),
        ], 
        0
    );
}

#[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
    test_selection_action(
        Config{
            user_options: std::collections::HashMap::new(),
            user_commands: std::collections::HashMap::new(),
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
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(3, 4, None),   //valid + line to line updates stored line position
            Selection::new_unchecked(Range::new(3, 4), None, None),
            //(14, 15, None)  //invalid
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(7, 8, Some(3)),
            Selection::new_unchecked(Range::new(7, 8), None, Some(3)),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0
    );
}

#[test] fn errors_when_single_selection_at_doc_end_block_semantics(){
    test_selection_action(
        Config{
            user_options: std::collections::HashMap::new(),
            user_commands: std::collections::HashMap::new(),
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
            DisplayMode::Error => {Mode::Error/*(SAME_STATE.to_string())*/},
            DisplayMode::Warning => {Mode::Warning/*(SAME_STATE.to_string())*/},
            DisplayMode::Notify => {Mode::Notify/*(SAME_STATE.to_string())*/},
            DisplayMode::Info => {Mode::Info/*(SAME_STATE.to_string())*/},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0
    );
}

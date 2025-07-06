use edit::{
    application::{SelectionAction::MoveCursorWordBoundaryForward, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::selection_actions::test_selection_action;



#[ignore] #[test] fn implement_tests_using_count(){
    unimplemented!()
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
            show_cursor_line: false
        },
        MoveCursorWordBoundaryForward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "use error::Error;    ",    //len 21    text end: (20, 21)    doc end: (21, 22)
        vec![
            (0, 1, None),   //common use
            (2, 3, None),   //skips whitespace and moves to next ending word boundary
            (8, 9, None),   //non alpha_numeric or whitespace jumps to next non whitespace
            (11, 16, None), //extended collapses then moves normally
            (16, 17, None)  //skips whitespace and moves to doc end if no other alphanumeric
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (2, 3, Some(2)),
            (8, 9, Some(8)),
            (9, 10, Some(9)),
            (16, 17, Some(16)),
            (21, 22, Some(21))
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
        MoveCursorWordBoundaryForward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (3, 4, None),   //valid + line to line updates stored line position
            (14, 15, None)  //invalid
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (7, 8, Some(3)),
            (14, 15, None)
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
            show_cursor_line: false
        },
        MoveCursorWordBoundaryForward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (14, 15, None)
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
            (14, 15, None)
        ], 
        0
    );
}

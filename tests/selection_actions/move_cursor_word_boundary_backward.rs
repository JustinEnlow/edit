use edit::{
    application::{SelectionAction::MoveCursorWordBoundaryBackward, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, ExtensionDirection},
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
    // _ _ _ _ u s e _ e r r o r : : E r r o r ;
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
        MoveCursorWordBoundaryBackward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    use error::Error;",    //len 21    text end: (20, 21)  doc end: (21, 22), 
        vec![
            //(4, 5, None),   //skips whitespace and moves to doc start if no other alphanumeric
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, None),
            //(8, 9, None),   //skips whitespace and moves to next starting word boundary
            Selection::new_unchecked(Range::new(8, 9), ExtensionDirection::None, None),
            //(14, 15, None), //non alpha_numeric or whitespace jumps to previous non whitespace
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
            //(20, 15, None), //extended collapses then moves normally
            Selection::new_unchecked(Range::new(15, 20), ExtensionDirection::Backward, None),
            //(21, 22, None)  //common use
            Selection::new_unchecked(Range::new(21, 22), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, Some(0)),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, Some(0)),
            //(4, 5, Some(4)),
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, Some(4)),
            //(13, 14, Some(13)),
            Selection::new_unchecked(Range::new(13, 14), ExtensionDirection::None, Some(13)),
            //(14, 15, Some(14)),
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, Some(14)),
            //(20, 21, Some(20))
            Selection::new_unchecked(Range::new(20, 21), ExtensionDirection::None, Some(20)),
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
        MoveCursorWordBoundaryBackward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n",
        vec![
            //(0, 1, None),   //invalid
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(9, 10, None)   //valid + line to line updates stored line position
            Selection::new_unchecked(Range::new(9, 10), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(4, 5, Some(0))
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, Some(0)),
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
        MoveCursorWordBoundaryBackward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n",
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
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
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
        ], 
        0
    );
}

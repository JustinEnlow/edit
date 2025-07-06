use edit::{
    application::{SelectionAction::MoveCursorUp, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::selection_actions::test_selection_action;



#[test] fn to_shorter_line_block_semantics(){
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
        MoveCursorUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshits\n", 
        vec![
            (8, 9, None),
            (14, 15, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (3, 4, Some(4)),    //notice this maintains stored line position of selection before operation
            (8, 9, Some(5))     //notice this maintains stored line position of selection before operation
        ], 
        0
    );
}
    
#[test] fn to_line_with_equal_len_or_more_block_semantics(){
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
        MoveCursorUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idfk\nsome\nshit\n", 
        vec![
            (9, 10, None),
            (14, 15, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (4, 5, Some(4)),
            (9, 10, Some(4))
        ], 
        0
    );
}

//with mixed valid and invalid selections   //one on top line, one not
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
        MoveCursorUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),
            (9, 10, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, None),
            (4, 5, Some(0))
        ], 
        0
    );
}

//merges overlapping resultant selections   //one on top line, one on second
#[test] fn merges_overlapping_resultant_selections_block_semantics(){
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
        MoveCursorUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),
            (4, 5, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, Some(0))
        ], 
        0
    );
}
    
    //with extended selections collapses
#[test] fn with_extended_selection_collapses_block_semantics(){
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
        MoveCursorUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (4, 9, None),
            (9, 14, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (3, 4, Some(4)),
            (8, 9, Some(4))
        ], 
        0
    );
}
    
#[test] fn errors_if_single_selection_on_topmost_line_block_semantics(){
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
        MoveCursorUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None)
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
            (0, 1, None)
        ], 
        0
    );
}

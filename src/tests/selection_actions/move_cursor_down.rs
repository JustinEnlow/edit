use crate::{
    action::SelectionAction::MoveCursorDown,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, /*SAME_STATE, */Config},
    keybind::default_keybinds
};
use crate::tests::selection_actions::test_selection_action;



//to shorter line
#[test] fn to_shorter_line_block_semantics(){
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
        MoveCursorDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "shits\nsome\nidk", 
        vec![
            //(5, 6, None),
            Selection::new_unchecked(Range::new(5, 6), None, None),
            //(10, 11, None)
            Selection::new_unchecked(Range::new(10, 11), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(10, 11, Some(5)),  //notice this maintains stored line position of selection before operation
            Selection::new_unchecked(Range::new(10, 11), None, Some(5)),
            //(14, 15, Some(4))   //notice this maintains stored line position of selection before operation
            Selection::new_unchecked(Range::new(14, 15), None, Some(4)),
        ], 
        0
    );
}

//to line with equal len or more
#[test] fn to_line_with_equal_len_or_more_block_semantics(){
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
        MoveCursorDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\nidfk\n", 
        vec![
            //(4, 5, None),
            Selection::new_unchecked(Range::new(4, 5), None, None),
            //(9, 10, None)
            Selection::new_unchecked(Range::new(9, 10), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(9, 10, Some(4)),
            Selection::new_unchecked(Range::new(9, 10), None, Some(4)),
            //(14, 15, Some(4))
            Selection::new_unchecked(Range::new(14, 15), None, Some(4)),
        ], 
        0
    );
}
    
//with mixed valid and invalid selections   //one on bottom line, one not
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
        MoveCursorDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(4, 5, None),
            Selection::new_unchecked(Range::new(4, 5), None, None),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(9, 10, Some(0)),
            Selection::new_unchecked(Range::new(9, 10), None, Some(0)),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0
    );
}
    
//merges overlapping resultant selections   //one on bottom line, one on second
#[test] fn merges_overlapping_resultant_selections_block_semantics(){
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
        MoveCursorDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(9, 10, None),
            Selection::new_unchecked(Range::new(9, 10), None, None),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(14, 15, Some(0))
            Selection::new_unchecked(Range::new(14, 15), None, Some(0)),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        MoveCursorDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 4, None),
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward), None),
            //(4, 9, None)
            Selection::new_unchecked(Range::new(4, 9), Some(Direction::Forward), None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(7, 8, Some(3)),
            Selection::new_unchecked(Range::new(7, 8), None, Some(3)),
            //(13, 14, Some(4))
            Selection::new_unchecked(Range::new(13, 14), None, Some(4)),
        ], 
        0
    );
}
    
//errors if single selection on bottom-most line
#[test] fn errors_if_single_selection_on_bottommost_line_block_semantics(){
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
        MoveCursorDown, 
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

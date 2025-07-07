use edit::{
    application::{SelectionAction::MoveCursorLeft, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, ExtensionDirection},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::selection_actions::test_selection_action;



#[test] fn with_multiple_valid_selections_block_semantics(){
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
        MoveCursorLeft, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(1, 2, None),   //common use
            Selection::new_unchecked(Range::new(1, 2), ExtensionDirection::None, None),
            //(4, 5, None),   //line to line updates stored line position
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, None),
            //(10, 13, None)  //extended selection collapses to cursor then does regular move
            Selection::new_unchecked(Range::new(10, 13), ExtensionDirection::Forward, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, Some(0)),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, Some(0)),
            //(3, 4, Some(3)),
            Selection::new_unchecked(Range::new(3, 4), ExtensionDirection::None, Some(3)),
            //(11, 12, Some(2))
            Selection::new_unchecked(Range::new(11, 12), ExtensionDirection::None, Some(2)),
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
        MoveCursorLeft, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),   //invalid
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(4, 5, None)    //valid
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(3, 4, Some(3))
            Selection::new_unchecked(Range::new(3, 4), ExtensionDirection::None, Some(3)),
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
            show_cursor_line: false
        },
        MoveCursorLeft, 
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
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
        ], 
        0
    );
}

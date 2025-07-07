use edit::{
    application::{SelectionAction::ExtendSelectionLineEnd, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, ExtensionDirection},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::selection_actions::test_selection_action;



#[test] fn moves_to_line_text_end_block_semantics(){
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
        ExtendSelectionLineEnd, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 3, Some(2)),
            Selection::new_unchecked(Range::new(0, 3), ExtensionDirection::Forward, Some(2)),
            //(4, 8, Some(3))
            Selection::new_unchecked(Range::new(4, 8), ExtensionDirection::Forward, Some(3)),
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
        ExtendSelectionLineEnd, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(2, 3, None),
            Selection::new_unchecked(Range::new(2, 3), ExtensionDirection::None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(2, 3, None),
            Selection::new_unchecked(Range::new(2, 3), ExtensionDirection::None, None),
            //(4, 8, Some(3))
            Selection::new_unchecked(Range::new(4, 8), ExtensionDirection::Forward, Some(3)),
        ], 
        0
    );
}

#[test] fn errors_if_already_at_line_text_end_block_semantics(){
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
        ExtendSelectionLineEnd, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(2, 3, None),
            Selection::new_unchecked(Range::new(2, 3), ExtensionDirection::None, None),
            //(7, 8, None)
            Selection::new_unchecked(Range::new(7, 8), ExtensionDirection::None, None),
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
            //(2, 3, None),
            Selection::new_unchecked(Range::new(2, 3), ExtensionDirection::None, None),
            //(7, 8, None)
            Selection::new_unchecked(Range::new(7, 8), ExtensionDirection::None, None),
        ], 
        0
    );
}

//Only applies to block cursor semantics
#[test] fn error_if_already_at_line_end(){  //with cursor over newline char
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
        ExtendSelectionLineEnd, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(3, 4, None),
            Selection::new_unchecked(Range::new(3, 4), ExtensionDirection::None, None),
            //(8, 9, None)
            Selection::new_unchecked(Range::new(8, 9), ExtensionDirection::None, None),
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
            //(3, 4, None),
            Selection::new_unchecked(Range::new(3, 4), ExtensionDirection::None, None),
            //(8, 9, None)
            Selection::new_unchecked(Range::new(8, 9), ExtensionDirection::None, None),
        ], 
        0
    );
}

use edit::{
    application::{SelectionAction::Surround, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, ExtensionDirection},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::selection_actions::test_selection_action;



#[test] fn with_non_extended_selection(){   //also ensures primary updates properly
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
        Surround, 
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
        1, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(1, 2, None),
            Selection::new_unchecked(Range::new(1, 2), ExtensionDirection::None, None),
            //(4, 5, None),
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), ExtensionDirection::None, None),
        ], 
        2
    );
}

#[test] fn with_extended_selection(){
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
        Surround, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 3, None),
            Selection::new_unchecked(Range::new(0, 3), ExtensionDirection::Forward, None),
            //(4, 8, None)
            Selection::new_unchecked(Range::new(4, 8), ExtensionDirection::Forward, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(3, 4, None),
            Selection::new_unchecked(Range::new(3, 4), ExtensionDirection::None, None),
            //(4, 5, None),
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, None),
            //(8, 9, None)
            Selection::new_unchecked(Range::new(8, 9), ExtensionDirection::None, None),
        ], 
        0
    );
}

//mixed valid and invalid selections  //one at doc end, one not
#[test] fn mixed_valid_and_invalid_selections(){    //also ensures primary updates properly
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
        Surround, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        1, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(1, 2, None),
            Selection::new_unchecked(Range::new(1, 2), ExtensionDirection::None, None),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        2
    );
}

#[test] fn errors_if_single_selection_at_doc_end(){
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
        Surround, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
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
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        0
    );
}

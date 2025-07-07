use edit::{
    application::{SelectionAction::ExtendSelectionDown, Mode},
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
        ExtendSelectionDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\nidk\n", 
        vec![
            //(0, 1, None),   //common use
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(9, 10, None)   //to shorter line
            Selection::new_unchecked(Range::new(9, 10), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 6, Some(0)),
            Selection::new_unchecked(Range::new(0, 6), ExtensionDirection::Forward, Some(0)),
            //(9, 14, Some(4))
            Selection::new_unchecked(Range::new(9, 14), ExtensionDirection::Forward, Some(4)),
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
        ExtendSelectionDown, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),   //valid
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(14, 15, None)  //invalid
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 5, Some(0)),
            Selection::new_unchecked(Range::new(0, 5), ExtensionDirection::Forward, Some(0)),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        0
    );
}

#[test] fn errors_when_single_selection_on_bottom_line_block_semantics(){
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
        ExtendSelectionDown, 
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

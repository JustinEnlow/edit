use edit::{
    application::{SelectionAction::ExtendSelectionHome, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
};
use crate::selection_actions::test_selection_action;



#[test] fn when_cursor_past_line_text_start_block_semantics(){
    test_selection_action(
        ExtendSelectionHome, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    idk\n    something\n", 
        vec![
            (6, 7, None),
            (16, 17, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (7, 4, Some(4)),
            (17, 12, Some(4))
        ], 
        0
    );
}
#[test] fn when_cursor_at_line_text_start_block_semantics(){
    test_selection_action(
        ExtendSelectionHome, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    idk\n    something\n", 
        vec![
            (4, 5, None),
            (12, 13, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (5, 0, Some(0)),
            (13, 8, Some(0))
        ], 
        0
    );
}
#[test] fn when_cursor_before_line_text_start_block_semantics(){
    test_selection_action(
        ExtendSelectionHome, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    idk\n    something\n", 
        vec![
            (2, 3, None),
            (10, 11, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (2, 5, Some(4)),
            (10, 13, Some(4))
        ], 
        0
    );
}

#[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
    test_selection_action(
        ExtendSelectionHome, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsomething\n", 
        vec![
            (0, 1, None),
            (6, 7, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, None),
            (7, 4, Some(0))
        ], 
        0
    );
}

#[test] fn errors_when_line_start_and_line_text_start_and_cursor_position_all_equal_block_semantics(){
    test_selection_action(
        ExtendSelectionHome, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsomething\n", 
        vec![
            (0, 1, None),
            (4, 5, None)
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
            (0, 1, None),
            (4, 5, None)
        ], 
        0
    );
}

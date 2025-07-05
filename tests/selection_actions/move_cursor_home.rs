use edit::{
    application::{SelectionAction::MoveCursorHome, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
};
use crate::selection_actions::test_selection_action;



//TODO: test switching between line start and line text start

#[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
    test_selection_action(
        MoveCursorHome, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),   //invalid
            (6, 7, None),   //from middle of line
            (13, 14, None)  //from end of line
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, None),
            (4, 5, Some(0)),
            (9, 10, Some(0))
        ], 
        0
    );
}

#[test] fn errors_when_single_selection_at_line_start_block_semantics(){
    test_selection_action(
        MoveCursorHome, 
        Block, 
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

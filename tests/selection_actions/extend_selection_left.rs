use edit::{
    application::{SelectionAction::ExtendSelectionLeft, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
};
use crate::selection_actions::test_selection_action;



//TODO: updates stored line position on line change

#[test] fn normal_use_block_semantics(){    //+ trims newline from selection
    // i d k \n         // i d k \n
    // s o m e \n       // s o m e \n
    // s h i t|\n>      // s h i<t|\n
    //                  //
    test_selection_action(
        ExtendSelectionLeft, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (13, 14, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (13, 12, Some(3))
        ], 
        0
    );
}

#[test] fn extends_to_doc_start_block_semantics(){
    test_selection_action(
        ExtendSelectionLeft, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (1, 2, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (2, 0, Some(0))
        ], 
        0
    );
}

#[test] fn with_previously_forward_extended_selection(){
    //|i d k ⏎         //|i d k ⏎
    // s o m e ⏎       // s o m e ⏎
    // s h i t:⏎>      // s h i:t>⏎
    //
    test_selection_action(
        ExtendSelectionLeft, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 14, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 13, Some(3))
        ], 
        0
    );
}

#[test] fn errors_if_cursor_at_doc_start_block_semantics(){
    test_selection_action(
        ExtendSelectionLeft, 
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

#[test] fn errors_if_already_extended_backward_at_doc_start_block_semantics(){
    test_selection_action(
        ExtendSelectionLeft, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (14, 0, None)
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
            (14, 0, None)
        ], 
        0
    );
}

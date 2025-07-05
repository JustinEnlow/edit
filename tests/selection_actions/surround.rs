use edit::{
    application::{SelectionAction::Surround, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
};
use crate::selection_actions::test_selection_action;



#[test] fn with_non_extended_selection(){   //also ensures primary updates properly
    test_selection_action(
        Surround, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),
            (4, 5, None)
        ], 
        1, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, None),
            (1, 2, None),
            (4, 5, None),
            (5, 6, None)
        ], 
        2
    );
}

#[test] fn with_extended_selection(){
    test_selection_action(
        Surround, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 3, None),
            (4, 8, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, None),
            (3, 4, None),
            (4, 5, None),
            (8, 9, None)
        ], 
        0
    );
}

//mixed valid and invalid selections  //one at doc end, one not
#[test] fn mixed_valid_and_invalid_selections(){    //also ensures primary updates properly
    test_selection_action(
        Surround, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),
            (14, 15, None)
        ], 
        1, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, None),
            (1, 2, None),
            (14, 15, None)
        ], 
        2
    );
}

#[test] fn errors_if_single_selection_at_doc_end(){
    test_selection_action(
        Surround, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (14, 15, None)
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
            (14, 15, None)
        ], 
        0
    );
}

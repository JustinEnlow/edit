use edit::{
    application::{SelectionAction::ClearNonPrimarySelections, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SINGLE_SELECTION_DISPLAY_MODE, SINGLE_SELECTION}
};
use crate::selection_actions::test_selection_action;



#[test] fn clears_non_primary_with_multiple_selections(){
    //test(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    vec![
    //        (0, 1, None),
    //        (4, 5, None)
    //    ], 0, 
    //    vec![
    //        (0, 1, None)
    //    ], 0
    //);
    test_selection_action(
        ClearNonPrimarySelections, 
        Block, 
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
            (0, 1, None)
        ], 
        0
    );
}
    
#[test] fn errors_if_single_selection(){
    //test_error(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    vec![
    //        (0, 1, None)
    //    ], 0
    //);
    test_selection_action(
        ClearNonPrimarySelections, 
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
        match SINGLE_SELECTION_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(SINGLE_SELECTION.to_string())},
            DisplayMode::Warning => {Mode::Warning(SINGLE_SELECTION.to_string())},
            DisplayMode::Notify => {Mode::Notify(SINGLE_SELECTION.to_string())},
            DisplayMode::Info => {Mode::Info(SINGLE_SELECTION.to_string())},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            (0, 1, None)
        ], 
        0
    );
}

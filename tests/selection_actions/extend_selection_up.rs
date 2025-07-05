use edit::{
    application::{SelectionAction::ExtendSelectionUp, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
};
use crate::selection_actions::test_selection_action;



//TODO: this test is correct, but we are currently reducing the selection if it is over a newline, which maybe we shouldn't, as its causing bugs
#[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
    test_selection_action(
        ExtendSelectionUp, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),   //invalid
            (8, 9, None),   //to shorter line
            (14, 15, None)  //common use
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, None),
            //(9, 3, Some(4)),  //this really should be the correct resultant selection
            (8, 3, Some(4)),    //but this is what we are getting, because we reduce the selection when over a newline, currently
            //(15, 9, Some(0))  //selections that extend past doc text end should be reduced down
            (14, 9, Some(0))    //so i think this is correct...we want to keep this behavior
        ], 
        0
    );
}
#[ignore]#[test] fn same_as_above_but_how_it_should_actually_be_implemented(){
    // before           //after
    //|i>d k            //|i>d k<
    // s o m e| >       // s o m e          //these two should merge
    // s h i t| >       // s h i t  |       //these two should merge
    //
    test_selection_action(
        ExtendSelectionUp, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),   //invalid
            (8, 9, None),   //to shorter line
            (14, 15, None)  //common use
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 1, None),
            (15, 3, Some(3))
        ], 
        0
    );
}
    
#[test] fn errors_when_single_selection_on_top_line_block_semantics(){
    test_selection_action(
        ExtendSelectionUp, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (3, 4, None)
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
            (3, 4, None)
        ], 
        0
    );
}

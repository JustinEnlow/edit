use edit::{
    application::{SelectionAction::ExtendSelectionUp, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, ExtensionDirection},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::selection_actions::test_selection_action;



//TODO: this test is correct, but we are currently reducing the selection if it is over a newline, which maybe we shouldn't, as its causing bugs
#[ignore] #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
    //0 1 2 3  4 5 6 7 8  9 0 1 2 3  4
    // i d k \n s o m e \n s h i t \n

    //|i>d k \n         //|i>d k<\n
    // s o m e|\n>      // s o m e|\n
    // s h i t \n       // s h i t<\n|
    //| >               //
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
        ExtendSelectionUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),   //invalid
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(8, 9, None),   //to shorter line
            Selection::new_unchecked(Range::new(8, 9), ExtensionDirection::None, None),
            //(14, 15, None)  //common use
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(9, 3, Some(4)),  //this really should be the correct resultant selection
            //(8, 3, Some(4)),    //but this is what we are getting, because we reduce the selection when over a newline, currently
            Selection::new_unchecked(Range::new(3, 8), ExtensionDirection::Backward, Some(4)),
            //(15, 9, Some(0))  //selections that extend past doc text end should be reduced down
            //(14, 9, Some(0))    //so i think this is correct...we want to keep this behavior
            Selection::new_unchecked(Range::new(9, 14), ExtensionDirection::Backward, Some(0)),
        ], 
        0
    );
}
/*#[ignore] */#[test] fn same_as_above_but_how_it_should_actually_be_implemented(){
    //0 1 2 3  4 5 6 7 8  9 0 1 2 3  4
    // i d k \n s o m e \n s h i t \n

    // before           //after
    //|i>d k \n         //|i>d k<\n
    // s o m e|\n>      // s o m e \n|
    // s h i t \n       //<s h i t \n|
    //| >               //
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
        ExtendSelectionUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),   //invalid
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(8, 9, None),   //to shorter line
            Selection::new_unchecked(Range::new(8, 9), ExtensionDirection::None, None),
            //(14, 15, None)  //common use
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(15, 3, Some(3))
            //Selection::new_unchecked(Range::new(3, 15), ExtensionDirection::Backward, None),
            Selection::new_unchecked(Range::new(3, 9), ExtensionDirection::Backward, Some(4)),
            Selection::new_unchecked(Range::new(9, 14), ExtensionDirection::Backward, Some(0)),
        ], 
        0
    );
}
    
#[test] fn errors_when_single_selection_on_top_line_block_semantics(){
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
        ExtendSelectionUp, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(3, 4, None)
            Selection::new_unchecked(Range::new(3, 4), ExtensionDirection::None, None),
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
            //(3, 4, None)
            Selection::new_unchecked(Range::new(3, 4), ExtensionDirection::None, None),
        ], 
        0
    );
}

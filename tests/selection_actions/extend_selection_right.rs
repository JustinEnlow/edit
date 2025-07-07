use edit::{
    application::{SelectionAction::ExtendSelectionRight, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::selection_actions::test_selection_action;



#[test] fn normal_use_block_semantics(){
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
        ExtendSelectionRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 2, Some(1))
            Selection::new_unchecked(Range::new(0, 2), Some(Direction::Forward), Some(1)),
        ], 
        0
    );
}

#[test] fn extends_to_doc_text_end_block_semantics(){
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
        ExtendSelectionRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(12, 13, None)
            Selection::new_unchecked(Range::new(12, 13), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(12, 14, Some(4))
            Selection::new_unchecked(Range::new(12, 14), Some(Direction::Forward), Some(4)),
        ], 
        0
    );
}

/*#[ignore] */#[test] fn with_previously_backward_extended_selection(){
    //<i d k \n         // i<d k \n
    // s o m e \n       // s o m e \n
    // s h i t \n|      // s h i t \n|
    //                  //
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
        ExtendSelectionRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(14, 0, None)
            Selection::new_unchecked(Range::new(0, 14), Some(Direction::Backward), None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(14, 1, Some(1))
            Selection::new_unchecked(Range::new(1, 14), Some(Direction::Backward), Some(1)),
        ], 
        0
    );
}

#[test] fn errors_if_cursor_at_doc_text_end_block_semantics(){
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
        ExtendSelectionRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(13, 14, None)
            Selection::new_unchecked(Range::new(13, 14), None, None),
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
            //(13, 14, None)
            Selection::new_unchecked(Range::new(13, 14), None, None),
        ], 
        0
    );
}

#[test] fn errors_if_already_extended_forward_at_doc_text_end_block_semantics(){
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
        ExtendSelectionRight, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 14, None)
            Selection::new_unchecked(Range::new(0, 14), Some(Direction::Forward), None),
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
            //(0, 14, None)
            Selection::new_unchecked(Range::new(0, 14), Some(Direction::Forward), None),
        ], 
        0
    );
}

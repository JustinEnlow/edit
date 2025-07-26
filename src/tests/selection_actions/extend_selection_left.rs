use crate::{
    action::SelectionAction::ExtendSelectionLeft,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::tests::selection_actions::test_selection_action;



//TODO: updates stored line position on line change

/*#[ignore] */#[test] fn normal_use_block_semantics(){    //+ trims newline from selection
    // i d k \n         // i d k \n
    // s o m e \n       // s o m e \n
    // s h i t|\n>      // s h i<t \n|
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
        ExtendSelectionLeft, 
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
        Mode::Insert, 
        vec![
            //(13, 12, Some(3))
            Selection::new_unchecked(Range::new(12, 14), Some(Direction::Backward), Some(3)),
        ], 
        0
    );
}

#[test] fn extends_to_doc_start_block_semantics(){
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
        ExtendSelectionLeft, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(1, 2, None)
            Selection::new_unchecked(Range::new(1, 2), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(2, 0, Some(0))
            Selection::new_unchecked(Range::new(0, 2), Some(Direction::Backward), Some(0)),
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
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        ExtendSelectionLeft, 
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
        Mode::Insert, 
        vec![
            //(0, 13, Some(3))
            Selection::new_unchecked(Range::new(0, 13), Some(Direction::Forward), Some(3)),
        ], 
        0
    );
}

#[test] fn if_extending_from_end_of_buffer_cursor_is_moved_left_instead(){  //because we can't have a selection past buffer text end
    //0 1 2 3  4 5 6 7 8  9 0 1 2 3  4
    // i d k \n s o m e \n s h i t \n

    // i d k \n         // i d k \n
    // s o m e \n       // s o m e \n
    // s h i t \n       // s h i t|\n>
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
        ExtendSelectionLeft, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(Range::new(14, 15), None, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(Range::new(13, 14), None, Some(4))
        ], 
        0
    );
}

#[test] fn errors_if_cursor_at_doc_start_block_semantics(){
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
        ExtendSelectionLeft, 
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
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
            DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
            DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
            DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
        ], 
        0
    );
}

#[test] fn errors_if_already_extended_backward_at_doc_start_block_semantics(){
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
        ExtendSelectionLeft, 
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
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
            DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
            DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
            DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            //(14, 0, None)
            Selection::new_unchecked(Range::new(0, 14), Some(Direction::Backward), None),
        ], 
        0
    );
}

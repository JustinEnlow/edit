use crate::{
    application::{SelectionAction::ExtendSelectionWordBoundaryBackward, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn with_multiple_valid_selections(){
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
        ExtendSelectionWordBoundaryBackward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(2, 3, None),
            Selection::new_unchecked(Range::new(2, 3), None, None),
            //(7, 8, None)
            Selection::new_unchecked(Range::new(7, 8), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(3, 0, Some(0)),
            Selection::new_unchecked(Range::new(0, 3), Some(Direction::Backward), Some(0)),
            //(8, 4, Some(0))
            Selection::new_unchecked(Range::new(4, 8), Some(Direction::Backward), Some(0)),
        ], 
        0
    );
}

#[test] fn with_mixed_valid_and_invalid_selections(){
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
        ExtendSelectionWordBoundaryBackward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(7, 8, None)
            Selection::new_unchecked(Range::new(7, 8), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(8, 4, Some(0))
            Selection::new_unchecked(Range::new(4, 8), Some(Direction::Backward), Some(0)),
        ], 
        0
    );
}
    
#[test] fn extends_to_doc_start_if_no_other_word_boundaries(){
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
        ExtendSelectionWordBoundaryBackward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "    idk\nsome\nshit\n", 
        vec![
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(5, 0, Some(0))
            Selection::new_unchecked(Range::new(0, 5), Some(Direction::Backward), Some(0)),
        ], 
        0
    );
}
    
#[test] fn shrinks_previously_forward_extended_selection(){
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
        ExtendSelectionWordBoundaryBackward, 
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
            //(0, 10, Some(0))
            Selection::new_unchecked(Range::new(0, 10), Some(Direction::Forward), Some(0)),
        ], 
        0
    );
}

#[ignore] #[test] fn works_with_count(){
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
        ExtendSelectionWordBoundaryBackward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(9, 10, None)
            Selection::new_unchecked(Range::new(9, 10), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 10, Some(0))
            Selection::new_unchecked(Range::new(0, 10), Some(Direction::Forward), Some(0)),
        ], 
        0
    );
}
#[ignore] #[test] fn works_with_non_alphanumeric_graphemes(){
    unimplemented!()
}



//////////////////////// ERRORS ///////////////////////////
#[test] fn errors_if_count_less_than_1(){
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
        ExtendSelectionWordBoundaryBackward, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
        ], 
        0, 
        0, 
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
            DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
            DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
            DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
        ], 
        0
    );
}
#[test] fn errors_if_single_selection_at_doc_start(){
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
        ExtendSelectionWordBoundaryBackward, 
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
#[test] fn errors_if_already_extended_backwards_to_doc_start(){
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
        ExtendSelectionWordBoundaryBackward, 
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

//#[test] fn extend_left_word_boundary(){
//    let text = Rope::from("use std::error::Error;");
//    assert_eq!(Selection::with_stored_line_position(Range::new(0, 4), Direction::Backward, 0), Selection::new(Range::new(3, 4), Direction::Forward).extend_left_word_boundary(&text, CursorSemantics::Block).unwrap());
//    assert_eq!(Selection::with_stored_line_position(Range::new(0, 3), Direction::Backward, 0), Selection::new(Range::new(3, 3), Direction::Forward).extend_left_word_boundary(&text, CursorSemantics::Bar).unwrap());
//}
//#[test] fn extend_left_word_boundary_error(){
//    let text = Rope::from("idk\nsome\nshit\n");
//    assert!(Selection::new(Range::new(0, 1), Direction::Forward).extend_left_word_boundary(&text, CursorSemantics::Block).is_err());
//    assert!(Selection::new(Range::new(0, 0), Direction::Forward).extend_left_word_boundary(&text, CursorSemantics::Bar).is_err());
//}

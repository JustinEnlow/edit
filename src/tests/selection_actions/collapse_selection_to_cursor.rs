use crate::{
    action::SelectionAction::CollapseSelectionToCursor,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::tests::selection_actions::test_selection_action;



//TODO: should these functions really result in selections with a stored line position?...
    
#[test] fn collapses_to_cursor_with_multiple_selections_with_selection_forward(){
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
        CollapseSelectionToCursor, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 3, None),
            Selection::new_unchecked(Range::new(0, 3), Some(Direction::Forward), None),
            //(4, 8, None)
            Selection::new_unchecked(Range::new(4, 8), Some(Direction::Forward), None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(2, 3, Some(2)),
            Selection::new_unchecked(Range::new(2, 3), None, Some(2)),
            //(7, 8, Some(3))
            Selection::new_unchecked(Range::new(7, 8), None, Some(3)),
        ], 
        0
    );
}
#[test] fn collapses_to_cursor_with_multiple_selections_with_selection_backward(){
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
        CollapseSelectionToCursor, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(3, 0, None),
            Selection::new_unchecked(Range::new(0, 3), Some(Direction::Backward), None),
            //(8, 4, None)
            Selection::new_unchecked(Range::new(4, 8), Some(Direction::Backward), None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, Some(0)),
            Selection::new_unchecked(Range::new(0, 1), None, Some(0)),
            //(4, 5, Some(0))
            Selection::new_unchecked(Range::new(4, 5), None, Some(0)),
        ], 
        0
    );
}

#[test] fn collapses_to_cursor_with_mixed_extension(){
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
        CollapseSelectionToCursor, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(4, 8, None)
            Selection::new_unchecked(Range::new(4, 8), Some(Direction::Forward), None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(7, 8, Some(3))
            Selection::new_unchecked(Range::new(7, 8), None, Some(3)),
        ], 
        0
    );
}

#[test] fn errors_if_already_collapsed(){
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
        CollapseSelectionToCursor, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
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
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
        ], 
        0
    );
}
//maybe test above with single selection too...idk

use crate::{
    application::{SelectionAction::RemovePrimarySelection, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SINGLE_SELECTION_DISPLAY_MODE, SINGLE_SELECTION, Config}
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn when_primary_is_first_next_becomes_new_primary(){
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
        RemovePrimarySelection, 
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
        Mode::Insert, 
        vec![
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), None, None),
        ], 
        0
    );
}
#[test] fn when_primary_not_first_previous_becomes_new_primary(){
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
        RemovePrimarySelection, 
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
        1, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
        ], 
        0
    );
}

#[test] fn errors_if_single_selection(){
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
        RemovePrimarySelection, 
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
        match SINGLE_SELECTION_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(SINGLE_SELECTION.to_string())},
            DisplayMode::Warning => {Mode::Warning(SINGLE_SELECTION.to_string())},
            DisplayMode::Notify => {Mode::Notify(SINGLE_SELECTION.to_string())},
            DisplayMode::Info => {Mode::Info(SINGLE_SELECTION.to_string())},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
        ], 
        0
    );
}

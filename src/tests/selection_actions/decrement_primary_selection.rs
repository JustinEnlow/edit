use crate::{
    action::SelectionAction::DecrementPrimarySelection,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block},
    display_area::DisplayArea,
    config::{DisplayMode, SINGLE_SELECTION_DISPLAY_MODE, Config},
    keybind::default_keybinds
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn with_multiple_selections(){
    test_selection_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        DecrementPrimarySelection, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
        ], 
        1, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0
    );
}
#[test] fn wraps_if_primary_is_first(){
    test_selection_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        DecrementPrimarySelection, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
        ], 
        1
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        DecrementPrimarySelection, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
        ], 
        0, 
        1, 
        match SINGLE_SELECTION_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error},
            DisplayMode::Warning => {Mode::Warning},
            DisplayMode::Notify => {Mode::Notify},
            DisplayMode::Info => {Mode::Info},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
        ], 
        0
    );
}

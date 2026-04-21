use crate::{
    action::SelectionAction::CollapseSelectionToCursor,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, Config},
    keybind::default_keybinds
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        CollapseSelectionToCursor, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
            //|i d:k>⏎|s o m:e>⏎ s h i t ⏎
            //0 1 2 3 4 5
            //|i d:k>⏎
            //|s o m:e>⏎
            // s h i t ⏎
            //          

            Selection::new_unchecked(0..3, Some(Direction::Forward), 2),
            Selection::new_unchecked(4..8, Some(Direction::Forward), 3),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
            // i d|k>⏎ s o m|e>⏎ s h i t ⏎
            //0 1 2 3 4 5
            // i d|k>⏎
            // s o m|e>⏎
            // s h i t ⏎
            //          

            Selection::new_unchecked(2..3, None, 2),
            Selection::new_unchecked(7..8, None, 3),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        CollapseSelectionToCursor, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..3, Some(Direction::Backward), 0),
            Selection::new_unchecked(4..8, Some(Direction::Backward), 0),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        CollapseSelectionToCursor, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..8, Some(Direction::Forward), 3),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(7..8, None, 3),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        CollapseSelectionToCursor, 
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
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error},
            DisplayMode::Warning => {Mode::Warning},
            DisplayMode::Notify => {Mode::Notify},
            DisplayMode::Info => {Mode::Info},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0
    );
}
//maybe test above with single selection too...idk

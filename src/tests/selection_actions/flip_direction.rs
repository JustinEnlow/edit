use crate::{
    action::SelectionAction::FlipDirection,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, /*SAME_STATE, */Config},
    keybind::default_keybinds
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn forward_selections_flip_backwards_block_semantics(){
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
        FlipDirection, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 4, None),
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward), None),
            //(4, 9, None)
            Selection::new_unchecked(Range::new(4, 9), Some(Direction::Forward), None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(4, 0, None),
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Backward), None),
            //(9, 4, None)
            Selection::new_unchecked(Range::new(4, 9), Some(Direction::Backward), None),
        ], 
        0
    );
}

#[test] fn backward_selections_flip_forwards_block_semantics(){
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
        FlipDirection, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(4, 0, None),
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Backward), None),
            //(9, 4, None)
            Selection::new_unchecked(Range::new(4, 9), Some(Direction::Backward), None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 4, None),
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward), None),
            //(4, 9, None)
            Selection::new_unchecked(Range::new(4, 9), Some(Direction::Forward), None),
        ], 
        0
    );
}

#[test] fn non_extended_return_error_block_semantics(){
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
        FlipDirection, 
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
            DisplayMode::Error => {Mode::Error/*(SAME_STATE.to_string())*/},
            DisplayMode::Warning => {Mode::Warning/*(SAME_STATE.to_string())*/},
            DisplayMode::Notify => {Mode::Notify/*(SAME_STATE.to_string())*/},
            DisplayMode::Info => {Mode::Info/*(SAME_STATE.to_string())*/},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), None, None),
        ], 
        0
    );
}

//TODO: what about mixed directions? should they even be allowed?...

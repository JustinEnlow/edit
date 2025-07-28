use crate::{
    action::SelectionAction::MoveCursorLineEnd,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, /*SAME_STATE, */Config},
    keybind::default_keybinds
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
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
        MoveCursorLineEnd, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),   //common use
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(6, 7, None),   //from middle of line
            Selection::new_unchecked(Range::new(6, 7), None, None),
            //(14, 15, None)  //invalid. already at line end
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(3, 4, Some(3)),
            Selection::new_unchecked(Range::new(3, 4), None, Some(3)),
            //(8, 9, Some(4)),
            Selection::new_unchecked(Range::new(8, 9), None, Some(4)),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), None, None),
        ], 
        0
    );
}

#[test] fn errors_when_single_selection_at_line_end_block_semantics(){
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
        MoveCursorLineEnd, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(3, 4, None)
            Selection::new_unchecked(Range::new(3, 4), None, None),
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
            //(3, 4, None)
            Selection::new_unchecked(Range::new(3, 4), None, None),
        ], 
        0
    );
}

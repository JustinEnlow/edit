use crate::{
    action::EditAction::InsertTab,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, /*READ_ONLY_BUFFER, *//*USE_HARD_TAB, */Config},
    keybind::default_keybinds
};
use crate::tests::edit_actions::test_edit_action;

const USE_HARD_TAB: bool = false;

#[test] fn with_multiple_selections(){
    test_edit_action(
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
        InsertTab, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(5..6, None, 0),
        ], 
        0, 
        "",
        Mode::Insert, 
        if USE_HARD_TAB{"\tsome\n\tshit\n"}else{"    some\n    shit\n"},
        if USE_HARD_TAB{
            vec![   //\tsome\n\tshit\n
                Selection::new_unchecked(1..2, None, 0),
                Selection::new_unchecked(7..8, None, 0),
            ]
        }else{
            vec![   //    some\n    shit\n      //this would depend on TAB_WIDTH as well...
                Selection::new_unchecked(4..5, None, 4),
                Selection::new_unchecked(13..14, None, 4),
            ]
        },
        0,
        ""
    );
}

#[test] fn with_read_only_buffer_is_error(){
    test_edit_action(
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
        InsertTab, 
        false, 
        false, 
        true, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(5..6, None, 0),
        ], 
        0, 
        "",
        match READ_ONLY_BUFFER_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error}
            DisplayMode::Warning => {Mode::Warning}
            DisplayMode::Notify => {Mode::Notify}
            DisplayMode::Info => {Mode::Info}
            DisplayMode::Ignore => {Mode::Insert}
        }, 
        "some\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(5..6, None, 0),
        ], 
        0,
        ""
    );
}

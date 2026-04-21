use crate::{
    action::EditAction::Paste,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, /*READ_ONLY_BUFFER, */INVALID_INPUT_DISPLAY_MODE, /*INVALID_INPUT, */Config},
    keybind::default_keybinds
};
use crate::tests::edit_actions::test_edit_action;

#[test] fn paste_single_selection_block_semantics(){
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
        Paste, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(9..10, None, 0),
        ], 
        0, 
        "other\n",
        Mode::Insert, 
        "idk\nsome\nother\nshit\n", 
        vec![
            Selection::new_unchecked(15..16, None, 0),
        ], 
        0, 
        "other\n"
    );
}
//TODO: paste_multi_selection_block_semantics

#[test] fn errors_if_empty_clipboard(){
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
        Paste, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nshit\n", 
        vec![
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0, 
        "",
        match INVALID_INPUT_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error},
            DisplayMode::Warning => {Mode::Warning},
            DisplayMode::Notify => {Mode::Notify},
            DisplayMode::Info => {Mode::Info},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        "idk\nshit\n", 
        vec![
            Selection::new_unchecked(4..5, None, 0),
        ], 
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
        Paste, 
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
        "idk",
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
        "idk"
    );
}

use crate::{
    action::EditAction::Backspace,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, /*READ_ONLY_BUFFER, */SAME_STATE_DISPLAY_MODE, /*SAME_STATE, */Config},
    keybind::default_keybinds
};
use crate::tests::edit_actions::test_edit_action;

#[test] fn common_use(){
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
        Backspace, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(1..2, None, 1),
            Selection::new_unchecked(5..6, None, 1),
        ], 
        0, 
        "",
        Mode::Insert, 
        "dk\nome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(3..4, None, 0),
        ], 
        0,
        ""
    );
}

#[test] fn when_at_line_start_appends_current_line_to_previous_line(){
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
        Backspace, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0, 
        "",
        Mode::Insert, 
        "idksome\nshit\n", 
        vec![
            Selection::new_unchecked(3..4, None, 3),
        ], 
        0,
        ""
    );
}

#[test] fn with_valid_selection_and_cursor_at_doc_start(){
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
        Backspace, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0, 
        "",
        Mode::Insert, 
        "idksome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(3..4, None, 3),
        ], 
        0,
        ""
    );
}

#[test] fn with_extended_selection_deletes_selection(){
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
        Backspace, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..4, Some(Direction::Forward), 3),
        ], 
        0, 
        "",
        Mode::Insert, 
        "some\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
        ], 
        0,
        ""
    );
}

#[test] fn errors_if_single_cursor_at_doc_start(){
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
        Backspace, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
        ], 
        0, 
        "",
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error}
            DisplayMode::Warning => {Mode::Warning}
            DisplayMode::Notify => {Mode::Notify}
            DisplayMode::Info => {Mode::Info}
            DisplayMode::Ignore => {Mode::Insert}
        }, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
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
        Backspace, 
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

//TODO: test tab deletion with soft tabs
//TODO: test tab deletion with hard tabs
//TODO: test with various tab widths


//TODO: test error described in range.rs:15:9

use crate::{
    action::EditAction::Cut,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, /*READ_ONLY_BUFFER, */MULTIPLE_SELECTIONS_DISPLAY_MODE, /*MULTIPLE_SELECTIONS, */Config},
    keybind::default_keybinds
};
use crate::tests::edit_actions::test_edit_action;

#[test] fn cut_with_selection_direction_forward_block_semantics(){
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
        Cut, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(4..9, Some(Direction::Forward), 0),
        ], 
        0, 
        "",
        Mode::Insert, 
        "idk\nshit\n", 
        vec![
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0,
        "some\n"
    );
}

#[test] fn cut_with_selection_direction_backward_block_semantics(){
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
        Cut, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(4..9, Some(Direction::Backward), 0),
        ], 
        0, 
        "",
        Mode::Insert, 
        "idk\nshit\n", 
        vec![
            Selection::new_unchecked(4..5, None, 0),
        ], 
        0,
        "some\n"
    );
}

#[test] fn cut_with_multiple_selections_returns_error(){
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
        Cut, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..3, Some(Direction::Forward), 2),
            Selection::new_unchecked(4..7, Some(Direction::Forward), 2),
        ], 
        0, 
        "",
        match MULTIPLE_SELECTIONS_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error}
            DisplayMode::Warning => {Mode::Warning}
            DisplayMode::Notify => {Mode::Notify}
            DisplayMode::Info => {Mode::Info}
            DisplayMode::Ignore => {Mode::Insert}
        }, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..3, Some(Direction::Forward), 2),
            Selection::new_unchecked(4..7, Some(Direction::Forward), 2),
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
        Cut, 
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

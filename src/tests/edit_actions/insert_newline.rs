use crate::{
    application::{EditAction::InsertNewline, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER, Config}
};
use crate::tests::edit_actions::test_edit_action;

#[test] fn with_multiple_selections(){
    test_edit_action(
        Config{
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false
        },
        InsertNewline, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), None, None),
        ], 
        0, 
        "",
        "\nsome\n\nshit\n", 
        Mode::Insert, 
        vec![
            //(1, 2, Some(0)),
            Selection::new_unchecked(Range::new(1, 2), None, Some(0)),
            //(7, 8, Some(0))
            Selection::new_unchecked(Range::new(7, 8), None, Some(0)),
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
            show_cursor_line: false
        },
        InsertNewline, 
        //CursorSemantics::Block, 
        false, 
        false, 
        true, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), None, None),
        ], 
        0, 
        "",
        "some\nshit\n", 
        match READ_ONLY_BUFFER_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(READ_ONLY_BUFFER.to_string())}
            DisplayMode::Warning => {Mode::Warning(READ_ONLY_BUFFER.to_string())}
            DisplayMode::Notify => {Mode::Notify(READ_ONLY_BUFFER.to_string())}
            DisplayMode::Info => {Mode::Info(READ_ONLY_BUFFER.to_string())}
            DisplayMode::Ignore => {Mode::Insert}
        }, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), None, None),
        ], 
        0,
        ""
    );
}

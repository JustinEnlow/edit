use edit::{
    application::{EditAction::InsertChar, Mode},
    selection::CursorSemantics,
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER}
};
use crate::edit_actions::test_edit_action;

#[test] fn with_multiple_selections(){
    test_edit_action(
        InsertChar('x'), 
        CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            (0, 1, None),
            (5, 6, None)
        ], 
        0, 
        "",
        "xsome\nxshit\n", 
        Mode::Insert, 
        vec![
            (1, 2, Some(1)),
            (7, 8, Some(1))
        ], 
        0,
        ""
    );
}

#[test] fn with_read_only_buffer_is_error(){
    test_edit_action(
        InsertChar('x'), 
        CursorSemantics::Block, 
        false, 
        false, 
        true, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            (0, 1, None),
            (5, 6, None)
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
            (0, 1, None),
            (5, 6, None)
        ], 
        0,
        ""
    );
}

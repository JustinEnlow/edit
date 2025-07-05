use edit::{
    application::{EditAction::Paste, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER, INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT}
};
use crate::edit_actions::test_edit_action;

#[test] fn paste_single_selection_block_semantics(){
    test_edit_action(
        Paste, 
        Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (9, 10, None)
        ], 
        0, 
        "other\n",
        "idk\nsome\nother\nshit\n", 
        Mode::Insert, 
        vec![
            (15, 16, Some(0))
        ], 
        0, 
        "other\n"
    );
}
//TODO: paste_multi_selection_block_semantics

#[test] fn errors_if_empty_clipboard(){
    test_edit_action(
        Paste, 
        Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nshit\n", 
        vec![
            (4, 5, None)
        ], 
        0, 
        "",
        "idk\nshit\n", 
        match INVALID_INPUT_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(INVALID_INPUT.to_string())},
            DisplayMode::Warning => {Mode::Warning(INVALID_INPUT.to_string())},
            DisplayMode::Notify => {Mode::Notify(INVALID_INPUT.to_string())},
            DisplayMode::Info => {Mode::Info(INVALID_INPUT.to_string())},
            DisplayMode::Ignore => {Mode::Insert},
        }, 
        vec![
            (4, 5, None)
        ], 
        0, 
        ""
    );
}

#[test] fn with_read_only_buffer_is_error(){
    test_edit_action(
        Paste, 
        Block, 
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
        "idk",
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
        "idk"
    );
}

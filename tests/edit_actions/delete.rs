use edit::{
    application::{EditAction::Delete, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, ExtensionDirection},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::edit_actions::test_edit_action;

#[test] fn with_non_extended_selections(){
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
        Delete, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), ExtensionDirection::None, None),
        ], 
        0, 
        "",
        "dk\nome\nshit\n", 
        Mode::Insert, 
        vec![
            //(0, 1, Some(0)),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, Some(0)),
            //(3, 4, Some(0))
            Selection::new_unchecked(Range::new(3, 4), ExtensionDirection::None, Some(0)),
        ], 
        0,
        ""
    );
}

#[test] fn with_extended_selections(){
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
        Delete, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 2, None),
            Selection::new_unchecked(Range::new(0, 2), ExtensionDirection::Forward, None),
            //(4, 6, None)
            Selection::new_unchecked(Range::new(4, 6), ExtensionDirection::Forward, None),
        ], 
        0, 
        "",
        "k\nme\nshit\n", 
        Mode::Insert, 
        vec![
            //(0, 1, Some(0)),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, Some(0)),
            //(2, 3, Some(0))
            Selection::new_unchecked(Range::new(2, 3), ExtensionDirection::None, Some(0)),
        ], 
        0,
        ""
    );
}
//TODO: maybe test direction backward too?...

#[test] fn with_valid_selection_and_cursor_at_doc_end(){
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
        Delete, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(9, 10, None),
            Selection::new_unchecked(Range::new(9, 10), ExtensionDirection::None, None),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        0, 
        "",
        "idk\nsome\nhit\n", 
        Mode::Insert, 
        vec![
            //(9, 10, Some(0)),
            Selection::new_unchecked(Range::new(9, 10), ExtensionDirection::None, Some(0)),
            //(13, 14, None)
            Selection::new_unchecked(Range::new(13, 14), ExtensionDirection::None, None),
        ], 
        0,
        ""
    );
}

#[test] fn errors_if_single_cursor_at_doc_end(){
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
        Delete, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(14, 15, None),
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
        ], 
        0, 
        "",
        "idk\nsome\nshit\n", 
        match SAME_STATE_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())}
            DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())}
            DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())}
            DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())}
            DisplayMode::Ignore => {Mode::Insert}
        }, 
        vec![
            //(14, 15, None),
            Selection::new_unchecked(Range::new(14, 15), ExtensionDirection::None, None),
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
        Delete, 
        //CursorSemantics::Block, 
        false, 
        false, 
        true, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), ExtensionDirection::None, None),
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
            Selection::new_unchecked(Range::new(0, 1), ExtensionDirection::None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), ExtensionDirection::None, None),
        ], 
        0,
        ""
    );
}

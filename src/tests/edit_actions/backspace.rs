use crate::{
    application::{EditAction::Backspace, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
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
            show_cursor_line: false
        },
        Backspace, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(1, 2, None),
            Selection::new_unchecked(Range::new(1, 2), /*ExtensionDirection::*/None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), /*ExtensionDirection::*/None, None),
        ], 
        0, 
        "",
        "dk\nome\nshit\n", 
        Mode::Insert, 
        vec![
            //(0, 1, Some(0)),
            Selection::new_unchecked(Range::new(0, 1), /*ExtensionDirection::*/None, Some(0)),
            //(3, 4, Some(0))
            Selection::new_unchecked(Range::new(3, 4), /*ExtensionDirection::*/None, Some(0)),
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
            show_cursor_line: false
        },
        Backspace, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), /*ExtensionDirection::*/None, None),
        ], 
        0, 
        "",
        "idksome\nshit\n", 
        Mode::Insert, 
        vec![
            //(3, 4, Some(3))
            Selection::new_unchecked(Range::new(3, 4), /*ExtensionDirection::*/None, Some(3)),
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
            show_cursor_line: false
        },
        Backspace, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), /*ExtensionDirection::*/None, None),
            //(4, 5, None)
            Selection::new_unchecked(Range::new(4, 5), /*ExtensionDirection::*/None, None),
        ], 
        0, 
        "",
        "idksome\nshit\n", 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), /*ExtensionDirection::*/None, None),
            //(3, 4, Some(3))
            Selection::new_unchecked(Range::new(3, 4), /*ExtensionDirection::*/None, Some(3)),
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
            show_cursor_line: false
        },
        Backspace, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 4, None)
            Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward)/*ExtensionDirection::Forward*/, None),
        ], 
        0, 
        "",
        "some\nshit\n", 
        Mode::Insert, 
        vec![
            //(0, 1, Some(0)),
            Selection::new_unchecked(Range::new(0, 1), /*ExtensionDirection::*/None, Some(0)),
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
            show_cursor_line: false
        },
        Backspace, 
        //CursorSemantics::Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 1, None)
            Selection::new_unchecked(Range::new(0, 1), /*ExtensionDirection::*/None, None),
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
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), /*ExtensionDirection::*/None, None),
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
        Backspace, 
        //CursorSemantics::Block, 
        false, 
        false, 
        true, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), /*ExtensionDirection::*/None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), /*ExtensionDirection::*/None, None),
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
            Selection::new_unchecked(Range::new(0, 1), /*ExtensionDirection::*/None, None),
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), /*ExtensionDirection::*/None, None),
        ], 
        0,
        ""
    );
}

//TODO: test tab deletion with soft tabs
//TODO: test tab deletion with hard tabs
//TODO: test with various tab widths


//TODO: test error described in range.rs:15:9

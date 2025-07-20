use crate::{
    application::EditAction::AddSurround,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER, INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT, Config}
};
use crate::tests::edit_actions::test_edit_action;

#[test] fn with_single_selection(){
    //test(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    vec![
    //        (0, 3, None)
    //    ], 0, 
    //    '{', '}', 
    //    "{idk}\nsome\nshit\n", 
    //    vec![
    //        (5, 6, Some(5))
    //    ], 0
    //);
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
        AddSurround('{', '}'), 
        //Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 3, None)
            Selection::new_unchecked(Range::new(0, 3), Some(Direction::Forward)/*ExtensionDirection::Forward*/, None),
        ], 
        0, 
        "", 
        "{idk}\nsome\nshit\n", 
        Mode::Insert, 
        vec![
            //(5, 6, Some(5))
            Selection::new_unchecked(Range::new(5, 6), /*ExtensionDirection::*/None, Some(5)),
        ], 
        0, 
        ""
    );
}

//TODO: test multiple selections

//TODO: test with selection over newline(should be the same, but worth verifying...)

#[test] fn with_valid_selection_and_cursor_at_end_of_doc(){
    //test(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    vec![
    //        (9, 11, None),
    //        (14, 15, None)
    //    ], 0, 
    //    '<', '>', 
    //    "idk\nsome\n<sh>it\n", 
    //    vec![
    //        (13, 14, Some(4)),
    //        (16, 17, None)
    //    ], 0
    //);
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
        AddSurround('<', '>'), 
        //Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(9, 11, None),
            Selection::new_unchecked(Range::new(9, 11), Some(Direction::Forward)/*ExtensionDirection::Forward*/, None),
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), /*ExtensionDirection::*/None, None),
        ], 
        0, 
        "", 
        "idk\nsome\n<sh>it\n", 
        Mode::Insert, 
        vec![
            //(13, 14, Some(4)),
            Selection::new_unchecked(Range::new(13, 14), /*ExtensionDirection::*/None, Some(4)),
            //(16, 17, None)
            Selection::new_unchecked(Range::new(16, 17), /*ExtensionDirection::*/None, None),
        ], 
        0, 
        ""
    );
}

#[test] fn errors_when_single_cursor_at_end_of_document(){
    //test_error(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    vec![
    //        (14, 15, None)
    //    ], 0, 
    //    '{', '}'
    //);
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
        AddSurround('{', '}'), 
        //Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), /*ExtensionDirection::*/None, None),
        ], 
        0, 
        "", 
        "idk\nsome\nshit\n", 
        match INVALID_INPUT_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(INVALID_INPUT.to_string())}
            DisplayMode::Warning => {Mode::Warning(INVALID_INPUT.to_string())}
            DisplayMode::Notify => {Mode::Notify(INVALID_INPUT.to_string())}
            DisplayMode::Info => {Mode::Info(INVALID_INPUT.to_string())}
            DisplayMode::Ignore => {Mode::Insert}
        }, 
        vec![
            //(14, 15, None)
            Selection::new_unchecked(Range::new(14, 15), /*ExtensionDirection::*/None, None),
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
        AddSurround('[', ']'), 
        //Block, 
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

//TODO?: should resultant selection after adding surrounding pair be a selection over the content and pair?...
//i think this is a much deeper question than this single function...
//this relates to all replacement text  (if we use the default Document::apply_replace...)

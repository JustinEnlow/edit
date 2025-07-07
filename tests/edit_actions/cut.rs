use edit::{
    application::{EditAction::Cut, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER, MULTIPLE_SELECTIONS_DISPLAY_MODE, MULTIPLE_SELECTIONS, Config}
};
use crate::edit_actions::test_edit_action;

#[test] fn cut_with_selection_direction_forward_block_semantics(){
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
        Cut, 
        //Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(4, 9, None)
            Selection::new_unchecked(Range::new(4, 9), Some(Direction::Forward), None),
        ], 
        0, 
        "",
        "idk\nshit\n", 
        Mode::Insert, 
        vec![
            //(4, 5, Some(0))
            Selection::new_unchecked(Range::new(4, 5), None, Some(0)),
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
            show_cursor_line: false
        },
        Cut, 
        //Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(9, 4, None)
            Selection::new_unchecked(Range::new(4, 9), Some(Direction::Backward), None),
        ], 
        0, 
        "",
        "idk\nshit\n", 
        Mode::Insert, 
        vec![
            //(4, 5, Some(0))
            Selection::new_unchecked(Range::new(4, 5), None, Some(0)),
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
            show_cursor_line: false
        },
        Cut, 
        //Block, 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            //(0, 3, None),
            Selection::new_unchecked(Range::new(0, 3), Some(Direction::Forward), None),
            //(4, 7, None)
            Selection::new_unchecked(Range::new(4, 7), Some(Direction::Forward), None),
        ], 
        0, 
        "",
        "idk\nsome\nshit\n", 
        match MULTIPLE_SELECTIONS_DISPLAY_MODE{
            DisplayMode::Error => {Mode::Error(MULTIPLE_SELECTIONS.to_string())}
            DisplayMode::Warning => {Mode::Warning(MULTIPLE_SELECTIONS.to_string())}
            DisplayMode::Notify => {Mode::Notify(MULTIPLE_SELECTIONS.to_string())}
            DisplayMode::Info => {Mode::Info(MULTIPLE_SELECTIONS.to_string())}
            DisplayMode::Ignore => {Mode::Insert}
        }, 
        vec![
            //(0, 3, None),
            Selection::new_unchecked(Range::new(0, 3), Some(Direction::Forward), None),
            //(4, 7, None)
            Selection::new_unchecked(Range::new(4, 7), Some(Direction::Forward), None),
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
        Cut, 
        //Block, 
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

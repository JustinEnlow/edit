use crate::{
    action::EditAction::InsertChar,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, /*READ_ONLY_BUFFER, */Config},
    keybind::default_keybinds
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        InsertChar('x'), 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "some\nshit\n", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(5..6, None, 0),
        ], 
        0, 
        "",
        Mode::Insert, 
        "xsome\nxshit\n", 
        vec![
            Selection::new_unchecked(1..2, None, 1),
            Selection::new_unchecked(7..8, None, 1),
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
        InsertChar('x'), 
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

#[test] fn with_multibyte_grapheme(){
    test_edit_action(
        Config::default(), 
        InsertChar('⏎'), 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(3..4, None, 3)
        ], 
        0, 
        "", 
        Mode::Insert, 
        "idk⏎\nsome\nshit\n", 
        vec![
            //Selection::new_unchecked(4..5, None, 4)
            //01236 7               //bytes
            //idk⏎\nsome\nshit\n
            //01234 0               //line offset
            Selection::new_unchecked(6..7, None, 4)
        ], 
        0, 
        ""
    );
}
//TODO: with multichar grapheme
//TODO: with wide grapheme
//TODO: with zero width grapheme

#[test] fn replace_with_multibyte_grapheme(){
    test_edit_action(
        Config::default(), 
        InsertChar('⏎'), 
        false, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            Selection::new_unchecked(0..3, Some(Direction::Forward), 2)
        ], 
        0, 
        "", 
        Mode::Insert, 
        "⏎\nsome\nshit\n", 
        vec![
            //Selection::new_unchecked(1..2, None, 1)   //although, i am considering having selections after replacement be equivalent to before, with replacement text still selected...
            //03 456            //bytes
            //⏎\nsome\nshit\n
            //01 23             //line offset
            Selection::new_unchecked(3..4, None, 1)
        ], 
        0, 
        ""
    );
}
//TODO: replace with multichar grapheme
//TODO: replace with wide grapheme
//TODO: replace with zero width grapheme

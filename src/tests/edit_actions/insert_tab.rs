use crate::{
    action::EditAction::InsertTab,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block},
    display_area::DisplayArea,
    config::{DisplayMode, READ_ONLY_BUFFER_DISPLAY_MODE, /*READ_ONLY_BUFFER, */USE_HARD_TAB, Config},
    keybind::default_keybinds
};
use crate::tests::edit_actions::test_edit_action;

#[test] fn with_multiple_selections(){
    test_edit_action(
        Config{
            user_options: std::collections::HashMap::new(),
            user_commands: std::collections::HashMap::new(),
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        InsertTab, 
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
        if USE_HARD_TAB{"\tsome\n\tshit\n"}else{"    some\n    shit\n"},
        Mode::Insert, 
        if USE_HARD_TAB{
            vec![   //\tsome\n\tshit\n
                //(1, 2, Some(0)),
                Selection::new_unchecked(Range::new(1, 2), None, Some(0)),
                //(7, 8, Some(0))
                Selection::new_unchecked(Range::new(7, 8), None, Some(0)),
            ]
        }else{
            vec![   //    some\n    shit\n      //this would depend on TAB_WIDTH as well...
                //(4, 5, Some(4)),
                Selection::new_unchecked(Range::new(4, 5), None, Some(4)),
                //(13, 14, Some(4))
                Selection::new_unchecked(Range::new(13, 14), None, Some(4)),
            ]
        },
        0,
        ""
    );
}

#[test] fn with_read_only_buffer_is_error(){
    test_edit_action(
        Config{
            user_options: std::collections::HashMap::new(),
            user_commands: std::collections::HashMap::new(),
            semantics: Block, 
            use_full_file_path: false, 
            use_hard_tab: false, 
            tab_width: 4, 
            view_scroll_amount: 1, 
            show_cursor_column: false, 
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        InsertTab, 
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
            DisplayMode::Error => {Mode::Error/*(READ_ONLY_BUFFER.to_string())*/}
            DisplayMode::Warning => {Mode::Warning/*(READ_ONLY_BUFFER.to_string())*/}
            DisplayMode::Notify => {Mode::Notify/*(READ_ONLY_BUFFER.to_string())*/}
            DisplayMode::Info => {Mode::Info/*(READ_ONLY_BUFFER.to_string())*/}
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

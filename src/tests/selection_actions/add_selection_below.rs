use crate::{
    action::SelectionAction::AddSelectionBelow,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block, Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SPANS_MULTIPLE_LINES_DISPLAY_MODE, Config},
    keybind::default_keybinds
};
use crate::tests::selection_actions::test_selection_action;



//to line with same len or more
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn to_line_with_same_len_or_more_with_non_extended_selection_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //|i>d k ⏎      //|i>d k ⏎
                // s o m e ⏎    //|s>o m e ⏎
                // s h i t ⏎    // s h i t ⏎
                //              //
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(0..1, None, 0),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        Selection::new_unchecked(0..1, None, 0),
                        Selection::new_unchecked(4..5, None, 0),
                    ], 
                    0
                );
            }
            //selection direction backward
            #[test] fn to_line_with_same_len_or_more_with_non_extended_selection_with_direction_backward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //<i|d k ⏎      //<i|d k ⏎
                // s o m e ⏎    //<s|o m e ⏎
                // s h i t ⏎    // s h i t ⏎
                //              //
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(0..1, None, 0),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        Selection::new_unchecked(0..1, None, 0),
                        Selection::new_unchecked(4..5, None, 0),
                    ], 
                    0
                );
            }
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn to_line_with_same_len_or_more_with_extended_selection_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //|i d k>⏎      //|i d k>⏎
                // s o m e ⏎    //|s o m>e ⏎
                // s h i t ⏎    // s h i t ⏎
                //              //
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(0..3, Some(Direction::Forward), 2),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        Selection::new_unchecked(0..3, Some(Direction::Forward), 2),
                        Selection::new_unchecked(4..7, Some(Direction::Forward), 2),
                    ], 
                    0
                );
            }
            //selection direction backward
            #[test] fn to_line_with_same_len_or_more_with_extended_selection_with_direction_backward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //<i d k|⏎      //<i d k|⏎
                // s o m e ⏎    //<s o m|e ⏎
                // s h i t ⏎    // s h i t ⏎
                //              //
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(0..3, Some(Direction::Backward), 0),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        Selection::new_unchecked(0..3, Some(Direction::Backward), 0),
                        Selection::new_unchecked(4..7, Some(Direction::Backward), 0),
                    ], 
                    0
                );
            }
    
//to shorter line
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            //selection direction backward
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            //selection direction backward
    
//to empty line
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn to_empty_line_with_non_extended_selection_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      // i d k ⏎
                // s o m e ⏎    // s o m e ⏎
                //|s>h i t ⏎    //|s>h i t ⏎
                //              //| >
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(9..10, None, 0),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        Selection::new_unchecked(9..10, None, 0),
                        Selection::new_unchecked(14..15, None, 0),
                    ], 
                    0
                );
            }
            //selection direction backward
            /*#[ignore] */#[test] fn to_empty_line_with_non_extended_selection_with_direction_backward(){
                //assertion failed "self.anchor() <= buffer.len_chars()"
                //this should only happen with bar semantics or with selection extension in block semantics
                //neither case should apply here...

                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      // i d k ⏎
                // s o m e ⏎    // s o m e ⏎
                //<s|h i t ⏎    //<s|h i t ⏎
                //              //< |
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(9..10, None, 0),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        Selection::new_unchecked(9..10, None, 0),
                        Selection::new_unchecked(14..15, None, 0),
                    ], 
                    0
                );
            }
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn to_empty_line_with_extended_selection_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      // i d k ⏎
                // s o m e ⏎    // s o m e ⏎
                //|s h i t>⏎    //|s h i t>⏎
                //              //| >
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(9..13, Some(Direction::Forward), 3),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        Selection::new_unchecked(9..13, Some(Direction::Forward), 3),
                        Selection::new_unchecked(14..15, None, 0),
                    ], 
                    0
                );
            }
            //selection direction backward
            /*#[ignore] */#[test] fn to_empty_line_with_extended_selection_with_direction_backward(){
                //assertion failed "self.anchor() <= buffer.len_chars()"
                //this should only happen with bar semantics or with selection extension in block semantics
                //neither case should apply here... the new selection is non extended...

                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      // i d k ⏎
                // s o m e ⏎    // s o m e ⏎
                //<s h i t|⏎    //<s h i t|⏎
                //              //< |
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(9..13, Some(Direction::Backward), 0),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        Selection::new_unchecked(9..13, Some(Direction::Backward), 0),
                        Selection::new_unchecked(14..15, None, 0),
                    ], 
                    0
                );
            }
    
//to line with only newline char
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            //selection direction backward
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            //selection direction backward
    
//with multiple selections on same line (should merge overlapping if needed)
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            //selection direction backward
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            //selection direction backward
    
//should error if on bottom line
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn should_error_if_non_extended_selection_with_forward_direction_on_bottom_line(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎
                // s o m e ⏎
                // s h i t ⏎
                //| >
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(14..15, None, 0),
                    ], 
                    0, 
                    1, 
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error},
                        DisplayMode::Warning => {Mode::Warning},
                        DisplayMode::Notify => {Mode::Notify},
                        DisplayMode::Info => {Mode::Info},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        Selection::new_unchecked(14..15, None, 0),
                    ], 
                    0
                );
            }
            //selection direction backward
            /*#[ignore] */#[test] fn should_error_if_non_extended_selection_with_backward_direction_on_bottom_line(){
                //assertion failed "self.anchor() <= buffer.len_chars()"
                //this should only happen with bar semantics or with selection extension in block semantics
                //neither case should apply here...
                
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎
                // s o m e ⏎
                // s h i t ⏎
                //< |
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(14..15, None, 0),
                    ], 
                    0, 
                    1, 
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error},
                        DisplayMode::Warning => {Mode::Warning},
                        DisplayMode::Notify => {Mode::Notify},
                        DisplayMode::Info => {Mode::Info},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        Selection::new_unchecked(14..15, None, 0),
                    ], 
                    0
                );
            }
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn should_error_if_extended_selection_with_forward_direction_on_bottom_line(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3
                // i d k ⏎ s o m e ⏎ s h i t

                // i d k ⏎
                // s o m e ⏎
                //|s h>i t
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit", 
                    vec![
                        Selection::new_unchecked(9..11, Some(Direction::Forward), 1),
                    ], 
                    0, 
                    1, 
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error},
                        DisplayMode::Warning => {Mode::Warning},
                        DisplayMode::Notify => {Mode::Notify},
                        DisplayMode::Info => {Mode::Info},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        Selection::new_unchecked(9..11, Some(Direction::Forward), 1),
                    ], 
                    0
                );
            }
            //selection direction backward
            #[test] fn should_error_if_extended_selection_with_backward_direction_on_bottom_line(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3
                // i d k ⏎ s o m e ⏎ s h i t

                // i d k ⏎
                // s o m e ⏎
                //<s h|i t
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit", 
                    vec![
                        Selection::new_unchecked(9..11, Some(Direction::Backward), 0),
                    ], 
                    0, 
                    1, 
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error},
                        DisplayMode::Warning => {Mode::Warning},
                        DisplayMode::Notify => {Mode::Notify},
                        DisplayMode::Info => {Mode::Info},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        Selection::new_unchecked(9..11, Some(Direction::Backward), 0),
                    ], 
                    0
                );
            }
    
//should error if any selection is multiline
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            //selection direction backward
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn should_error_if_any_selection_is_multiline_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //|i d k ⏎
                // s o m e ⏎>
                // s h i t ⏎
                //
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(0..9, Some(Direction::Forward), 4),
                    ], 
                    0, 
                    1, 
                    match SPANS_MULTIPLE_LINES_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error},
                        DisplayMode::Warning => {Mode::Warning},
                        DisplayMode::Notify => {Mode::Notify},
                        DisplayMode::Info => {Mode::Info},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        Selection::new_unchecked(0..9, Some(Direction::Forward), 4),
                    ], 
                    0
                );
            }
            //selection direction backward
            #[test] fn should_error_if_any_selection_is_multiline_with_direction_backward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //<i d k ⏎
                // s o m e ⏎|
                // s h i t ⏎
                //
                test_selection_action(
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
                    AddSelectionBelow, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        Selection::new_unchecked(0..9, Some(Direction::Backward), 0),
                    ], 
                    0, 
                    1, 
                    match SPANS_MULTIPLE_LINES_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error},
                        DisplayMode::Warning => {Mode::Warning},
                        DisplayMode::Notify => {Mode::Notify},
                        DisplayMode::Info => {Mode::Info},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        Selection::new_unchecked(0..9, Some(Direction::Backward), 0),
                    ], 
                    0
                );
            }

#[ignore] #[test] fn with_multiple_selections_on_primary_cursor_line(){
    todo!()
}

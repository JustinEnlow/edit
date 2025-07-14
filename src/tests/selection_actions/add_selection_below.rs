use crate::{
    application::{SelectionAction::AddSelectionBelow, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, SPANS_MULTIPLE_LINES_DISPLAY_MODE, SPANS_MULTIPLE_LINES, Config}
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(0, 1, None)
                        Selection::new_unchecked(Range::new(0, 1), None, None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(0, 1, None),
                        Selection::new_unchecked(Range::new(0, 1), None, None),
                        //(4, 5, None)
                        Selection::new_unchecked(Range::new(4, 5), None, None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(1, 0, None)
                        Selection::new_unchecked(Range::new(0, 1), None, None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(1, 0, None),
                        Selection::new_unchecked(Range::new(0, 1), None, None),
                        //(5, 4, None)
                        Selection::new_unchecked(Range::new(4, 5), None, None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(0, 3, None)
                        Selection::new_unchecked(Range::new(0, 3), Some(Direction::Forward), None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(0, 3, None),
                        Selection::new_unchecked(Range::new(0, 3), Some(Direction::Forward), None),
                        //(4, 7, None)
                        Selection::new_unchecked(Range::new(4, 7), Some(Direction::Forward), None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(3, 0, None)
                        Selection::new_unchecked(Range::new(0, 3), Some(Direction::Backward), None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(3, 0, None),
                        Selection::new_unchecked(Range::new(0, 3), Some(Direction::Backward), None),
                        //(7, 4, None)
                        Selection::new_unchecked(Range::new(4, 7), Some(Direction::Backward), None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(9, 10, None)
                        Selection::new_unchecked(Range::new(9, 10), None, None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(9, 10, None),
                        Selection::new_unchecked(Range::new(9, 10), None, None),
                        //(14, 15, None)
                        Selection::new_unchecked(Range::new(14, 15), None, None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(10, 9, None)
                        Selection::new_unchecked(Range::new(9, 10), None, None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(10, 9, None),
                        Selection::new_unchecked(Range::new(9, 10), None, None),
                        //(15, 14, None)
                        Selection::new_unchecked(Range::new(14, 15), None, None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(9, 13, None)
                        Selection::new_unchecked(Range::new(9, 13), Some(Direction::Forward), None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(9, 13, None),
                        Selection::new_unchecked(Range::new(9, 13), Some(Direction::Forward), None),
                        //(14, 15, None)
                        Selection::new_unchecked(Range::new(14, 15), None, None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(13, 9, None)
                        Selection::new_unchecked(Range::new(9, 13), Some(Direction::Backward), None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(13, 9, None),
                        Selection::new_unchecked(Range::new(9, 13), Some(Direction::Backward), None),
                        //(15, 14, None)
                        Selection::new_unchecked(Range::new(14, 15), None, None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(14, 15, None)
                        Selection::new_unchecked(Range::new(14, 15), None, None),
                    ], 
                    0, 
                    1, 
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(14, 15, None)
                        Selection::new_unchecked(Range::new(14, 15), None, None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(15, 14, None)
                        Selection::new_unchecked(Range::new(14, 15), None, None),
                    ], 
                    0, 
                    1, 
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(15, 14, None)
                        Selection::new_unchecked(Range::new(14, 15), None, None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit", 
                    vec![
                        //(9, 11, None)
                        Selection::new_unchecked(Range::new(9, 11), Some(Direction::Forward), None),
                    ], 
                    0, 
                    1, 
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(9, 11, None)
                        Selection::new_unchecked(Range::new(9, 11), Some(Direction::Forward), None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit", 
                    vec![
                        //(11, 9, None)
                        Selection::new_unchecked(Range::new(9, 11), Some(Direction::Backward), None),
                    ], 
                    0, 
                    1, 
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(11, 9, None)
                        Selection::new_unchecked(Range::new(9, 11), Some(Direction::Backward), None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(0, 9, None)
                        Selection::new_unchecked(Range::new(0, 9), Some(Direction::Forward), None),
                    ], 
                    0, 
                    1, 
                    match SPANS_MULTIPLE_LINES_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SPANS_MULTIPLE_LINES.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SPANS_MULTIPLE_LINES.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SPANS_MULTIPLE_LINES.to_string())},
                        DisplayMode::Info => {Mode::Info(SPANS_MULTIPLE_LINES.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(0, 9, None)
                        Selection::new_unchecked(Range::new(0, 9), Some(Direction::Forward), None),
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
                        show_cursor_line: false
                    },
                    AddSelectionBelow, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(9, 0, None)
                        Selection::new_unchecked(Range::new(0, 9), Some(Direction::Backward), None),
                    ], 
                    0, 
                    1, 
                    match SPANS_MULTIPLE_LINES_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SPANS_MULTIPLE_LINES.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SPANS_MULTIPLE_LINES.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SPANS_MULTIPLE_LINES.to_string())},
                        DisplayMode::Info => {Mode::Info(SPANS_MULTIPLE_LINES.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(9, 0, None)
                        Selection::new_unchecked(Range::new(0, 9), Some(Direction::Backward), None),
                    ], 
                    0
                );
            }

#[ignore] #[test] fn with_multiple_selections_on_primary_cursor_line(){
    todo!()
}

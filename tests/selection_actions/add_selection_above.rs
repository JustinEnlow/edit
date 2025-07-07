use edit::{
    application::{SelectionAction::AddSelectionAbove, Mode},
    range::Range,
    selection::{Selection, CursorSemantics::Block, /*Extension*/Direction},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::selection_actions::test_selection_action;



//to line with same len or more
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn to_line_with_same_len_or_more_with_non_extended_selection_with_direction_forward(){
                // i d k \n         // i d k \n
                // s o m e \n       //|s>o m e \n
                //|s>h i t \n       //|s>h i t \n
                //                  //
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
                    AddSelectionAbove, 
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
                        //(4, 5, None),
                        Selection::new_unchecked(Range::new(4, 5), None, None),
                        //(9, 10, None)
                        Selection::new_unchecked(Range::new(9, 10), None, None),
                    ], 
                    1
                );
            }
            //selection direction backward
            #[test] fn to_line_with_same_len_or_more_with_non_extended_selection_with_direction_backward(){
                // i d k \n         // i d k \n
                // s o m e \n       //<s|o m e \n
                //<s|h i t \n       //<s|h i t \n
                //                  //
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
                    AddSelectionAbove, 
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
                        //(5, 4, None),
                        Selection::new_unchecked(Range::new(4, 5), None, None),
                        //(10, 9, None)
                        Selection::new_unchecked(Range::new(9, 10), None, None),
                    ], 
                    1
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

                // i d k ⏎      // i d k ⏎
                // s o m e ⏎    //|s o m e>⏎
                //|s h i t>⏎    //|s h i t>⏎
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
                    AddSelectionAbove, 
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
                        //(4, 8, None),
                        Selection::new_unchecked(Range::new(4, 8), Some(Direction::Forward), None),
                        //(9, 13, None)
                        Selection::new_unchecked(Range::new(9, 13), Some(Direction::Forward), None),
                    ], 
                    1
                );
            }
            //selection direction backward
            #[test] fn to_line_with_same_len_or_more_with_extended_selection_with_direction_backward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      // i d k ⏎
                // s o m e ⏎    //<s o m e|⏎
                //<s h i t|⏎    //<s h i t|⏎
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
                    AddSelectionAbove, 
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
                        //(8, 4, None),
                        Selection::new_unchecked(Range::new(4, 8), Some(Direction::Backward), None),
                        //(13, 9, None)
                        Selection::new_unchecked(Range::new(9, 13), Some(Direction::Backward), None),
                    ], 
                    1
                );
            }

//to shorter line
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn to_shorter_line_with_non_extended_selection_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      //|i>d k ⏎
                //|s>o m e ⏎    //|s>o m e ⏎
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
                    AddSelectionAbove, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(4, 5, None)
                        Selection::new_unchecked(Range::new(4, 5), None, None),
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
                    1
                );
            }
            //selection direction backward
            #[test] fn to_shorter_line_with_non_extended_selection_with_direction_backward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      //<i|d k ⏎
                //<s|o m e ⏎    //<s|o m e ⏎
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
                    AddSelectionAbove, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(5, 4, None)
                        Selection::new_unchecked(Range::new(4, 5), None, None),
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
                    1
                );
            }
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn to_shorter_line_with_extended_selection_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      //|i d k ⏎>
                //|s o m e ⏎>   //|s o m e ⏎>
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
                    AddSelectionAbove, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(4, 9, None)
                        Selection::new_unchecked(Range::new(4, 9), Some(Direction::Forward), None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(0, 4, None),
                        Selection::new_unchecked(Range::new(0, 4), Some(Direction::Forward), None),
                        //(4, 9, None)
                        Selection::new_unchecked(Range::new(4, 9), Some(Direction::Forward), None),
                    ], 
                    1
                );
            }
            //selection direction backward
            #[test] fn to_shorter_line_with_extended_selection_with_direction_backward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                // i d k ⏎      //<i d k ⏎|
                //<s o m e ⏎|   //<s o m e ⏎|
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
                    AddSelectionAbove, 
                    //Block, 
                    false, 
                    false, 
                    DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
                    "idk\nsome\nshit\n", 
                    vec![
                        //(9, 4, None)
                        Selection::new_unchecked(Range::new(4, 9), Some(Direction::Backward), None),
                    ], 
                    0, 
                    1, 
                    Mode::Insert, 
                    vec![
                        //(4, 0, None),
                        Selection::new_unchecked(Range::new(0, 4), Some(Direction::Backward), None),
                        //(9, 4, None)
                        Selection::new_unchecked(Range::new(4, 9), Some(Direction::Backward), None),
                    ], 
                    1
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

//should error if on top line
    //non extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn should_error_if_on_top_line_with_non_extended_selection_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //|i>d k ⏎
                // s o m e ⏎
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
                    AddSelectionAbove, 
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
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(0, 1, None)
                        Selection::new_unchecked(Range::new(0, 1), None, None),
                    ], 
                    0
                );
            }
            //selection direction backward
    //extended
        //bar
            //selection direction forward
            //selection direction backward
        //block
            //selection direction forward
            #[test] fn should_error_if_on_top_line_with_extended_selection_with_direction_forward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //|i d k>⏎
                // s o m e ⏎
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
                    AddSelectionAbove, 
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
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(0, 3, None)
                        Selection::new_unchecked(Range::new(0, 3), Some(Direction::Forward), None),
                    ], 
                    0
                );
            }
            //selection direction backward
            #[test] fn should_error_if_on_top_line_with_extended_selection_with_direction_backward(){
                //0 1 2 3 4 5 6 7 8 9 0 1 2 3 4
                // i d k ⏎ s o m e ⏎ s h i t ⏎

                //<i d k|⏎
                // s o m e ⏎
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
                    AddSelectionAbove, 
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
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
                        DisplayMode::Ignore => {Mode::Insert},
                    }, 
                    vec![
                        //(3, 0, None)
                        Selection::new_unchecked(Range::new(0, 3), Some(Direction::Backward), None),
                    ], 
                    0
                );
            }

//should error if any selection is multiline
    //non extended
        //block
            //selection direction forward
            //selection direction backward
    //extended
        //block
            //selection direction forward
            #[test] fn should_error_if_any_selection_is_multiline_with_direction_forward(){
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
                    AddSelectionAbove, 
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
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
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
                    AddSelectionAbove, 
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
                    match SAME_STATE_DISPLAY_MODE{
                        DisplayMode::Error => {Mode::Error(SAME_STATE.to_string())},
                        DisplayMode::Warning => {Mode::Warning(SAME_STATE.to_string())},
                        DisplayMode::Notify => {Mode::Notify(SAME_STATE.to_string())},
                        DisplayMode::Info => {Mode::Info(SAME_STATE.to_string())},
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
    unimplemented!()
}

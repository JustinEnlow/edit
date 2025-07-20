//TODO: rename to NearestSurroundingPair?...

use crate::{
    application::SelectionAction::SurroundingPair,
    mode::Mode,
    range::Range,
    selection::{Selection, CursorSemantics::Block},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE, Config}
};
use crate::tests::selection_actions::test_selection_action;



#[test] fn with_multiple_selections(){
    //                     1                   2
    // 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7
    //|i>d k ( s|o>m e|[>] _ t h|i>n g _ {|}>e l s e ) _ i|d>k
    //|i>d k|(>s o m e|[>]>_ t h i n g _|{>}>e l s e|)>_ i|d>k
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            //(0, 1, None),   //no pair
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(5, 6, None),   //pair
            Selection::new_unchecked(Range::new(5, 6), None, None),
            //(8, 9, None),   //pair
            Selection::new_unchecked(Range::new(8, 9), None, None),
            //(13, 14, None), //pair
            Selection::new_unchecked(Range::new(13, 14), None, None),
            //(18, 19, None), //pair
            Selection::new_unchecked(Range::new(18, 19), None, None),
            //(26, 27, None)  //no pair
            Selection::new_unchecked(Range::new(26, 27), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(0, 1, None),
            Selection::new_unchecked(Range::new(0, 1), None, None),
            //(3, 4, Some(3)),    //idk why these have stored line position and others don't
            Selection::new_unchecked(Range::new(3, 4), None, Some(3)),
            //(8, 9, None),
            Selection::new_unchecked(Range::new(8, 9), None, None),
            //(9, 10, None),
            Selection::new_unchecked(Range::new(9, 10), None, None),
            //(17, 18, None),
            Selection::new_unchecked(Range::new(17, 18), None, None),
            //(18, 19, None),
            Selection::new_unchecked(Range::new(18, 19), None, None),
            //(23, 24, Some(23)), //idk why these have stored line position and others don't
            Selection::new_unchecked(Range::new(23, 24), None, Some(23)),
            //(26, 27, None)
            Selection::new_unchecked(Range::new(26, 27), None, None),
            //TODO: merge overlapping in selection.rs causing the stored line position. only the overlapping selections have it
            //if so, this should def be fixed in merge_overlapping impl
            //or more correctly, every movement fn should update the stored line position...
                //the only reason we have a None variant is so that we don't need to take a &Rope in Selection::new()
        ], 
        0
    );
}

//|i>d k ( s o m e [ ] _ t h i n g _ { } e l s e ) _ i d k     //no surrounding pair with cursor at this location
#[test] fn at_start_with_no_surrounding_pair(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
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

// i d k ( s|o>m e [ ] _ t h i n g _ { } e l s e ) _ i d k     //paren surrounding pair with cursor at this location
#[test] fn normal_case(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            //(5, 6, None)
            Selection::new_unchecked(Range::new(5, 6), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(3, 4, None),
            Selection::new_unchecked(Range::new(3, 4), None, None),
            //(23, 24, None)
            Selection::new_unchecked(Range::new(23, 24), None, None),
        ], 
        0
    );
}

// i d k ( s o m e|[>] _ t h i n g _ { } e l s e ) _ i d k     //square bracket surrounding pair with cursor at this location
#[test] fn with_cursor_over_surrounding_pair_opening(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            //(8, 9, None)
            Selection::new_unchecked(Range::new(8, 9), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(8, 9, None),
            Selection::new_unchecked(Range::new(8, 9), None, None),
            //(9, 10, None)
            Selection::new_unchecked(Range::new(9, 10), None, None),
        ], 
        0
    );
}

// i d k ( s o m e [ ] _ t h|i>n g _ { } e l s e ) _ i d k     //paren surrounding pair with cursor at this location
#[test] fn with_other_pairs_inside_surrounding_pair(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            //(13, 14, None)
            Selection::new_unchecked(Range::new(13, 14), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(3, 4, None),
            Selection::new_unchecked(Range::new(3, 4), None, None),
            //(23, 24, None)
            Selection::new_unchecked(Range::new(23, 24), None, None),
        ], 
        0
    );
}

// i d k ( s o m e [ ] _ t h i n g _ {|}>e l s e ) _ i d k     //curly bracket surrounding pair with cursor at this location
#[test] fn with_cursor_over_surrounding_pair_closing(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            //(18, 19, None)
            Selection::new_unchecked(Range::new(18, 19), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(17, 18, None),
            Selection::new_unchecked(Range::new(17, 18), None, None),
            //(18, 19, None)
            Selection::new_unchecked(Range::new(18, 19), None, None),
        ], 
        0
    );
}

// i d k ( s o m e [ ] _ t h i n g _ { } e l s e ) _ i|d>k     //no surrounding pair with cursor at this location
#[test] fn at_end_with_no_surrounding_pair(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            //(26, 27, None)
            Selection::new_unchecked(Range::new(26, 27), None, None),
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
            //(26, 27, None)
            Selection::new_unchecked(Range::new(26, 27), None, None),
        ], 
        0
    );
}

//These two seem redundant given previous tests...
#[test] fn no_opening_bracket_pair_returns_empty_vec(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsomething)\n", 
        vec![
            //(3, 4, None)
            Selection::new_unchecked(Range::new(3, 4), None, None),
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
            //(3, 4, None)
            Selection::new_unchecked(Range::new(3, 4), None, None),
        ], 
        0
    );
}
#[test] fn no_closing_bracket_pair_returns_empty_vec(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "(idk\nsomething\n", 
        vec![
            //(3, 4, None)
            Selection::new_unchecked(Range::new(3, 4), None, None),
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
            //(3, 4, None)
            Selection::new_unchecked(Range::new(3, 4), None, None),
        ], 
        0
    );
}

////idk(some()t(h(i)n)g()else)    //test from multiple levels of same surrounding pair
#[test] fn with_multiple_levels_of_same_surrounding_pair(){
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
        SurroundingPair, 
        //Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some()t(h(i)n)g()else", 
        vec![
            //(12, 13, None)
            Selection::new_unchecked(Range::new(12, 13), None, None),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            //(11, 12, None),
            Selection::new_unchecked(Range::new(11, 12), None, None),
            //(17, 18, None)
            Selection::new_unchecked(Range::new(17, 18), None, None),
        ], 
        0
    );
}

//TODO: impl test with expected quote pair behavior
//note: quote pairs may have to work differently than bracket pairs
//#[test] fn with_same_surrounding_pair_opening_and_closing(){
//    //idk"some""t"h"i"n"g""else"
//    let text = Rope::from("idk\"some\"\"t\"h\"i\"n\"g\"\"else");
//    let selection = Selection::new(Range::new(12, 13), Direction::Forward);
//    assert_eq!(
//        vec![
//            Selection::new(Range::new(11, 12), Direction::Forward),
//            //Selection::new(Range::new(17, 18), Direction::Forward)
//            Selection::new(Range::new(13, 14), Direction::Forward)
//        ],
//        selection.nearest_surrounding_pair(&text)
//    );
//}

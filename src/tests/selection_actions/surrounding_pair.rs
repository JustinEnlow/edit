//TODO: rename to NearestSurroundingPair?...

use crate::{
    action::SelectionAction::SurroundingPair,
    mode::Mode,
    selection::{Selection, CursorSemantics::Block},
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, Config},
    keybind::default_keybinds
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            //no pair
            Selection::new_unchecked(0..1, None, 0),
            //pair
            Selection::new_unchecked(5..6, None, 5),
            //pair
            Selection::new_unchecked(8..9, None, 8),
            //pair
            Selection::new_unchecked(13..14, None, 13),
            //pair
            Selection::new_unchecked(18..19, None, 18),
            //no pair
            Selection::new_unchecked(26..27, None, 26),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(0..1, None, 0),
            Selection::new_unchecked(3..4, None, 3),
            Selection::new_unchecked(8..9, None, 8),
            Selection::new_unchecked(9..10, None, 9),
            Selection::new_unchecked(17..18, None, 17),
            Selection::new_unchecked(18..19, None, 18),
            Selection::new_unchecked(23..24, None, 23),
            Selection::new_unchecked(26..27, None, 26),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            Selection::new_unchecked(0..1, None, 0),
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
            Selection::new_unchecked(0..1, None, 0),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            Selection::new_unchecked(5..6, None, 5),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(3..4, None, 3),
            Selection::new_unchecked(23..24, None, 23),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            Selection::new_unchecked(8..9, None, 8),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(8..9, None, 8),
            Selection::new_unchecked(9..10, None, 9),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            Selection::new_unchecked(13..14, None, 13),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(3..4, None, 3),
            Selection::new_unchecked(23..24, None, 23),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            Selection::new_unchecked(18..19, None, 18),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(17..18, None, 17),
            Selection::new_unchecked(18..19, None, 18),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some[] thing {}else) idk", 
        vec![
            Selection::new_unchecked(26..27, None, 26),
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
            Selection::new_unchecked(26..27, None, 26),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsomething)\n", 
        vec![
            Selection::new_unchecked(3..4, None, 3),
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
            Selection::new_unchecked(3..4, None, 3),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "(idk\nsomething\n", 
        vec![
            Selection::new_unchecked(3..4, None, 3),
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
            Selection::new_unchecked(3..4, None, 3),
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
            show_cursor_line: false,
            keybinds: default_keybinds()
        },
        SurroundingPair, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk(some()t(h(i)n)g()else", 
        vec![
            Selection::new_unchecked(12..13, None, 12),
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            Selection::new_unchecked(11..12, None, 11),
            Selection::new_unchecked(17..18, None, 17),
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

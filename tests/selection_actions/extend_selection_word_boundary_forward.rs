use edit::{
    application::{SelectionAction::ExtendSelectionWordBoundaryForward, Mode},
    selection::CursorSemantics::Block,
    display_area::DisplayArea,
    config::{DisplayMode, SAME_STATE_DISPLAY_MODE, SAME_STATE}
};
use crate::selection_actions::test_selection_action;



#[test] fn with_multiple_valid_selections(){
    //test::selection_movement_with_count(
    //    extend_selection_word_boundary_forward::application_impl,
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    vec![
    //        (0, 1, None),
    //        (4, 5, None)
    //    ], 0, 
    //    1,
    //    None,
    //    vec![
    //        (0, 3, Some(2)),
    //        (4, 8, Some(3))
    //    ], 0
    //);
    test_selection_action(
        ExtendSelectionWordBoundaryForward, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),
            (4, 5, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 3, Some(2)),
            (4, 8, Some(3))
        ], 
        0
    );
}
#[test] fn with_mixed_valid_and_invalid_selections(){
    //test::selection_movement_with_count(
    //    extend_selection_word_boundary_forward::application_impl,
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    vec![
    //        (0, 1, None),
    //        (13, 14, None)
    //    ], 0, 
    //    1,
    //    None,
    //    vec![
    //        (0, 3, Some(2)),
    //        (13, 14, None)
    //    ], 0
    //);
    test_selection_action(
        ExtendSelectionWordBoundaryForward, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None),
            (13, 14, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 3, Some(2)),
            (13, 14, None)
        ], 
        0
    );
}
    
#[test] fn extends_to_doc_text_end_if_no_other_word_boundaries(){
    //test(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit    \n", 
    //    None, 
    //    (12, 13, None), 
    //    1, 
    //    (12, 18, Some(8))
    //);
    test_selection_action(
        ExtendSelectionWordBoundaryForward, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit    \n", 
        vec![
            (12, 13, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (12, 18, Some(8))
        ], 
        0
    );
}
//TODO: shrinks previously backward extended
//TODO: test with cursor over word boundary
    
//should error if single selection at doc end

#[test] fn normal_use_block_semantics(){
    //test(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    None, 
    //    (0, 1, None), 
    //    1, 
    //    (0, 3, Some(2))
    //);
    test_selection_action(
        ExtendSelectionWordBoundaryForward, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (0, 3, Some(2))
        ], 
        0
    );
}
    
#[test] fn extends_to_doc_end_from_doc_text_end_block_semantics(){  //i don't think this should actually work...
    //test(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    None, 
    //    (12, 13, None), 
    //    1, 
    //    (12, 14, Some(4))
    //);
    test_selection_action(
        ExtendSelectionWordBoundaryForward, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (12, 13, None)
        ], 
        0, 
        1, 
        Mode::Insert, 
        vec![
            (12, 14, Some(4))
        ], 
        0
    );
}

#[test] fn errors_if_cursor_at_doc_end_block_semantics(){
    //test_error(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    None, 
    //    (13, 14, None), 
    //    1
    //);
    test_selection_action(
        ExtendSelectionWordBoundaryForward, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (13, 14, None)
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
            (13, 14, None)
        ], 
        0
    );
}

#[test] fn errors_if_already_extended_forward_to_doc_end_block_semantics(){
    //test_error(
    //    CursorSemantics::Block, 
    //    "idk\nsome\nshit\n", 
    //    None, 
    //    (0, 14, None), 
    //    1
    //);
    test_selection_action(
        ExtendSelectionWordBoundaryForward, 
        Block, 
        false, 
        false, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50}, 
        "idk\nsome\nshit\n", 
        vec![
            (0, 14, None)
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
            (0, 14, None)
        ], 
        0
    );
}

//TODO: actually, this should work... it should move the cursor from 0 to 3...
//#[test] fn errors_if_already_extended_backward_from_doc_end_bar_semantics(){
//    test::error_selection_movement_with_count(
//        extend_selection_word_boundary_forward::application_impl,
//        CursorSemantics::Bar, 
//        "idk\nsome\nshit\n", 
//        vec![
//            (14, 0, None)
//        ], 0,
//        1,
//        None
//    );
//}
//#[test] fn errors_if_already_extended_backward_from_doc_end_block_semantics(){
//    test::error_selection_movement_with_count(
//        extend_selection_word_boundary_forward::application_impl,
//        CursorSemantics::Block, 
//        "idk\nsome\nshit\n", 
//        vec![
//            (14, 0, None)
//        ], 0,
//        1,
//        None
//    );
//}

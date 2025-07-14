//! since new_from_range calls assert_invariants(), this is also testing that fn
//! also tests selection.direction(), selection.is_extended(), and selection.cursor()     //TODO: test selection.anchor() and selection.head()

use crate::{
    range::Range,
    selection::*,
    buffer::Buffer,
};

//TODO: can we catch a range index being in the middle of a unicode grapheme? maybe panic if so...

#[should_panic] #[test] fn non_extended_block_cursor_should_panic_if_set_to_extension_direction_forward(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Block;
    let _ = Selection::new_from_range(Range::new(14, 15), Some(Direction::Forward), buffer, semantics.clone());
}
#[should_panic] #[test] fn non_extended_block_cursor_should_panic_if_set_to_extension_direction_backward(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Block;
    let _ = Selection::new_from_range(Range::new(14, 15), Some(Direction::Backward), buffer, semantics.clone());
}
#[should_panic] #[test] fn extended_bar_selection_should_panic_if_set_to_extension_direction_none(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Bar;
    let _ = Selection::new_from_range(Range::new(0, 3), None, buffer, semantics.clone());
}
#[test] #[should_panic] fn zero_width_block_selection_panics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Block;
    let _ = Selection::new_from_range(Range::new(0, 0), None, buffer, semantics);
}
#[test] #[should_panic] fn index_past_buffer_len_panics_bar_semantics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Bar;
    let _ = Selection::new_from_range(Range::new(15, 15), None, buffer, semantics);
}
#[test] #[should_panic] fn index_past_buffer_len_panics_block_semantics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Block;
    let _ = Selection::new_from_range(Range::new(15, 16), None, buffer, semantics);
}
#[test] fn non_extended_bar_semantics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Bar;
    let idk = Selection::new_from_range(Range::new(0, 0), None, buffer, semantics.clone());
    assert_eq!(0, idk.range.start);
    assert_eq!(0, idk.range.end);
    assert_eq!(0, idk.cursor(buffer, semantics.clone()));
    assert_eq!(None, idk.stored_line_offset);
    assert_eq!(None, idk.extension_direction);
    assert_eq!(None, idk.direction(buffer, semantics.clone()));
    assert_eq!(false, idk.is_extended());
}
#[test] fn non_extended_block_semantics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Block;
    let idk = Selection::new_from_range(Range::new(0, 1), None, buffer, semantics.clone());
    assert_eq!(0, idk.range.start);
    assert_eq!(1, idk.range.end);
    assert_eq!(0, idk.cursor(buffer, semantics.clone()));
    assert_eq!(None, idk.stored_line_offset);
    assert_eq!(None, idk.extension_direction);
    assert_eq!(None, idk.direction(buffer, semantics.clone()));
    assert_eq!(false, idk.is_extended());
}
#[should_panic]
#[test] fn backward_extended_bar_semantics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Bar;
    let idk = Selection::new_from_range(Range::new(1, 0), Some(Direction::Backward), buffer, semantics.clone());
    assert_eq!(0, idk.range.start);
    assert_eq!(1, idk.range.end);
    assert_eq!(0, idk.cursor(buffer, semantics.clone()));
    assert_eq!(None, idk.stored_line_offset);
    assert_eq!(Some(Direction::Backward), idk.extension_direction);
    assert_eq!(Some(Direction::Backward), idk.direction(buffer, semantics.clone()));
    assert_eq!(true, idk.is_extended());
}
#[test] fn backward_extended_block_semantics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Block;
    let idk = Selection::new_from_range(Range::new(0, 2), Some(Direction::Backward), buffer, semantics.clone());
    assert_eq!(0, idk.range.start);
    assert_eq!(2, idk.range.end);
    assert_eq!(0, idk.cursor(buffer, semantics.clone()));
    assert_eq!(None, idk.stored_line_offset);
    assert_eq!(Some(Direction::Backward), idk.extension_direction);
    assert_eq!(Some(Direction::Backward), idk.direction(buffer, semantics.clone()));
    assert_eq!(true, idk.is_extended());
}
#[test] fn forward_extended_bar_semantics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Bar;
    let idk = Selection::new_from_range(Range::new(0, 1), Some(Direction::Forward), buffer, semantics.clone());
    assert_eq!(0, idk.range.start);
    assert_eq!(1, idk.range.end);
    assert_eq!(1, idk.cursor(buffer, semantics.clone()));
    assert_eq!(None, idk.stored_line_offset);
    assert_eq!(Some(Direction::Forward), idk.extension_direction);
    assert_eq!(Some(Direction::Forward), idk.direction(buffer, semantics.clone()));
    assert_eq!(true, idk.is_extended());
}
#[test] fn forward_extended_block_semantics(){
    let buffer = &Buffer::new("idk\nsome\nshit\n", None, false);
    let semantics = crate::selection::CursorSemantics::Block;
    let idk = Selection::new_from_range(Range::new(0, 2), Some(Direction::Forward), buffer, semantics.clone());
    assert_eq!(0, idk.range.start);
    assert_eq!(2, idk.range.end);
    assert_eq!(1, idk.cursor(buffer, semantics.clone()));
    assert_eq!(None, idk.stored_line_offset);
    assert_eq!(Some(Direction::Forward), idk.extension_direction);
    assert_eq!(Some(Direction::Forward), idk.direction(buffer, semantics.clone()));
    assert_eq!(true, idk.is_extended());
}



//utf-8
//#[test] fn utf_8_non_extended_block_semantics(){
//    let buffer = &Buffer::new("→idk\nsome\nshit\n", None, false);
//    let semantics = crate::selection::CursorSemantics::Block;
//    let idk = Selection::new_from_range(Range::new(0, 3), None, buffer, semantics.clone());
//    assert_eq!(0, idk.range.start);
//    assert_eq!(0, idk.anchor());
//    assert_eq!(3, idk.range.end);
//    assert_eq!(3, idk.head());
//    assert_eq!(0, idk.cursor(buffer, semantics.clone()), "cursor");
//    assert_eq!(None, idk.stored_line_offset);
//    assert_eq!(None, idk.extension_direction);
//    assert_eq!(None, idk.direction(buffer, semantics.clone()));
//    assert_eq!(false, idk.is_extended());
//}

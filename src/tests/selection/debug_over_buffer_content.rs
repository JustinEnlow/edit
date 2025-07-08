use crate::{
    range::Range,
    selection::{Selection, CursorSemantics, Direction},
    buffer::Buffer
};

#[test] fn non_extended_bar_semantics(){
    let semantics = CursorSemantics::Bar;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(0, 0), None, &buffer, semantics.clone());
    assert_eq!("|>idk\nsome\nshit\n", selection.debug_over_buffer_content(&buffer, semantics));
}
#[test] fn forward_extended_bar_semantics(){
    let semantics = CursorSemantics::Bar;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(2, 6), Some(Direction::Forward), &buffer, semantics.clone());
    assert_eq!("id|k\nso>me\nshit\n", selection.debug_over_buffer_content(&buffer, semantics));
}
#[test] fn backward_extended_bar_semantics(){
    let semantics = CursorSemantics::Bar;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(2, 6), Some(Direction::Backward), &buffer, semantics.clone());
    assert_eq!("id<k\nso|me\nshit\n", selection.debug_over_buffer_content(&buffer, semantics));
}

#[test] fn non_extended_block_semantics(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(0, 1), None, &buffer, semantics.clone());
    assert_eq!("|:i>dk\nsome\nshit\n", selection.debug_over_buffer_content(&buffer, semantics));
}
#[test] fn forward_extended_block_semantics(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(2, 6), Some(Direction::Forward), &buffer, semantics.clone());
    assert_eq!("id|k\ns:o>me\nshit\n", selection.debug_over_buffer_content(&buffer, semantics));
}
#[test] fn backward_extended_block_semantics(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(2, 6), Some(Direction::Backward), &buffer, semantics.clone());
    assert_eq!("id<k\nso|me\nshit\n", selection.debug_over_buffer_content(&buffer, semantics));
}

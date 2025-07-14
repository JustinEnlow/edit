use crate::{
    range::Range,
    selection::{Selection, CursorSemantics, Direction},
    buffer::Buffer,
};


#[test] fn with_ascii_string(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(0, 14), Some(Direction::Forward), &buffer, semantics.clone());
    assert_eq!("idk\nsome\nshit\n".to_string(), selection.to_string(&buffer));
    let selection = Selection::new_from_range(Range::new(0, 4), Some(Direction::Forward), &buffer, semantics.clone());
    assert_eq!("idk\n".to_string(), selection.to_string(&buffer));
    let selection = Selection::new_from_range(Range::new(9, 14), Some(Direction::Forward), &buffer, semantics.clone());
    assert_eq!("shit\n".to_string(), selection.to_string(&buffer));
}

//can we catch a range index being in the middle of a unicode grapheme? maybe panic if so...
#[ignore] #[test] fn with_unicode_grapheme_string(){
    todo!()
}

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



#[test] fn with_selection_at_buffer_end_block_semantics(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(14, 15), None, &buffer, semantics.clone());
    assert_eq!("idk\nsome\nshit\n|: >", selection.debug_over_buffer_content(&buffer, semantics));
}
#[test] fn with_selection_at_buffer_end_bar_semantics(){
    let semantics = CursorSemantics::Bar;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new_from_range(Range::new(14, 14), None, &buffer, semantics.clone());
    assert_eq!("idk\nsome\nshit\n|>", selection.debug_over_buffer_content(&buffer, semantics));
}




#[test] fn with_multibyte_grapheme(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk⏎\n", None, false);
    println!("⏎: byte_count: {}, char_count: {}", '⏎'.len_utf8(), "⏎".chars().count());
    println!("buffer len_chars {}", buffer.len_chars());
    println!("buffer len chars {}", buffer.to_string().chars().count());
    for (i, char) in buffer.chars().enumerate(){
        //next_grapheme_char_index() fn was returning an incorrect value...i think it is fixed now?...
        println!("char index: {}, char: {:?}, char index of next grapheme: {}", i, char, buffer.next_grapheme_char_index(i));
    }
    let selection = Selection::new_from_range(Range::new(3, 4), None, &buffer, semantics.clone());
    
    assert_eq!("idk|:⏎>\n", selection.debug_over_buffer_content(&buffer, semantics));
}
#[test] fn with_wide_grapheme(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk𘀀\n", None, false);
    println!("𘀀: byte_count: {}, char_count: {}", '𘀀'.len_utf8(), "𘀀".chars().count());
    let selection = Selection::new_from_range(Range::new(3, 4), None, &buffer, semantics.clone());
    assert_eq!("idk|:𘀀>\n", selection.debug_over_buffer_content(&buffer, semantics));
}
//TODO: with multichar/multicodepoint grapheme
//TODO: with zero width grapheme
//TODO: with bidirectional text (may skip this, at least for now...)

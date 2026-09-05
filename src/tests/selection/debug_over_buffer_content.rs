use unicode_segmentation::UnicodeSegmentation;

use crate::{
    selection::{Selection, CursorSemantics, Direction},
    buffer::Buffer
};

#[test] fn non_extended_bar_semantics(){
    let semantics = CursorSemantics::Bar;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new(
        0..0,
        None, 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("|>idk\nsome\nshit\n", selection.debug_over_buffer_content(&buffer, semantics, true));
}
#[test] fn forward_extended_bar_semantics(){
    let semantics = CursorSemantics::Bar;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new(
        2..6,
        Some(Direction::Forward), 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("id|k\nso>me\nshit\n", selection.debug_over_buffer_content(&buffer, semantics, true));
}
#[test] fn backward_extended_bar_semantics(){
    let semantics = CursorSemantics::Bar;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new(
        2..6,
        Some(Direction::Backward), 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("id<k\nso|me\nshit\n", selection.debug_over_buffer_content(&buffer, semantics, true));
}

#[test] fn non_extended_block_semantics(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new(
        0..1,
        None, 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("|:i>dk\nsome\nshit\n", selection.debug_over_buffer_content(&buffer, semantics, true));
}
#[test] fn forward_extended_block_semantics(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new(
        2..6,
        Some(Direction::Forward), 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("id|k\ns:o>me\nshit\n", selection.debug_over_buffer_content(&buffer, semantics, true));
}
#[test] fn backward_extended_block_semantics(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new(
        2..6,
        Some(Direction::Backward), 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("id<k\nso|me\nshit\n", selection.debug_over_buffer_content(&buffer, semantics, true));
}



#[test] fn with_selection_at_buffer_end_block_semantics(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new(
        14..15,
        None, 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("idk\nsome\nshit\n|: >", selection.debug_over_buffer_content(&buffer, semantics, true));
}
#[test] fn with_selection_at_buffer_end_bar_semantics(){
    let semantics = CursorSemantics::Bar;
    let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
    let selection = Selection::new(
        14..14,
        None, 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("idk\nsome\nshit\n|>", selection.debug_over_buffer_content(&buffer, semantics, true));
}




#[test] fn with_multibyte_grapheme(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk⏎\n", None, false);
    println!("⏎: byte_count: {}, char_count: {}", "⏎".bytes().count(), "⏎".chars().count());
    println!("buffer len_bytes {}", buffer.len_bytes());
    println!("buffer len chars {}", buffer.to_string().chars().count());
    for (i, grapheme) in buffer.to_string().grapheme_indices(true){
        println!("byte index: {}, grapheme: {:?}, byte index of next grapheme: {}", i, grapheme, buffer.next_grapheme_boundary_byte_offset(i));
    }
    let selection = Selection::new(
        //3..4,
        3..6,
        None, 
        &buffer, 
        semantics.clone()
    );
    
    assert_eq!("idk|:⏎>\n", selection.debug_over_buffer_content(&buffer, semantics, true));
}
#[test] fn with_wide_grapheme(){
    let semantics = CursorSemantics::Block;
    let buffer = Buffer::new("idk𘀀\n", None, false);
    println!("𘀀: byte_count: {}, char_count: {}", "𘀀".bytes().count(), "𘀀".chars().count());
    let selection = Selection::new(
        //3..4,
        3..7,
        None, 
        &buffer, 
        semantics.clone()
    );
    assert_eq!("idk|:𘀀>\n", selection.debug_over_buffer_content(&buffer, semantics, true));
}
//TODO: with multichar/multicodepoint grapheme
//TODO: with zero width grapheme
//TODO: with bidirectional text (may skip this, at least for now...)

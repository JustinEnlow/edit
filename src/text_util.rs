use ropey::{Rope, RopeSlice};
use unicode_segmentation::UnicodeSegmentation;
use crate::document::TAB_WIDTH;
use crate::selection::Selection;

//TODO: should be grapheme count instead of char count?
/// Returns the count of visible graphemes in a line of text.
/// # Example
/// ```
/// # use ropey::Rope;
/// # use edit::text_util;
/// 
/// let text = Rope::from("idk\n");
/// assert!(text_util::line_width_excluding_newline(text.slice(..)) == 3);
/// ```
pub fn line_width_excluding_newline(line: RopeSlice) -> usize{
    let mut line_width = 0;
    let line = line.to_string();
    //for char in line.chars(){
        //if char != '\n'{
    for grapheme in line.graphemes(true){
        if grapheme != "\n"{
            line_width = line_width + 1;
        }
    }
    line_width
}

//TODO: handle graphemes instead of chars?
/// Returns the offset of the first non whitespace grapheme from the start of a line of text.
/// # Example
/// ```
/// # use ropey::Rope;
/// # use edit::text_util;
/// 
/// let text = Rope::from("   idk\n");
/// assert!(text_util::first_non_whitespace_character_offset(text.slice(..)) == 3);
/// 
/// let text = Rope::from("");
/// assert!(text_util::first_non_whitespace_character_offset(text.slice(..)) == 0);
/// 
/// let text = Rope::from("   ");
/// assert!(text_util::first_non_whitespace_character_offset(text.slice(..)) == 0);
/// ```
pub fn first_non_whitespace_character_offset(line: RopeSlice) -> usize{
    let line = line.to_string();
    
    //if line.len_chars() == 0{return 0;}
    if line.is_empty(){return 0;}

    //for (index, char) in line.chars().enumerate(){
    for (index, grapheme) in line.graphemes(true).enumerate(){
        //if char != ' '{return index;}
        if grapheme != " "{return index;}
    }

    0
}

/// Returns true if slice contains only spaces.
/// # Example
/// ```
/// # use edit::text_util;
/// 
/// let text = String::from("    ");
/// assert!(text_util::slice_is_all_spaces(&text, 0, 3));
/// 
/// let text = String::from(" idk ");
/// assert!(!text_util::slice_is_all_spaces(&text, 0, 4));
/// ```
pub fn slice_is_all_spaces(line: &str, start_of_slice: usize, end_of_slice: usize) -> bool{
    for grapheme in line[start_of_slice..end_of_slice].graphemes(true){
        if grapheme != " "{
            return false;
        }
    }

    true
}

/// Returns the grapheme distance to next multiple of user defined tab width.
/// # Example
/// ```
/// # use ropey::Rope;
/// # use edit::document::TAB_WIDTH;
/// # use edit::selection::Selection;
/// # use edit::text_util;
/// 
/// let mut tab = String::new();
/// for _ in 0..TAB_WIDTH{
///     tab.push(' ');
/// }
/// let text = Rope::from(format!("{}idk\n", tab));
/// let selection = Selection::new(1, 1);
/// let distance = text_util::distance_to_next_multiple_of_tab_width(selection, &text);
/// assert!(distance == 3);
/// ```
pub fn distance_to_next_multiple_of_tab_width(selection: Selection, text: &Rope) -> usize{
    //if cursor.stored_line_position() % TAB_WIDTH != 0{
    if offset_from_line_start(selection.head(), text) % TAB_WIDTH != 0{ //TODO: should this use selection.cursor() instead?
        //TAB_WIDTH - (cursor.stored_line_position() % TAB_WIDTH)
        TAB_WIDTH - (offset_from_line_start(selection.head(), text) % TAB_WIDTH)
    }else{
        0
    }
}

/// Returns the offset of cursor position from the start of a line of text.
/// # Example
/// ```
/// # use ropey::Rope;
/// # use edit::selection::Selection;
/// # use edit::text_util;
/// 
/// let text = Rope::from("idk\nsome\nshit\n");
/// let selection = Selection::new(2, 2);
/// assert!(text_util::offset_from_line_start(selection.head(), &text) == 2);
/// ```
// TODO: maybe this really does belong in [Selection] in selection.rs?
pub fn offset_from_line_start(point: usize, text: &Rope) -> usize{
    let line_start = text.line_to_char(text.char_to_line(point));
    point.saturating_sub(line_start)
}

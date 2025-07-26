// This should probably use a zed editor style rope, built on a sum_tree

use unicode_segmentation::UnicodeSegmentation;
use std::path::PathBuf;
use ropey::Rope;
//
use crate::{
    selection::{Selection, CursorSemantics},
    history::{Change, Operation},
};
//

/// Abstraction over a stringy data type, to allow for the underlying data type to be changed as desired
// passing this structure as a reference has no added cost compared to passing inner as a reference. they are both just the architecture pointer size
#[derive(Clone, Debug, PartialEq)]
pub struct Buffer{
    inner: Rope, 
    pub file_path: Option<PathBuf>,  //None if scratch buffer, Some(path) if from file
    pub read_only: bool
}
impl Buffer{
    //TODO?: maybe new() should only take a &str, and we could make .with_file_path() and .read_only() builder methods...
    pub fn new(str: &str, file_path: Option<PathBuf>, read_only: bool) -> Self{
        //TODO?: if UPDATE_BUFFER_TEXT_TO_FOLLOW_USE_HARD_TAB_SETTING
        //let str = match crate::config::USE_HARD_TAB{
        //    true => {swap existing TAB_WIDTH spaces for tabs}
        //    false => {swap existing tabs for TAB_WIDTH spaces}
        //}
        Buffer{
            inner: Rope::from(str),
            file_path,
            read_only
        }
    }

    pub fn to_string(&self) -> String{
        self.inner.to_string()
    }

    pub fn file_path(&self) -> Option<String>{
        //match &self.file_path{
        //    Some(path) => {Some(path.to_string_lossy().to_string())}
        //    None => None
        //}
        self.file_path.as_ref().map(|path| path.to_string_lossy().to_string())
    }
    pub fn file_name(&self) -> Option<String>{
        match &self.file_path{
            Some(path) => {
                //match path.file_name(){
                //    Some(file_name) => {Some(file_name.to_string_lossy().to_string())}
                //    None => None
                //}
                path.file_name().map(|file_name| file_name.to_string_lossy().to_string())
            }
            None => None
        }
    }

    pub fn is_modified(&self) -> bool{
        match &self.file_path{
            Some(path) => {
                let file_text = Rope::from(std::fs::read_to_string(path).unwrap());
                self.inner != file_text
            }
            None => {false} //is it reasonable to say that a buffer with no file_path is always considered unmodified?...   //we can always quit without a modified warning
        }
    }

    //TODO: replace ropey specific return types
    pub fn line(&self, line_idx: usize) -> ropey::RopeSlice{    //-> String?
        self.inner.line(line_idx)
    }
    //fn get_line(&self, line_idx: usize) -> Option<ropey::RopeSlice<'_>>{
    //    self.inner.get_line(line_idx)
    //}
    pub fn lines(&self) -> ropey::iter::Lines{  //-> Vec<String>?
        self.inner.lines()
    }
    pub fn len_lines(&self) -> usize{
        self.inner.len_lines()
    }
    pub fn len_chars(&self) -> usize{
        self.inner.len_chars()
    }
    pub fn char_to_line(&self, char_idx: usize) -> usize{
        self.inner.char_to_line(char_idx)   //would try_char_to_line be worth the extra work?...
    }
    pub fn line_to_char(&self, line_idx: usize) -> usize{
        self.inner.line_to_char(line_idx)
    }
    //pub fn insert(&mut self, char_idx: usize, insert_text: &str){
    //    self.inner.insert(char_idx, insert_text);
    //}
    //pub fn remove(&mut self, start_char_idx: usize, exclusive_end_char_idx: usize){
    //    self.inner.remove(start_char_idx..exclusive_end_char_idx);
    //}
    pub fn slice(&self, start: usize, end: usize) -> String{    //this really prob ought to be &str, which is a slice
        self.inner.slice(start..end).to_string()
    }
    //TODO: should really be getting a byte or a grapheme(potentially multiple chars(unicode codepoints))
    pub fn char(&self, char_idx: usize) -> char{
        self.inner.char(char_idx)
    }
    //TODO: should really be getting a byte or a grapheme(potentially multiple chars(unicode codepoints))
    pub fn get_char(&self, char_idx: usize) -> Option<char>{
        self.inner.get_char(char_idx)
    }
    pub fn chars(&self) -> ropey::iter::Chars{
        self.inner.chars()
    }
    pub fn write_to<T>(&mut self, writer: T) -> std::io::Result<()>
        where T: std::io::Write
    {
        self.inner.write_to(writer)
    }
    pub fn byte_to_char(&self, byte_idx: usize) -> usize{
        self.inner.byte_to_char(byte_idx)
    }
    //TODO: char_to_grapheme    //the combo of these two fns could let us assert char indices are aligned to a grapheme
    //TODO: grapheme_to_char

    /// Returns the count of chars in a line of text.
    #[must_use] pub fn line_width_chars(&self, line_idx: usize, include_newline: bool) -> usize{
        let mut line_width = 0;
        for char in self.line(line_idx).chars(){
            if include_newline || char != '\n'{
                line_width += 1;
            }
        }
        line_width
    }
    //TODO?: line_width_bytes
    //TODO?: line_width_graphemes
    // returns the count of display cells a line of text inhabits
    pub fn line_width_display_cells(&self, line_idx: usize, include_newline: bool) -> usize{
        let mut line_width = 0;
        for grapheme in self.line(line_idx).to_string().graphemes(true){
            if include_newline || grapheme != "\n"{
                //determine grapheme width
                line_width += 1;    //TODO: += grapheme width
            }
        }
        line_width
    }

    /// Returns the offset of the first non space char from the start of a line of text.
    #[must_use] pub fn first_non_space_char_offset(&self, line_idx: usize) -> usize{  //-> Option<usize>?
        let line = self.line(line_idx).to_string();
        if line.is_empty(){return 0;}
        for (index, char) in line.chars().enumerate(){
            if char != ' '{return index;}
        }
        0   //if no non space chars, return no offset
    }

    /// Returns true if slice contains only spaces.
    #[must_use] pub fn slice_is_all_spaces(&self, start: usize, end: usize) -> bool{
        let slice = self.slice(start, end);
        for char in slice.chars(){
            if char != ' '{
                return false;
            }
        }
        true
    }

    /// Returns the char distance to next multiple of tab width.
    // should this be visual distance(terminal cells)/graphemes?
    #[must_use] pub fn distance_to_next_multiple_of_tab_width(
        &self,
        selection: &crate::selection::Selection,    //maybe this should take a char_idx instead?...
        semantics: crate::selection::CursorSemantics, 
        tab_width: usize
    ) -> usize{
        let next_tab_distance = self.offset_from_line_start(selection.cursor(self, semantics)) % tab_width;
        if next_tab_distance != 0{
            tab_width.saturating_sub(next_tab_distance)
        }else{
            0
        }
    }

    /// Returns the char offset of a given char from the start of a line of text.
    //TODO?: would a version of this using grapheme|cell width be useful?...
    #[must_use] pub fn offset_from_line_start(&self, point: usize) -> usize{
        let line_start = self.line_to_char(self.char_to_line(point));
        point.saturating_sub(line_start)
    }

    //TODO: should this eventually be Option<usize>?, and not saturate at buffer end
    #[must_use] pub fn next_grapheme_char_index(&self, current_index: usize) -> usize{
        let text = self.inner.slice(current_index..).to_string();
        let mut grapheme_indices = text.grapheme_indices(true).skip(1);
        let diff = match grapheme_indices.next(){
            Some((byte_idx, _str)) => self.inner.byte_to_char(byte_idx),
            None => 1   //+1 to allow for the additional space after text end for new text insertion
        };
        current_index.saturating_add(diff).min(self.inner.len_chars().saturating_add(1))
    }
    
    //TODO: should this eventually be Option<usize>?, and not saturate at buffer start
    #[must_use] pub fn previous_grapheme_char_index(&self, current_index: usize) -> usize{
        if current_index == self.len_chars().saturating_add(1){return current_index.saturating_sub(1);}
        let text = self.inner.slice(..current_index).to_string();
        let mut rev_grapheme_indices = text.grapheme_indices(true).rev();
        match rev_grapheme_indices.next(){
            Some((byte_idx, _str)) => self.inner.byte_to_char(byte_idx),
            None => 0
        }
    }

    fn is_word_char(char: char) -> bool{
        char.is_alphabetic() || char.is_numeric()/* || char == '_'*/
    }
    
    fn is_whitespace(char: char) -> bool{
        char == ' ' || char == '\t' || char == '\n'
    }
    
    /// Returns the index of the next word boundary
    #[must_use] pub fn next_word_boundary(&self, current_position: usize) -> usize{   //should this be Option<usize>?
        // if current_position == text.len_chars(){return None;}
        
        let mut index = current_position;
    
        // Skip any leading whitespace
        while index < self.len_chars() && Self::is_whitespace(self.inner.char(index)){
            index = self.next_grapheme_char_index(index);
        }
    
        // Skip to end of word chars, if any
        let mut found_word_char = false;
        while index < self.len_chars() && Self::is_word_char(self.inner.char(index)){
            index = self.next_grapheme_char_index(index);
            found_word_char = true;
        }
    
        // if no word chars, set index after next single non word char
        //if !found_word_char{
        //    if index < text.len_chars() 
        //    && !is_word_char(text.char(index)) 
        //    && !is_whitespace(text.char(index)){
        //        index = next_grapheme_index(index, text);
        //    }
        //}
        if !found_word_char
        && index < self.len_chars()
        && !Self::is_word_char(self.inner.char(index))
        && !Self::is_whitespace(self.inner.char(index)){
            index = self.next_grapheme_char_index(index);
        }
    
        if index < self.len_chars(){
            index
        }else{
            self.len_chars()
        }
    }
    
    /// Returns the index of the previous word boundary
    #[must_use] pub fn previous_word_boundary(&self, current_position: usize) -> usize{   //should this be Option<usize>?
        // if current_position == 0{return None;}
        
        let mut index = current_position;
    
        // Skip any trailing whitespace
        while index > 0 && Self::is_whitespace(self.inner.char(self.previous_grapheme_char_index(index))){
            index = self.previous_grapheme_char_index(index);
        }
    
        // Skip to start of word chars, if any
        let mut found_word_char = false;
        while index > 0 && Self::is_word_char(self.inner.char(self.previous_grapheme_char_index(index))){
            index = self.previous_grapheme_char_index(index);
            found_word_char = true;
        }
    
        // if no word chars, set index before next single non word char
        if !found_word_char{    //&& !found_whitespace
            if index > 0
            && !Self::is_word_char(self.inner.char(self.previous_grapheme_char_index(index))) 
            && !Self::is_whitespace(self.inner.char(self.previous_grapheme_char_index(index))){
                index = self.previous_grapheme_char_index(index);
            }
        }
    
        if index > 0{
            index
        }else{
            0
        }
    }


    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_replace(
        &mut self, 
        replacement_text: &str, 
        selection: &mut Selection, 
        semantics: CursorSemantics
    ) -> Change{ //TODO: Error if replacement_text is empty(or if selection empty? is this possible?)
        let old_selection = selection.clone();
        let delete_change = self.apply_delete(selection, semantics.clone());
        let replaced_text = if let Operation::Insert{inserted_text} = delete_change.inverse(){inserted_text}else{unreachable!();};  // inverse of delete change should always be insert
        let _ = self.apply_insert(replacement_text, selection, semantics.clone());   //intentionally discard returned Change

        Change::new(
            Operation::Replace{replacement_text: replacement_text.to_string()}, 
            old_selection, 
            selection.clone(), 
            Operation::Replace{replacement_text: replaced_text}
        )
    }
    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_insert(
        &mut self, 
        string: &str, 
        selection: &mut Selection, 
        semantics: CursorSemantics
    ) -> Change{    //TODO: Error if string is empty
        let old_selection = selection.clone();
        //self.insert(selection.cursor(self, semantics.clone()), string);
        self.inner.insert(selection.cursor(self, semantics.clone()), string);
        for _ in 0..string.len(){
            if let Ok(new_selection) = crate::utilities::move_cursor_right::selection_impl(selection, 1, self, None, semantics.clone()){
                *selection = new_selection;
            }
        }

        Change::new(
            Operation::Insert{inserted_text: string.to_string()}, 
            old_selection, 
            selection.clone(), 
            Operation::Delete
        )
    }
    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_delete(
        &mut self, 
        selection: &mut Selection, 
        semantics: CursorSemantics
    ) -> Change{  //TODO: Error if cursor and anchor at end of text
        use std::cmp::Ordering;
        
        let old_selection = selection.clone();
        let original_text = self.clone();

        let (start, end, new_cursor) = match selection.cursor(self, semantics.clone()).cmp(&selection.anchor()){
            Ordering::Less => {(selection.head(), selection.anchor(), selection.cursor(self, semantics.clone()))}
            Ordering::Greater => {
                match semantics{
                    CursorSemantics::Bar => {(selection.anchor(), selection.head(), selection.anchor())}
                    CursorSemantics::Block => {
                        if selection.cursor(self, semantics.clone()) == self.len_chars(){(selection.anchor(), selection.cursor(self, semantics.clone()), selection.anchor())}
                        else{(selection.anchor(), selection.head(), selection.anchor())}
                    }
                }
            }
            Ordering::Equal => {
                if selection.cursor(self, semantics.clone()) == self.len_chars(){ //do nothing    //or preferrably return error   //could have condition check in calling fn
                    return Change::new(
                        Operation::Delete, 
                        old_selection, 
                        selection.clone(), 
                        Operation::Insert{inserted_text: String::new()}
                    );   //change suggested by clippy lint
                }
                
                match semantics.clone(){
                    CursorSemantics::Bar => {(selection.head(), selection.head().saturating_add(1), selection.anchor())}
                    CursorSemantics::Block => {(selection.anchor(), selection.head(), selection.anchor())}
                }
            }
        };

        let change_text = original_text.slice(start, end);
        //buffer.remove(start..end);
        //self.remove(start, end);
        self.inner.remove(start..end);
        if let Ok(new_selection) = selection.put_cursor(new_cursor, &original_text, crate::selection::Movement::Move, semantics, true){
            *selection = new_selection;
        }

        Change::new(
            Operation::Delete, 
            old_selection, 
            selection.clone(), 
            Operation::Insert{inserted_text: change_text.to_string()}
        )
    }
}


#[cfg(test)]
mod tests{
    use unicode_width::UnicodeWidthStr;
    #[test] fn verify_unicode_width_behaves_as_expected(){
        assert_eq!(1, "a̐".width());
        assert_eq!(1, "\r\n".width());
    }
}

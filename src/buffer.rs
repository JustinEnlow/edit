// This should probably use a zed editor style rope, built on a sum_tree

use unicode_segmentation::UnicodeSegmentation;
use std::path::PathBuf;
use std::ops::Range;
use ropey::Rope;

//TODO: use explicit index type in fns  //index = from start of buffer, offset = from start of slice
//struct IndexByteUtf8(usize)
//struct IndexChar(usize);          //IndexCodePoint
//struct IndexGrapheme(usize);
//struct IndexLine(usize);
//struct IndexDisplayLine(usize);   //0 based line for terminal cell
//struct IndexDisplayCell(usize);   //0 based column for terminal cell
//struct IndexTemporal(usize, TimeStamp)    //this would be like zed "Anchor"s. resolves to another index type for the current buffer state

/// Abstraction over a stringy data type, to allow for the underlying data type to be changed as desired
// passing this structure as a reference should have no added cost compared to passing inner as a reference. they are both just the architecture pointer size
#[derive(Clone, Debug, PartialEq)]
pub struct Buffer{
    inner: Rope, 
    pub file_path: Option<PathBuf>,  //None if scratch buffer, Some(path) if from file/dir  //terminal's current dir will be used as file path for commands/plumber for temp buffers(file_path: None)
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

    pub fn file_path(&self) -> Option<String>{
        match &self.file_path{
            //Some(path) => {Some(path.to_string_lossy().to_string())}
            Some(path) => { //does this cause problems anywhere?...
                let mut file_path = path.to_string_lossy().to_string();
                if path.is_dir(){
                    file_path.push('/');
                }
                Some(file_path)
            }
            None => None
        }
        //self.file_path.as_ref().map(|path| path.to_string_lossy().to_string())
    }
    pub fn file_name(&self) -> Option<String>{
        match &self.file_path{
            Some(path) => {
                match path.file_name(){
                    //Some(file_name) => {Some(file_name.to_string_lossy().to_string())}
                    Some(file_name) => {    //does this cause problems anywhere?...
                        let mut name = file_name.to_string_lossy().to_string();
                        if path.is_dir(){
                            name.push('/');
                        }
                        Some(name)
                    }
                    None => None
                }
                //path.file_name().map(|file_name| file_name.to_string_lossy().to_string())
            }
            None => None
        }
    }

    pub fn is_modified(&self) -> bool{
        match &self.file_path{
            Some(path) => {
                if path.is_file(){
                    let file_text = Rope::from(std::fs::read_to_string(path).unwrap());
                    self.inner != file_text
                }else{
                    //maybe a better way to do this...
                    false
                }
            }
            None => {false} //is it reasonable to say that a buffer with no file_path is always considered unmodified?...   //we can always quit without a modified warning
        }
    }

    //TODO: offset_to_point, point_to_offset

    ///```should_panic
    /// # use edit::buffer::Buffer;
    /// 
    /// let buffer = Buffer::new("idk\nsome\nshit\n", None, true);
    /// assert_eq!("idk\n", buffer.line(0), "first line");
    /// assert_eq!("some\n", buffer.line(1), "second line");
    /// assert_eq!("shit\n", buffer.line(2), "third line");
    /// assert_eq!("", buffer.line(3), "last line");
    /// let _ = buffer.line(4); //any line after will panic
    ///```
    pub fn line(&self, line_idx: usize) -> String{
        self.inner.line(line_idx).to_string()
    }
    ///```
    /// # use edit::buffer::Buffer;
    /// 
    /// let text = "idk\nsome\nshit\n";
    /// let buffer = Buffer::new(text, None, true);
    /// assert_eq!(
    ///     vec![
    ///         String::from("idk\n"),
    ///         String::from("some\n"),
    ///         String::from("shit\n"),
    ///         String::new()
    ///     ], 
    ///     buffer.lines()
    /// );
    ///```
    pub fn lines(&self) -> Vec<String>{
        self.inner.lines().map(|line| line.to_string()).collect()
    }
    /// ```
    /// # use edit::buffer::Buffer;
    /// 
    /// let buffer = Buffer::new("idk\nsome\nshit\n", None, true);
    /// assert_eq!(4, buffer.len_lines());
    /// ```
    pub fn len_lines(&self) -> usize{
        self.inner.len_lines()
    }

    pub fn len_bytes(&self) -> usize{
        self.inner.len_bytes()
    }
    pub fn byte_to_line(&self, byte_offset: usize) -> usize{
        self.inner.byte_to_line(byte_offset)
    }
    pub fn line_to_byte(&self, line_idx: usize) -> usize{
        self.inner.line_to_byte(line_idx)
    }
    pub fn insert(&mut self, byte_offset: usize, insert_text: &str){
        let char_idx = self.inner.byte_to_char(byte_offset);
        self.inner.insert(char_idx, insert_text);
    }
    pub fn remove(&mut self, byte_range: Range<usize>){
        let start_char_idx = self.inner.byte_to_char(byte_range.start);
        let exclusive_end_char_idx = self.inner.byte_to_char(byte_range.end);
        self.inner.remove(start_char_idx..exclusive_end_char_idx);
    }
    pub fn slice(&self, byte_range: Range<usize>) -> String{    //this really prob ought to be &str, which is a slice
        let start = self.inner.byte_to_char(byte_range.start);
        let end = self.inner.byte_to_char(byte_range.end);
        self.inner.slice(start..end).to_string()
    }
    //TODO: should really be getting a byte or a grapheme(potentially multiple chars(unicode codepoints))
    pub fn char(&self, char_idx: usize) -> char{
        self.inner.char(char_idx)
    }
    pub fn bytes(&self) -> ropey::iter::Bytes<'_>{
        self.inner.bytes()
    }
    pub fn write_to<T>(&mut self, writer: T) -> std::io::Result<()>
        where T: std::io::Write
    {
        self.inner.write_to(writer)
    }



    /// Returns the count of bytes in a line of text.
    #[must_use] pub fn line_width_bytes(&self, line_idx: usize, include_newline: bool) -> usize{
        let mut line_width = 0;
        for byte in self.line(line_idx).bytes(){
            if include_newline || byte != b'\n'{
                line_width += 1;
            }
        }
        line_width
    }
    //TODO?: line_width_graphemes
    /// Returns the count of display cells a line of text inhabits.
    /// ```
    /// # use edit::buffer::Buffer;
    /// 
    /// let buffer = Buffer::new("何\nidk\n", None, true);
    /// assert_eq!(2, buffer.line_width_terminal_cells(0, false), "何");
    /// assert_eq!(3, buffer.line_width_terminal_cells(0, true), "{:?}", "何\n");
    /// assert_eq!(3, buffer.line_width_terminal_cells(1, false), "idk");
    /// assert_eq!(4, buffer.line_width_terminal_cells(1, true), "{:?}", "idk\n");
    /// assert_eq!(0, buffer.line_width_terminal_cells(2, false), "end of buffer");
    /// assert_eq!(0, buffer.line_width_terminal_cells(2, true), "end of buffer");
    /// ```
    // this may really belong in display_area/display_map since this is more of a visual thing...
    pub fn line_width_terminal_cells(&self, line_idx: usize, include_newline: bool) -> usize{
        let mut line_width = 0;
        for grapheme in self.line(line_idx).to_string().graphemes(true){
            if include_newline || grapheme != "\n"{
                //TODO: may need to specially handle "\t"...
                line_width += unicode_width::UnicodeWidthStr::width(grapheme);
            }
        }
        line_width
    }

    /// Returns the byte offset of the first non space grapheme from the start of a line of text.
    /// ```
    /// # use edit::buffer::Buffer;
    /// 
    /// let buffer = Buffer::new("  idk\n", None, true);
    /// assert_eq!(2, buffer.first_non_space_byte_offset(0));   //line, not byte offset
    /// assert_eq!(0, buffer.first_non_space_byte_offset(1));   //line, not byte offset
    /// ```
    // should this be terminal cells, graphemes, or bytes?...
    #[must_use] pub fn first_non_space_byte_offset(&self, line_idx: usize) -> usize{  //-> Option<usize>?
        let line = self.line(line_idx);
        if line.is_empty(){return 0;}
        for (i, grapheme) in line.grapheme_indices(true){
            #[cfg(test) ]println!("grapheme: {:?}, byte: {:?}", grapheme, i);
            //if grapheme != " "{return i;}
            if grapheme == " "{continue;}
            else{return i;}
        }
        0   //if no non space chars, return no offset
    }

    /// Returns true if slice contains only spaces.
    /// ```
    /// # use edit::buffer::Buffer;
    /// 
    /// let buffer = Buffer::new("    idk", None, true);
    /// assert_eq!(true, buffer.slice_is_all_spaces(0..4));
    /// assert_eq!(false, buffer.slice_is_all_spaces(2..6));
    /// assert_eq!(false, buffer.slice_is_all_spaces(4..buffer.len_bytes()));
    /// ```
    //TODO?: could take slice: &str instead of byte_range...
    #[must_use] pub fn slice_is_all_spaces(&self, byte_range: Range<usize>) -> bool{
        let slice = self.slice(byte_range);
        for grapheme in slice.graphemes(true){
            if grapheme != " "{
                return false;
            }
        }
        true
    }

    /// Returns the char offset of a given char from the start of a line of text.
    //TODO?: would a version of this using grapheme|cell width be useful?...
    //#[must_use] pub fn offset_from_line_start(&self, point: usize) -> usize{
    //    let line_start = self.line_to_char(self.char_to_line(point));
    //    point.saturating_sub(line_start)
    //}
    //this should give us offset in terminal cells...
    /// ```
    /// # use edit::buffer::Buffer;
    /// # use unicode_segmentation::UnicodeSegmentation;
    /// 
    /// let text = "何idk";
    /// let buffer = Buffer::new(text, None, true);
    /// for grapheme in text.graphemes(true){
    ///     println!("grapheme: {:?}, byte_count: {}", grapheme, grapheme.bytes().count());
    /// }
    /// assert_eq!(0, buffer.offset_from_line_start(0), "first byte of 何");
    /// assert_eq!(0, buffer.offset_from_line_start(1), "second byte of 何");
    /// assert_eq!(0, buffer.offset_from_line_start(2), "third byte of 何");
    /// assert_eq!(2, buffer.offset_from_line_start(3), "i");
    /// assert_eq!(3, buffer.offset_from_line_start(4), "d");
    /// assert_eq!(4, buffer.offset_from_line_start(5), "k");
    /// assert_eq!(5, buffer.offset_from_line_start(6), "out of buffer bounds");    //why does this extra work?...
    /// //assert_eq!(0, buffer.offset_from_line_start(7), "out of buffer bounds"); //this should panic...and does
    /// ```
    #[must_use] pub fn offset_from_line_start(&self, byte_offset: usize) -> usize{
        //TODO: assert byte_offset is grapheme boundary
        let line_start = self.line_to_byte(self.byte_to_line(byte_offset));
        let slice = self.slice(line_start..byte_offset);
        let mut offset: usize = 0;
        for (_i, grapheme) in slice.grapheme_indices(true){
            //TODO: maybe need to handle \t specially, since it can be expanded visually...
            offset = offset.saturating_add(unicode_width::UnicodeWidthStr::width(grapheme));
        }
        offset
    }

    /// Returns the distance to next tab_stop.
    /// ```
    /// # use edit::buffer::Buffer;
    /// # use unicode_segmentation::UnicodeSegmentation;
    /// # use unicode_width::UnicodeWidthStr;
    /// 
    /// let text = "何idk";
    /// let buffer = Buffer::new(text, None, true);
    /// for grapheme in text.graphemes(true){
    ///     println!("grapheme: {:?}, byte count: {}, grapheme width: {}", grapheme, grapheme.bytes().count(), UnicodeWidthStr::width(grapheme));
    /// }
    /// assert_eq!(4, buffer.distance_to_next_tab_stop(0, /*tab_width*/4), "first byte of 何");
    /// assert_eq!(4, buffer.distance_to_next_tab_stop(1, /*tab_width*/4), "second byte of 何");
    /// assert_eq!(4, buffer.distance_to_next_tab_stop(2, /*tab_width*/4), "third byte of 何");
    /// assert_eq!(2, buffer.distance_to_next_tab_stop(3, /*tab_width*/4), "i");
    /// assert_eq!(1, buffer.distance_to_next_tab_stop(4, /*tab_width*/4), "d");
    /// assert_eq!(4, buffer.distance_to_next_tab_stop(5, /*tab_width*/4), "k"); //although maybe won't get full tab width because buffer ends...
    /// assert_eq!(3, buffer.distance_to_next_tab_stop(6, 4), "out of buffer bounds");    //why does this extra work?...
    /// //assert_eq!(2, buffer.distance_to_next_tab_stop(7, 4), "out of buffer bounds");  //this should panic...and does
    /// ```
    // should this be terminal cells, graphemes, or bytes?...
    #[must_use] pub fn distance_to_next_tab_stop(&self, byte_offset: usize, tab_width: usize) -> usize{
        let next_tab_distance = self.offset_from_line_start(byte_offset) % tab_width;
        //if next_tab_distance != 0{
        //    tab_width.saturating_sub(next_tab_distance)
        //}else{
        //    0
        //}
        tab_width.saturating_sub(next_tab_distance)
    }

    /// ```
    /// # use edit::buffer::Buffer;
    /// 
    /// let buffer = Buffer::new("何 idk\n", None, true);
    /// assert_eq!(true, buffer.is_grapheme_boundary(0), "first byte of 何");
    /// assert_eq!(false, buffer.is_grapheme_boundary(1), "second byte of 何");
    /// assert_eq!(false, buffer.is_grapheme_boundary(2), "third byte of 何");
    /// assert_eq!(true, buffer.is_grapheme_boundary(3), "{:?}", " ");
    /// ```
    //TODO: for repeated checks, maybe cache grapheme boundaries, and update on insert/remove/etc...
    pub fn is_grapheme_boundary(&self, byte_offset: usize) -> bool{
        self.slice(0..self.len_bytes())
            .grapheme_indices(true)
            .any(|(i, _g)| i == byte_offset)
    }

    /// ```
    /// # use edit::buffer::Buffer;
    /// # use unicode_segmentation::UnicodeSegmentation;
    /// 
    /// let text = "何 idk\n";
    /// let buffer = Buffer::new(text, None, true);
    /// println!("buffer_text: {:?}", text);
    /// for grapheme in text.graphemes(true){
    ///     println!("grapheme: {:?}, grapheme bytes: {:?}", grapheme, grapheme.as_bytes());
    /// }
    /// assert_eq!(3, buffer.next_grapheme_boundary_byte_offset(0), "{:?}", "first byte of 何");
    /// assert_eq!(3, buffer.next_grapheme_boundary_byte_offset(1), "{:?}", "second byte of 何");
    /// assert_eq!(3, buffer.next_grapheme_boundary_byte_offset(2), "{:?}", "third byte of 何");
    /// assert_eq!(4, buffer.next_grapheme_boundary_byte_offset(3), "{:?}", " ");
    /// assert_eq!(5, buffer.next_grapheme_boundary_byte_offset(4), "{:?}", "i");
    /// assert_eq!(6, buffer.next_grapheme_boundary_byte_offset(5), "{:?}", "d");
    /// assert_eq!(7, buffer.next_grapheme_boundary_byte_offset(6), "{:?}", "k");
    /// assert_eq!(8, buffer.next_grapheme_boundary_byte_offset(7), "{:?}", "\n");
    /// assert_eq!(9, buffer.next_grapheme_boundary_byte_offset(8), "{:?}", "");    //why does this work?...
    /// //assert_eq!(10, buffer.next_grapheme_boundary_byte_offset(9), "{:?}", "out of buffer bounds"); //this should panic...and does
    /// ```
    //TODO: should this eventually be Option<usize>?, and not saturate at buffer end
    #[must_use] pub fn next_grapheme_boundary_byte_offset(&self, byte_offset: usize) -> usize{  //-> Option<usize>  //then let selections,etc handle buffer overshoot logic
        //if byte_offset == self.len_bytes(){return None;}  //if we decide to return Option
        if self.is_grapheme_boundary(byte_offset){  //although, our selections are supposed to always have valid ranges...
            let sub_string = self.slice(byte_offset..self.len_bytes());
            let byte_diff = match sub_string.grapheme_indices(true).next(){
                //can't use _byte_idx directly because we are in a substring of the overall buffer...
                Some((_byte_idx, grapheme)) => grapheme.bytes().count(),
                None => 1   //+1 to allow for the additional space after text end for new text insertion    //will change if we return Option
            };
            //self.len_bytes()+1 to allow for the additional space after text end for new text insertion    //will change if we return Option
            usize::min(byte_offset.saturating_add(byte_diff), self.len_bytes().saturating_add(1))
        }else{  //nudge our offset until we are at a valid grapheme boundary
            let mut i = byte_offset;
            //<= self.len_bytes() to allow for the additional space after text end for new text insertion   //will change if we return Option
            while i <= self.len_bytes() && !self.is_grapheme_boundary(i){
                i+=1;
            }
            i
        }
    }
    
    /// ```
    /// # use edit::buffer::Buffer;
    /// # use unicode_segmentation::UnicodeSegmentation;
    /// 
    /// let text = "\nidk 何";
    /// let buffer = Buffer::new(text, None, true);
    /// println!("buffer_text: {:?}", text);
    /// for grapheme in text.graphemes(true){
    ///     println!("grapheme: {:?}, grapheme bytes: {:?}", grapheme, grapheme.as_bytes());
    /// }
    /// //assert_eq!(9, buffer.previous_grapheme_boundary_byte_offset(10), "out of buffer bounds"); //this should panic...and does
    /// assert_eq!(8, buffer.previous_grapheme_boundary_byte_offset(9), "{:?}", "");    //why does this work?...
    /// assert_eq!(5, buffer.previous_grapheme_boundary_byte_offset(8), "{:?}", "third byte of 何");
    /// assert_eq!(5, buffer.previous_grapheme_boundary_byte_offset(7), "{:?}", "second byte of 何");
    /// assert_eq!(5, buffer.previous_grapheme_boundary_byte_offset(6), "{:?}", "first byte of 何");
    /// assert_eq!(4, buffer.previous_grapheme_boundary_byte_offset(5), "{:?}", " ");
    /// assert_eq!(3, buffer.previous_grapheme_boundary_byte_offset(4), "{:?}", "k");
    /// assert_eq!(2, buffer.previous_grapheme_boundary_byte_offset(3), "{:?}", "d");
    /// assert_eq!(1, buffer.previous_grapheme_boundary_byte_offset(2), "{:?}", "i");
    /// assert_eq!(0, buffer.previous_grapheme_boundary_byte_offset(1), "{:?}", "\n");
    /// ```
    //TODO: should this eventually be Option<usize>?, and not saturate at buffer start
    #[must_use] pub fn previous_grapheme_boundary_byte_offset(&self, byte_offset: usize) -> usize{  //-> Option<usize>  //then let selections,etc handle buffer overshoot logic
        if byte_offset == self.len_bytes().saturating_add(1){return byte_offset.saturating_sub(1);}
        if self.is_grapheme_boundary(byte_offset){  //although, our selections are supposed to always have valid ranges...
            let sub_string = self.slice(0..byte_offset);
            let byte_diff = match sub_string.grapheme_indices(true).rev().next(){
                Some((_byte_idx, grapheme)) => grapheme.bytes().count(),
                None => 0
            };
            byte_offset.saturating_sub(byte_diff)
        }else{  //nudge our offset until we are at a valid grapheme boundary
            let mut i = byte_offset;
            while !self.is_grapheme_boundary(i){
                i-=1;
            }
            i
        }
    }
    

    /// Returns the index of the next word boundary
    /// ```
    /// # use edit::buffer::Buffer;
    /// 
    /// let buffer = Buffer::new("idk some\nshit\n", None, true);
    /// assert_eq!(3, buffer.next_word_end_boundary(0), "from 0 to end of {:?}", "idk");
    /// assert_eq!(8, buffer.next_word_end_boundary(3), "from end of {:?} to end of {:?}", "idk", "some");
    /// assert_eq!(13, buffer.next_word_end_boundary(8), "from end of {:?} to end of {:?}", "some", "shit");
    /// assert_eq!(14, buffer.next_word_end_boundary(13), "from end of {:?} to end of buffer", "shit");
    /// assert_eq!(14, buffer.next_word_end_boundary(14), "currently saturates, but may return None instead...");
    /// ```
    //TODO: these next two functions rely on char, and shouldn't, if possible
    #[must_use] pub fn next_word_end_boundary(&self, current_position: usize) -> usize{   //should this be Option<usize>?   //then let selections,etc handle buffer overshoot logic
        // if current_position == text.len_chars(){return None;}
        
        let mut index = current_position;
    
        // Skip any leading whitespace
        while index < self.len_bytes() && is_whitespace(self.char(index)){
            index = self.next_grapheme_boundary_byte_offset(index);
        }
    
        // Skip to end of word chars, if any
        let mut found_word_char = false;
        while index < self.len_bytes() && is_word_char(self.char(index)){
            index = self.next_grapheme_boundary_byte_offset(index);
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
        && index < self.len_bytes()
        && !is_word_char(self.char(index))
        && !is_whitespace(self.char(index)){
            index = self.next_grapheme_boundary_byte_offset(index);
        }
    
        if index < self.len_bytes(){
            index
        }else{
            self.len_bytes()
        }
    }
    
    /// Returns the index of the previous word boundary
    /// ```
    /// # use edit::buffer::Buffer;
    /// 
    /// let buffer = Buffer::new("idk some\nshit\n", None, true);
    /// assert_eq!(9, buffer.previous_word_start_boundary(14), "from end of buffer to start of {:?}", "shit");
    /// assert_eq!(4, buffer.previous_word_start_boundary(9), "from start of {:?} to start of {:?}", "shit", "some");
    /// assert_eq!(0, buffer.previous_word_start_boundary(4), "from start of {:?} to start of {:?}", "some", "idk");
    /// assert_eq!(0, buffer.previous_word_start_boundary(0), "currently saturates, but may return None instead...");
    /// ```
    #[must_use] pub fn previous_word_start_boundary(&self, current_position: usize) -> usize{   //should this be Option<usize>?   //then let selections,etc handle buffer overshoot logic
        // if current_position == 0{return None;}
        
        let mut index = current_position;
    
        // Skip any trailing whitespace
        while index > 0 && is_whitespace(self.char(self.previous_grapheme_boundary_byte_offset(index))){
            index = self.previous_grapheme_boundary_byte_offset(index);
        }
    
        // Skip to start of word chars, if any
        let mut found_word_char = false;
        while index > 0 && is_word_char(self.char(self.previous_grapheme_boundary_byte_offset(index))){
            index = self.previous_grapheme_boundary_byte_offset(index);
            found_word_char = true;
        }
    
        // if no word chars, set index before next single non word char
        if !found_word_char{    //&& !found_whitespace
            if index > 0
            && !is_word_char(self.char(self.previous_grapheme_boundary_byte_offset(index))) 
            && !is_whitespace(self.char(self.previous_grapheme_boundary_byte_offset(index))){
                index = self.previous_grapheme_boundary_byte_offset(index);
            }
        }
    
        if index > 0{
            index
        }else{
            0
        }
    }
}
impl std::fmt::Display for Buffer{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "{}", self.inner)
    }
}

fn is_word_char(char: char) -> bool{
    char.is_alphabetic() || char.is_numeric()/* || char == '_'*/
}
fn is_whitespace(char: char) -> bool{
    char == ' ' || char == '\t' || char == '\n'
}


#[cfg(test)]
mod tests{
    use unicode_width::UnicodeWidthStr;
    #[test] fn verify_unicode_width_behaves_as_expected(){
        assert_eq!(1, "a̐".width());
        assert_eq!(1, "\r\n".width());
        assert_eq!(2, "何".width());
        assert_eq!(2, "🏴‍☠️".width());    //can any single grapheme be wider than 2?...
        //TODO: zero width grapheme
        //TODO: wide grapheme
    }
}

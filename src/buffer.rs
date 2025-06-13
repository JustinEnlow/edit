/// Abstraction over a stringy data type, to allow for the underlying data type to be changed as desired
// passing this structure as a reference has no added cost compared to passing inner as a reference. they are both just the architecture pointer size
#[derive(Clone, Debug, PartialEq)]
pub struct Buffer{
    pub inner: ropey::Rope, //TODO: remove pub  //inner should not be accessable externally //provide functions to access needed data
    pub file_path: Option<std::path::PathBuf>,  //None if scratch buffer, Some(path) if from file
    pub read_only: bool //true if explicitly set, or if from file with read only permissions
}
impl Buffer{
    pub fn new(str: &str, file_path: Option<std::path::PathBuf>, read_only: bool) -> Self{
        Buffer{
            inner: ropey::Rope::from(str),
            file_path,
            read_only
        }
    }

    pub fn file_path(&self) -> Option<String>{
        match &self.file_path{
            Some(path) => {Some(path.to_string_lossy().to_string())}
            None => None
        }
    }
    pub fn file_name(&self) -> Option<String>{
        match &self.file_path{
            Some(path) => {
                match path.file_name(){
                    Some(file_name) => {Some(file_name.to_string_lossy().to_string())}
                    None => None
                }
            }
            None => None
        }
    }

    pub fn len_lines(&self) -> usize{
        self.inner.len_lines()
    }

    pub fn len_chars(&self) -> usize{
        self.inner.len_chars()
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize{
        self.inner.char_to_line(char_idx)
    }
    pub fn line_to_char(&self, line_idx: usize) -> usize{
        self.inner.line_to_char(line_idx)
    }

    pub fn get_char(&self, char_idx: usize) -> Option<char>{
        self.inner.get_char(char_idx)
    }

    pub fn insert(&mut self, char_idx: usize, text: &str){
        self.inner.insert(char_idx, text)
    }
    pub fn remove<R>(&mut self, char_range: R)
        where R: std::ops::RangeBounds<usize>
    {
        self.inner.remove(char_range)
    }

    pub fn is_modified(&self) -> bool{
        match &self.file_path{
            Some(path) => {
                let file_text = ropey::Rope::from(std::fs::read_to_string(path).unwrap());
                self.inner != file_text
            }
            None => {false} //is it reasonable to say that a buffer with no file_path is always considered unmodified?...   //we can always quit without a modified warning
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> String{    //this really prob ought to be &str, which is a slice
        self.inner.slice(start..end).to_string()
    }

    /// Returns the count of chars in a line of text.
    //TODO: handle non standard width chars such as '\t'    //although, maybe that should be a separate line_width_visual fn
    //TODO: return grapheme count instead of char count
    #[must_use] pub fn line_width(&self, line_idx: usize, include_newline: bool) -> usize{
        let mut line_width = 0;
        for char in self.inner.line(line_idx).chars(){
            if include_newline || char != '\n'{
                line_width += 1;
            }
        }
        line_width
    }

    //TODO: handle graphemes instead of chars?
    /// Returns the offset of the first non whitespace grapheme from the start of a line of text.
    #[must_use] pub fn first_non_whitespace_character_offset(&self, line_idx: usize) -> usize{
        //let line = line.to_string();
        let line = self.inner.line(line_idx).to_string();

        //if line.len_chars() == 0{return 0;}
        if line.is_empty(){return 0;}

        use unicode_segmentation::UnicodeSegmentation;
        //for (index, char) in line.chars().enumerate(){
        for (index, grapheme) in line.graphemes(true).enumerate(){
            //if char != ' '{return index;}
            if grapheme != " "{return index;}
        }

        0
    }

    /// Returns the grapheme distance to next multiple of user defined tab width.
    #[must_use] pub fn distance_to_next_multiple_of_tab_width(
        &self,
        selection: &crate::selection::Selection, 
        semantics: crate::selection::CursorSemantics, 
        tab_width: usize
    ) -> usize{
        let next_tab_distance = self.offset_from_line_start(selection.cursor(self, semantics)) % tab_width;//TAB_WIDTH;
        //if offset_from_line_start(selection.cursor(semantics), text) % TAB_WIDTH != 0{
        if next_tab_distance != 0{
            //TAB_WIDTH.saturating_sub(offset_from_line_start(selection.cursor(semantics), text) % TAB_WIDTH)
            //TAB_WIDTH.saturating_sub(next_tab_distance)
            tab_width.saturating_sub(next_tab_distance)
        }else{
            0
        }
    }

    //TODO: handle graphemes instead of chars?
    /// Returns the offset of the first non whitespace grapheme from the start of a line of text.
    #[must_use] pub fn offset_of_first_non_whitespace_character_in_line(&self, line_idx: usize) -> usize{
        let line = self.inner.line(line_idx).to_string();

        //if line.len_chars() == 0{return 0;}
        if line.is_empty(){return 0;}

        use unicode_segmentation::UnicodeSegmentation;
        //for (index, char) in line.chars().enumerate(){
        for (index, grapheme) in line.graphemes(true).enumerate(){
            //if char != ' '{return index;}
            if grapheme != " "{return index;}
        }

        0
    }

    /// Returns the offset of cursor position from the start of a line of text.
    // TODO: maybe this really does belong in [Selection] in selection.rs?
    #[must_use] pub fn offset_from_line_start(&self, point: usize) -> usize{
        let line_start = self.inner.line_to_char(self.inner.char_to_line(point));
        point.saturating_sub(line_start)
    }

    #[must_use] pub fn next_grapheme_boundary_index(&self, current_index: usize) -> usize{ //should this eventually be Option<usize>?
        current_index.saturating_add(1).min(self.inner.len_chars().saturating_add(1)) //placeholder to handle ascii text. code will need to change to handle UTF-8
    }
    
    #[must_use] pub fn previous_grapheme_boundary_index(&self, current_index: usize) -> usize{ //should this eventually be Option<usize>?
        current_index.saturating_sub(1) //placeholder to handle ascii text. code will need to change to handle UTF-8
    }

    fn is_word_char(char: char) -> bool{
        if char.is_alphabetic() || char.is_numeric()/* || char == '_'*/{
            return true;
        }
    
        false
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
            index = self.next_grapheme_boundary_index(index);
        }
    
        // Skip to end of word chars, if any
        let mut found_word_char = false;
        while index < self.len_chars() && Self::is_word_char(self.inner.char(index)){
            index = self.next_grapheme_boundary_index(index);
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
            index = self.next_grapheme_boundary_index(index);
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
        while index > 0 && Self::is_whitespace(self.inner.char(self.previous_grapheme_boundary_index(index))){
            index = self.previous_grapheme_boundary_index(index);
        }
    
        // Skip to start of word chars, if any
        let mut found_word_char = false;
        while index > 0 && Self::is_word_char(self.inner.char(self.previous_grapheme_boundary_index(index))){
            index = self.previous_grapheme_boundary_index(index);
            found_word_char = true;
        }
    
        // if no word chars, set index before next single non word char
        if !found_word_char{    //&& !found_whitespace
            if index > 0
            && !Self::is_word_char(self.inner.char(self.previous_grapheme_boundary_index(index))) 
            && !Self::is_whitespace(self.inner.char(self.previous_grapheme_boundary_index(index))){
                index = self.previous_grapheme_boundary_index(index);
            }
        }
    
        if index > 0{
            index
        }else{
            0
        }
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
}

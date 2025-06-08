use std::path::PathBuf;

pub struct Buffer{
    pub inner: ropey::Rope,
    file_path: Option<PathBuf>,   //file name can be derived from path        //None if scratch buffer, Some(path) if from file
    read_only: bool                                                           //true if explicitly set, or if from file with read only permissions
}
impl Buffer{
    pub fn from_str(str: &str, read_only: bool) -> Self{
        Buffer{
            inner: ropey::Rope::from(str),
            file_path: None,
            read_only
        }
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

    #[must_use] pub fn next_grapheme_boundary_index(&self, current_index: usize) -> usize{ //should this eventually be Option<usize>?
        current_index.saturating_add(1).min(self.inner.len_chars().saturating_add(1)) //placeholder to handle ascii text. code will need to change to handle UTF-8
    }
    
    #[must_use] pub fn previous_grapheme_boundary_index(&self, current_index: usize) -> usize{ //should this eventually be Option<usize>?
        current_index.saturating_sub(1) //placeholder to handle ascii text. code will need to change to handle UTF-8
    }
}

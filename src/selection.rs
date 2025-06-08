#[derive(PartialEq)]
pub enum ExtensionDirection{
    Forward,
    Backward,
    None
}

pub struct Selection{
    range: crate::range::Range,
    direction: ExtensionDirection,
    //stored_line_offset: Option<usize>,      //grapheme offset of the cursor from line start
}
impl Selection{
    ///```
    /// use ropey::Rope;
    /// use edit::buffer::Buffer;
    /// use edit::range::Range;
    /// use edit::selection::{Selection, ExtensionDirection};
    /// let buffer = Buffer{inner: Rope::from("idk\nsome\nshit\n")};
    /// let selection = Selection{range: Range{start: 0, end: 3}, direction: ExtensionDirection::Forward, stored_line_offset: None};
    /// assert_eq!("idk".to_string(), selection.to_string(&buffer));
    ///```
    pub fn to_string(&self, buffer: &crate::buffer::Buffer) -> String{
        buffer.inner.slice(self.range.start..self.range.end).to_string()
    }

    pub fn anchor(&self) -> usize{
        match self.direction{
            ExtensionDirection::Forward |
            ExtensionDirection::None => {self.range.start}
            ExtensionDirection::Backward => {self.range.end}
        }
    }
    pub fn head(&self) -> usize{
        match self.direction{
            ExtensionDirection::Forward |
            ExtensionDirection::None => {self.range.end}
            ExtensionDirection::Backward => {self.range.start}
        }
    }
    pub fn cursor(&self, buffer: &crate::buffer::Buffer) -> usize{
        match self.direction{
            ExtensionDirection::Forward |
            ExtensionDirection::None => buffer.previous_grapheme_boundary_index(self.head()),
            ExtensionDirection::Backward => self.head()
        }
    }
    
    pub fn is_extended(&self) -> bool{
        self.direction == ExtensionDirection::Forward || self.direction == ExtensionDirection::Backward
    }

    pub fn spans_multiple_lines(&self, buffer: &crate::buffer::Buffer) -> bool{
        // ensure the selection does not exceed the length of the text
        if self.range.end > buffer.inner.len_chars(){
            return false;
        }

        let start_line = buffer.inner.char_to_line(self.range.start);
        let end_line = buffer.inner.char_to_line(self.range.end);

        // if selection is not extended or is extended on the same line
        if !self.is_extended() || 
        start_line == end_line{
            return false;
        }
        // if selection extends to a newline char, but doesn't span multiple lines
        if end_line.saturating_sub(start_line) == 1 && 
        buffer.inner.line_to_char(end_line) == self.range.end{
            return false;
        }

        // all other cases
        true
    }

    //merge overlapping?
}

#[derive(Debug, PartialEq)]
pub enum SelectionError{        //or should each fallible fn have its own fn specific Error? this would prevent the calling fn from having to match unused variants in the fallible fn...
    ResultsInSameState,
    NoOverlap,
    SpansMultipleLines,
    DirectionMismatch
}

#[derive(PartialEq, Clone, Debug)]
pub enum ExtensionDirection{
    Forward,
    Backward,
    None
}

#[derive(PartialEq)]
pub enum Movement{  //TODO: remove when move_vertically/horizontally and put_cursor fns are removed
    Extend,
    Move,
}

#[derive(Clone)]
pub enum CursorSemantics{
    Bar,
    Block,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Selection{
    pub range: crate::range::Range,
    pub direction: ExtensionDirection,
    pub stored_line_offset: Option<usize>,      //grapheme offset of the cursor from line start     //may eventually remove Option
}
impl Selection{
    /// Returns a new instance of [`Selection`] with a specified `stored_line_position`.
    // #[cfg(test)] ensures this is only compiled for tests
    #[cfg(test)] #[must_use] pub fn with_stored_line_offset(range: crate::range::Range, direction: ExtensionDirection, stored_line_offset: usize) -> Self{
        Self{range, direction, stored_line_offset: Some(stored_line_offset)}
    }
    
    //for testing...
    #[cfg(test)]pub fn new_from_components(
        anchor: usize, 
        head: usize, 
        stored_line_offset: Option<usize>, 
        buffer: &crate::buffer::Buffer, 
        semantics: CursorSemantics
    ) -> Self{
        //let (start, end, direction) = if head >= anchor{(anchor, head, ExtensionDirection::Forward)}else{(head, anchor, ExtensionDirection::Backward)};
        let (start, end, direction) = match semantics.clone(){
            CursorSemantics::Bar => {
                if head == anchor{(anchor, head, ExtensionDirection::None)}
                else if head > anchor{(anchor, head, ExtensionDirection::Forward)}
                else{(head, anchor, ExtensionDirection::Backward)}
            }
            CursorSemantics::Block => {
                if head > anchor{
                    if head.saturating_sub(anchor) == 1{(anchor, head, ExtensionDirection::None)}
                    else{(anchor, head, ExtensionDirection::Forward)}
                }
                else{(head, anchor, ExtensionDirection::Backward)}
            }
        };
        let instance = Self{
            range: crate::range::Range::new(start, end),
            direction,
            stored_line_offset
        };
        
        instance.assert_invariants(&buffer, semantics);
    
        instance
    }
    pub fn new_from_range(
        range: crate::range::Range, 
        direction: ExtensionDirection, 
        buffer: &crate::buffer::Buffer, 
        semantics: CursorSemantics
    ) -> Self{
        let instance = Self{range, direction, stored_line_offset: None};    //TODO: since we take buffer as an arg, we should determine stored_line_offset

        instance.assert_invariants(buffer, semantics);

        instance
    }

    //TODO: eventually, this should be removed, and replaced with either new_from_range or new_from_components
    /// Returns a new instance of [`Selection`].
    //#[must_use] pub fn new(range: Range, direction: Direction) -> Self{
    //    Self{range, direction, stored_line_position: None}
    //}

    //TODO: make private, to determine where this is being called unnecessarily and delete calling code
    pub fn assert_invariants(&self, buffer: &crate::buffer::Buffer, semantics: CursorSemantics){
        //assert!(self.anchor() >= 0);  //already ensured by `usize` type
        //assert!(self.head() >= 0);    //already ensured by `usize` type

        match semantics{
            CursorSemantics::Bar => {
                assert!(self.anchor() <= buffer.len_chars());
                assert!(self.head() <= buffer.len_chars());
            }
            CursorSemantics::Block => {
                if self.is_extended(){
                    assert!(self.anchor() <= buffer.len_chars());
                    assert!(self.head() <= buffer.len_chars());
                }else{    //cursor can be 1 past text end
                    assert!(self.anchor() <= buffer.len_chars().saturating_add(1));
                    assert!(self.head() <= buffer.len_chars().saturating_add(1));
                }
                assert!(self.anchor() != self.head());
            }
        }
        assert!(self.cursor(buffer, semantics) <= buffer.len_chars());
    }

    // ///```
    // /// use ropey::Rope;
    // /// use edit::buffer::Buffer;
    // /// use edit::range::Range;
    // /// use edit::selection::{Selection, ExtensionDirection};
    // /// let buffer = Buffer{inner: Rope::from("idk\nsome\nshit\n")};
    // /// let selection = Selection{range: Range{start: 0, end: 3}, direction: ExtensionDirection::Forward, stored_line_offset: None};
    // /// assert_eq!("idk".to_string(), selection.to_string(&buffer));
    // ///```
    pub fn to_string(&self, buffer: &crate::buffer::Buffer) -> String{
        //buffer.inner.slice(self.range.start..self.range.end).to_string()
        buffer.slice(self.range.start, self.range.end)
    }

    pub fn anchor(&self) -> usize{
        match self.direction{
            ExtensionDirection::None |
            ExtensionDirection::Forward => {self.range.start}
            ExtensionDirection::Backward => {self.range.end}
        }
    }
    pub fn head(&self) -> usize{
        match self.direction{
            ExtensionDirection::None |
            ExtensionDirection::Forward => {self.range.end}
            ExtensionDirection::Backward => {self.range.start}
        }
    }
    pub fn cursor(&self, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> usize{
        match self.direction{
            ExtensionDirection::None |
            ExtensionDirection::Forward => match semantics{//buffer.previous_grapheme_boundary_index(self.head()),
                CursorSemantics::Bar => self.head(),
                CursorSemantics::Block => buffer.previous_grapheme_boundary_index(self.head()),
            }
            ExtensionDirection::Backward => self.head()
        }
    }
    /// Returns the char index of the start of the [`Selection`] from left to right.
    // note: not tested in selection_tests, and i don't think it should be because all relevant tests are done in range_tests module
    #[must_use] pub fn start(&self) -> usize{self.range.start}      //only needed for Selections::sort. figure out how to make that work without this...
    
    //could do pub fn extension() -> ExtensionDirection{} instead, and use if selection.extension() == ExtensionDirection::None
    pub fn is_extended(&self) -> bool{
        self.direction == ExtensionDirection::Forward || self.direction == ExtensionDirection::Backward
    }

    pub fn spans_multiple_lines(&self, buffer: &crate::buffer::Buffer) -> bool{
        // ensure the selection does not exceed the length of the text
        if self.range.end > buffer.len_chars(){//buffer.inner.len_chars(){
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

    /// Returns a new [`Selection`] from the overlapping [`Range`]s of `self` and `other`, with a reasonable `stored_line_position` calculated.
    pub fn merge_overlapping(&self, other: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
        //assert!(self.semantics == other.semantics)    //for future consideration...
        //assert!(self.text == other.text)  //for future consideration...
        if self.range.overlaps(&other.range){
            // perform indiscriminate merge to get selection range
            let new_range = self.range.merge(&other.range);
        //    //let mut selection = Selection::new(new_range.start, new_range.end);
        //    let mut selection = Selection::new(Range::new(new_range.start, new_range.end), Direction::Forward); //maybe move match here instead of assigning Forward
        //    
        //    // set resultant direction, based on inputs
        //    match (self.direction, other.direction, self.is_extended(semantics), other.is_extended(semantics)){
        //        (Direction::Forward, Direction::Forward, false, false) => selection.direction = Direction::Forward,
        //        (Direction::Forward, Direction::Forward, true, false) => selection.direction = Direction::Forward,
        //        (Direction::Forward, Direction::Forward, false, true) => selection.direction = Direction::Forward,
        //        (Direction::Forward, Direction::Forward, true, true) => selection.direction = Direction::Forward,
        //
        //        (Direction::Forward, Direction::Backward, false, false) => selection.direction = Direction::Forward,
        //        (Direction::Forward, Direction::Backward, true, false) => selection.direction = Direction::Forward,
        //        (Direction::Forward, Direction::Backward, false, true) => selection.direction = Direction::Backward,
        //        (Direction::Forward, Direction::Backward, true, true) => selection.direction = Direction::Forward,
        //
        //        (Direction::Backward, Direction::Forward, false, false) => selection.direction = Direction::Forward,
        //        (Direction::Backward, Direction::Forward, true, false) => selection.direction = Direction::Backward,
        //        (Direction::Backward, Direction::Forward, false, true) => selection.direction = Direction::Forward,
        //        (Direction::Backward, Direction::Forward, true, true) => selection.direction = Direction::Forward,
        //
        //        (Direction::Backward, Direction::Backward, false, false) => selection.direction = Direction::Backward,
        //        (Direction::Backward, Direction::Backward, true, false) => selection.direction = Direction::Backward,
        //        (Direction::Backward, Direction::Backward, false, true) => selection.direction = Direction::Backward,
        //        (Direction::Backward, Direction::Backward, true, true) => selection.direction = Direction::Backward,
        //    }
        //    
        //    // calculate new stored_line_position
        //    //selection.stored_line_position = Some(text_util::offset_from_line_start(selection.cursor(text, semantics), text));
        //    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer)));

            let mut selection = Selection::new_from_range(
                crate::range::Range::new(new_range.start, new_range.end), 
                match (self.direction.clone(), other.direction.clone()){
                    (ExtensionDirection::None, ExtensionDirection::None) => ExtensionDirection::None,
                    (ExtensionDirection::None, ExtensionDirection::Forward) => ExtensionDirection::Forward,
                    (ExtensionDirection::None, ExtensionDirection::Backward) => ExtensionDirection::Backward,
                    (ExtensionDirection::Forward, ExtensionDirection::None) => ExtensionDirection::Forward,
                    (ExtensionDirection::Forward, ExtensionDirection::Forward) => ExtensionDirection::Forward,
                    (ExtensionDirection::Forward, ExtensionDirection::Backward) => ExtensionDirection::Forward,     //this may still change in the future...if even possible
                    (ExtensionDirection::Backward, ExtensionDirection::None) => ExtensionDirection::Backward,
                    (ExtensionDirection::Backward, ExtensionDirection::Forward) => ExtensionDirection::Forward,     //this may still change in the future...if even possible
                    (ExtensionDirection::Backward, ExtensionDirection::Backward) => ExtensionDirection::Backward,
                }, 
                buffer, 
                semantics.clone()
            );
            selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
            
            // return merged selection
            Ok(selection)
        }else{Err(SelectionError::NoOverlap)}
    }

    //TODO: should this be made purely functional?
    //TODO: should this pass up possible errors from move/extend calls?
    pub fn shift_and_extend(&mut self, amount: usize, buffer: &crate::buffer::Buffer, semantics: CursorSemantics){ //-> Result<(), SelectionError>{
        for _ in 0..amount{
            if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(self, buffer, semantics.clone()){
                *self = new_selection;
            }
        }
        if amount > 1{
            for _ in match semantics.clone(){   //match semantics to determine our iter range
                CursorSemantics::Bar => 0..amount,
                CursorSemantics::Block => 0..amount.saturating_sub(1)
            }{
                if let Ok(new_selection) = crate::utilities::extend_selection_right::selection_impl(self, buffer, semantics.clone()){
                    *self = new_selection;
                }
            }
        }
    }

    /// Translates a [`Selection`] to a [Selection2d].
    #[must_use] pub fn selection_to_selection2d(&self, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> crate::selection2d::Selection2d{
        let line_number_head = buffer.inner.char_to_line(self.cursor(buffer, semantics.clone()));
        let line_number_anchor = buffer.inner.char_to_line(self.anchor());

        let head_line_start_idx = buffer.inner.line_to_char(line_number_head);
        let anchor_line_start_idx = buffer.inner.line_to_char(line_number_anchor);

        //let mut column_head = 0;
        //for grapheme in text.slice(head_line_start_idx..self.cursor(semantics)).to_string().graphemes(true){
        //    if grapheme == "\t"{
        //        column_head += TAB_WIDTH - (column_head % TAB_WIDTH);
        //    }else{
        //        column_head += 1;
        //    }
        //}
        //let mut column_anchor = 0;
        //for grapheme in text.slice(anchor_line_start_idx..self.anchor).to_string().graphemes(true){
        //    if grapheme == "\t"{
        //        column_anchor += TAB_WIDTH - (column_head % TAB_WIDTH);
        //    }else{
        //        column_anchor += 1;
        //    }
        //}
        crate::selection2d::Selection2d::new(
            crate::position::Position::new(
                self.anchor().saturating_sub(anchor_line_start_idx),
                //column_anchor,
                line_number_anchor
            ),
            crate::position::Position::new(
                self.cursor(buffer, semantics).saturating_sub(head_line_start_idx),
                //column_head,
                line_number_head
            ) 
        )
    }



    /// Returns a new instance of [`Selection`] with the cursor moved vertically by specified amount.
    /// Errors if `amount` < 1.
    // ExtensionDirection is misused here. it used to be Direction{Forward, Backward} which makes more sense. this will all be removed when move_vertically/horizontally and put_cursor are done away with...
    pub fn move_vertically(&self, amount: usize, buffer: &crate::buffer::Buffer, movement: Movement, direction: ExtensionDirection, semantics: CursorSemantics) -> Result<Self, SelectionError>{    //TODO: error if current_line + amount > text.len_lines, or if current_line < amount when moving backward
        if amount < 1{return Err(SelectionError::ResultsInSameState);}  //and this may make sense to be an assert. we want the calling function to ensure any input is valid...
        
        let mut selection = self.clone();
        
        let current_line = buffer.char_to_line(self.cursor(buffer, semantics.clone()));
        let goal_line_number = match direction{
            ExtensionDirection::None |
            ExtensionDirection::Forward => (current_line + amount).min(buffer.len_lines().saturating_sub(1)),
            ExtensionDirection::Backward => current_line.saturating_sub(amount),
        };

        let start_of_line = buffer.line_to_char(goal_line_number);
        let line_width = buffer.line_width(goal_line_number, false);
    
        // Use the stored line offset or calculate it if None
        let stored_line_offset = self.stored_line_offset.unwrap_or_else(|| {
            buffer.offset_from_line_start(self.cursor(buffer, semantics.clone()))
        });

        // Calculate the new position based on line width
        let new_position = if stored_line_offset < line_width{
            start_of_line + stored_line_offset
        }else{
            start_of_line + line_width
        };

        selection.stored_line_offset = Some(stored_line_offset);
        selection.put_cursor(new_position, buffer, movement, semantics.clone(), false)
    }

    /// Returns a new instance of [`Selection`] with the cursor moved horizontally by specified amount.
    /// Errors if `amount` < 1.
    pub fn move_horizontally(&self, amount: usize, buffer: &crate::buffer::Buffer, movement: Movement, direction: ExtensionDirection, semantics: CursorSemantics) -> Result<Self, SelectionError>{
        if amount < 1{return Err(SelectionError::ResultsInSameState);}     //and this may make sense to be an assert. we want the calling function to ensure any input is valid...
        
        let new_position = match direction{
            ExtensionDirection::None |
            ExtensionDirection::Forward => {
                let mut index = self.cursor(buffer, semantics.clone());
                for _ in 0..amount{
                    index = buffer.next_grapheme_boundary_index(index);
                }
                index.min(buffer.len_chars()) //ensures this does not move past text end      //could match on semantics, and ensure extend does index.min(previous_grapheme_index(text.len_chars()))
            }
            ExtensionDirection::Backward => {
                let mut index = self.cursor(buffer, semantics.clone());
                for _ in 0..amount{
                    index = buffer.previous_grapheme_boundary_index(index);
                }
                index
            }
        };
        self.put_cursor(new_position, buffer, movement, semantics.clone(), true)
    }

    /// Returns a new instance of [`Selection`] with cursor at specified char index in rope.
    /// Will shift `anchor`/`head` positions to accommodate Bar/Block cursor semantics.
    /// If movement == `Movement::Move`, returned selection will always be `Direction::Forward`.
    /// `to` saturates at doc or text boundaries.
    //TODO: even if we saturate `to` at boundaries, we should assert it here, to ensure all calling functions are handling this correctly, and catching errors as early as possible
    pub fn put_cursor(&self, to: usize, buffer: &crate::buffer::Buffer, movement: Movement, semantics: CursorSemantics, update_stored_line_position: bool) -> Result<Self, SelectionError>{
        use core::cmp::Ord;
        let mut selection = self.clone();
        match (semantics.clone(), movement){
            (CursorSemantics::Bar, Movement::Move) => {
                //let to = to.min(buffer.len_chars());
                let to = Ord::min(to, buffer.len_chars());
                //Selection::new(Range::new(to, to), ExtensionDirection::None)
                selection.range.start = to;
                selection.range.end = to;
                selection.direction = ExtensionDirection::None;
            }
            (CursorSemantics::Bar, Movement::Extend) => {
                //let to = to.min(buffer.len_chars());
                let to = Ord::min(to, buffer.len_chars());
                let (start, end, direction) = if to < self.anchor(){
                    (to, self.anchor(), ExtensionDirection::Backward)
                }else{
                    (self.anchor(), to, ExtensionDirection::Forward)
                };
                //Selection::new(Range::new(start, end), direction)
                selection.range.start = start;
                selection.range.end = end;
                selection.direction = direction;
            }
            (CursorSemantics::Block, Movement::Move) => {
                //let to = to.min(buffer.len_chars());
                let to = Ord::min(to, buffer.len_chars());
                //Selection::new(Range::new(to, buffer.next_grapheme_boundary_index(to).min(buffer.len_chars().saturating_add(1))), ExtensionDirection::None)
                selection.range.start = to;
                selection.range.end = Ord::min(buffer.next_grapheme_boundary_index(to), buffer.len_chars().saturating_add(1));
                selection.direction = ExtensionDirection::None;
            }
            (CursorSemantics::Block, Movement::Extend) => {
                //let to = to.min(buffer.previous_grapheme_boundary_index(buffer.len_chars()));
                let to = Ord::min(to, buffer.previous_grapheme_boundary_index(buffer.len_chars()));
                let new_anchor = match self.direction{
                    ExtensionDirection::None |
                    ExtensionDirection::Forward => {
                        if to < self.anchor(){  //could also do self.range.start
                            if let Some(char_at_cursor) = buffer.get_char(self.cursor(buffer, semantics.clone())){
                                if char_at_cursor == '\n'{self.anchor()}
                                else{buffer.next_grapheme_boundary_index(self.anchor()).min(buffer.len_chars())}
                            }else{buffer.next_grapheme_boundary_index(self.anchor()).min(buffer.len_chars())}
                        }else{self.anchor()}
                    }
                    ExtensionDirection::Backward => {
                        if to >= self.anchor(){buffer.previous_grapheme_boundary_index(self.anchor())} //could also do self.range.end
                        else{self.anchor()}
                    }
                };

                if new_anchor <= to{    //allowing one more char past text.len_chars() for block cursor
                    //Selection::new(Range::new(new_anchor, buffer.next_grapheme_boundary_index(to).min(buffer.len_chars().saturating_add(1))), ExtensionDirection::Forward)
                    selection.range.start = new_anchor;
                    selection.range.end = Ord::min(buffer.next_grapheme_boundary_index(to), buffer.len_chars().saturating_add(1));
                    selection.direction = ExtensionDirection::Forward;
                }else{
                    //Selection::new(Range::new(to, new_anchor), ExtensionDirection::Backward)
                    selection.range.start = to;
                    selection.range.end = new_anchor;
                    selection.direction = ExtensionDirection::Backward;
                }
            }
        };

        selection.stored_line_offset = if update_stored_line_position{    //TODO: this really ought to be handled by calling fn...
            Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())))
        }else{
            self.stored_line_offset
        };

        selection.assert_invariants(buffer, semantics.clone());
        Ok(selection)
    }
}

#[cfg(test)]
mod tests{
    //verify new from components
    #[test] #[should_panic] fn zero_width_block_selection_panics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Block;
        let _ = crate::selection::Selection::new_from_components(0, 0, None, buffer, semantics.clone());
    }
    #[test] #[should_panic] fn index_past_buffer_len_panics_bar_semantics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Bar;
        let _ = crate::selection::Selection::new_from_components(15, 15, None, buffer, semantics.clone());
    }
    #[test] #[should_panic] fn index_past_buffer_len_panics_block_semantics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Block;
        let _ = crate::selection::Selection::new_from_components(15, 16, None, buffer, semantics.clone());
    }
    #[test] fn non_extended_bar_semantics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Bar;
        let idk = crate::selection::Selection::new_from_components(0, 0, None, buffer, semantics.clone());
        assert_eq!(0, idk.range.start);
        assert_eq!(0, idk.range.end);
        assert_eq!(0, idk.cursor(buffer, semantics));
        assert_eq!(None, idk.stored_line_offset);
        assert_eq!(crate::selection::ExtensionDirection::None, idk.direction);
    }
    #[test] fn non_extended_block_semantics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Block;
        let idk = crate::selection::Selection::new_from_components(0, 1, None, buffer, semantics.clone());
        assert_eq!(0, idk.range.start);
        assert_eq!(1, idk.range.end);
        assert_eq!(0, idk.cursor(buffer, semantics));
        assert_eq!(None, idk.stored_line_offset);
        assert_eq!(crate::selection::ExtensionDirection::None, idk.direction);
    }
    #[test] fn backward_extended_bar_semantics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Bar;
        let idk = crate::selection::Selection::new_from_components(1, 0, None, buffer, semantics.clone());
        assert_eq!(0, idk.range.start);
        assert_eq!(1, idk.range.end);
        assert_eq!(0, idk.cursor(buffer, semantics));
        assert_eq!(None, idk.stored_line_offset);
        assert_eq!(crate::selection::ExtensionDirection::Backward, idk.direction);
    }
    #[test] fn backward_extended_block_semantics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Block;
        let idk = crate::selection::Selection::new_from_components(2, 0, None, buffer, semantics.clone());
        assert_eq!(0, idk.range.start);
        assert_eq!(2, idk.range.end);
        assert_eq!(0, idk.cursor(buffer, semantics));
        assert_eq!(None, idk.stored_line_offset);
        assert_eq!(crate::selection::ExtensionDirection::Backward, idk.direction);
    }
    #[test] fn forward_extended_bar_semantics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Bar;
        let idk = crate::selection::Selection::new_from_components(0, 1, None, buffer, semantics.clone());
        assert_eq!(0, idk.range.start);
        assert_eq!(1, idk.range.end);
        assert_eq!(1, idk.cursor(buffer, semantics));
        assert_eq!(None, idk.stored_line_offset);
        assert_eq!(crate::selection::ExtensionDirection::Forward, idk.direction);
    }
    #[test] fn forward_extended_block_semantics(){
        let buffer = &crate::buffer::Buffer::new("idk\nsome\nshit\n", None, false);
        let semantics = crate::selection::CursorSemantics::Block;
        let idk = crate::selection::Selection::new_from_components(0, 2, None, buffer, semantics.clone());
        assert_eq!(0, idk.range.start);
        assert_eq!(2, idk.range.end);
        assert_eq!(1, idk.cursor(buffer, semantics));
        assert_eq!(None, idk.stored_line_offset);
        assert_eq!(crate::selection::ExtensionDirection::Forward, idk.direction);
    }
}

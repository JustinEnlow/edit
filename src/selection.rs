use crate::{
    range::Range,
    buffer::Buffer,
};

#[derive(PartialEq, Debug)] pub enum InvariantError{
    SelectionAnchorPastBufferEnd,
    SelectionHeadPastBufferEnd,
    ExtensionDirectionIsSomeAndShouldBeNone,
    ExtensionDirectionShouldBeBackward,
    ExtensionDirectionShouldBeForward,
    BlockSelectionAnchorSameAsHead,
    SelectionCursorPastBufferEnd,
}

#[derive(PartialEq, Clone, Debug)] pub enum Direction{Forward, Backward}
#[derive(PartialEq)] pub enum Movement{Extend, Move}
#[derive(PartialEq, Clone)] pub enum CursorSemantics{Bar, Block}   //TODO?: change to SelectionSemantics{Exclusive, Inclusive}
#[derive(Debug, PartialEq)] pub enum SelectionError{
    ResultsInSameState,
    NoOverlap,
    SpansMultipleLines,
    DirectionMismatch
}
//TODO: currently indices over collection of chars. should prob be over collection of graphemes
#[derive(PartialEq, Clone, Debug)]
pub struct Selection{
    pub range: Range,   //TODO?: use std::ops::Range
    pub extension_direction: Option<Direction>,
    /// char offset of the cursor from line start
    //TODO?: this may need to become stored_visual_offset_from_line_start, where it represents the number of display cells offset from line start(to handle multicell graphemes)
    pub stored_line_offset: Option<usize>,  //TODO: remove Option   //with buffer being passed in to new_from_range, we should be able to always derive stored_line_offset
}
impl Selection{
    // only use in tests, because this does not assert invariants
    #[cfg(test)] #[must_use] pub fn new_unchecked(range: Range, extension_direction: Option<Direction>, stored_line_offset: Option<usize>) -> Self{
        Self{range, extension_direction, stored_line_offset}
    }
    
    pub fn new_from_range(range: Range, extension_direction: Option<Direction>, buffer: &Buffer, semantics: CursorSemantics) -> Self{
        let instance = Self{range, extension_direction, stored_line_offset: None};    //TODO: since we take buffer as an arg, we should determine stored_line_offset
        //instance.assert_invariants(buffer, semantics);
        assert_eq!(Ok(()), instance.invariants_hold(buffer, semantics));
        instance
    }

    //TODO: make private, to determine where this is being called unnecessarily and delete calling code
    //pub fn assert_invariants(&self, buffer: &Buffer, semantics: CursorSemantics){
    //    //assert!(self.anchor() >= 0);  //already ensured by `usize` type
    //    //assert!(self.head() >= 0);    //already ensured by `usize` type
    //
    //    match semantics{
    //        CursorSemantics::Bar => {
    //            assert!(self.anchor() <= buffer.len_chars());
    //            assert!(self.head() <= buffer.len_chars());
    //            // new. trying this out
    //            if self.range.start == self.range.end{assert!(self.extension_direction.is_none());}
    //            else if self.cursor(buffer, semantics.clone()) < self.anchor(){assert!(self.extension_direction == Some(Direction::Backward));}
    //            else{assert!(self.extension_direction == Some(Direction::Forward));}
    //            //
    //        }
    //        CursorSemantics::Block => {
    //            if self.is_extended(){
    //                assert!(self.anchor() <= buffer.len_chars());
    //                assert!(self.head() <= buffer.len_chars());
    //            }else{    //cursor can be 1 past text end
    //                assert!(self.anchor() <= buffer.len_chars().saturating_add(1));
    //                assert!(self.head() <= buffer.len_chars().saturating_add(1));
    //            }
    //            assert!(self.anchor() != self.head());
    //            // new. trying this out... it did already catch something that was useful... prob keep
    //            if buffer.next_grapheme_char_index(self.range.start) == self.range.end{assert!(self.extension_direction.is_none());}
    //            else if self.cursor(buffer, semantics.clone()) < self.anchor(){assert!(self.extension_direction == Some(Direction::Backward));}
    //            else{assert!(self.extension_direction == Some(Direction::Forward));}
    //            //
    //        }
    //    }
    //    assert!(self.cursor(buffer, semantics) <= buffer.len_chars());
    //}
    //use this instead of assert_invariants, to get failures inside the calling fn
    pub fn invariants_hold(&self, buffer: &Buffer, semantics: CursorSemantics) -> Result<(), InvariantError>{
        match semantics{
            CursorSemantics::Bar => {
                if self.anchor() > buffer.len_chars(){return Err(InvariantError::SelectionAnchorPastBufferEnd);}
                if self.head() > buffer.len_chars(){return Err(InvariantError::SelectionHeadPastBufferEnd);}
                //
                if self.range.start == self.range.end{if self.extension_direction.is_some(){return Err(InvariantError::ExtensionDirectionIsSomeAndShouldBeNone);}}
                else if self.cursor(buffer, semantics.clone()) < self.anchor(){if self.extension_direction != Some(Direction::Backward){return Err(InvariantError::ExtensionDirectionShouldBeBackward);}}
                else{if self.extension_direction != Some(Direction::Forward){return Err(InvariantError::ExtensionDirectionShouldBeForward);}}
            }
            CursorSemantics::Block => {
                if self.is_extended(){
                    if self.anchor() > buffer.len_chars(){return Err(InvariantError::SelectionAnchorPastBufferEnd);}
                    if self.head() > buffer.len_chars(){return Err(InvariantError::SelectionHeadPastBufferEnd);}
                }else{
                    if self.anchor() > buffer.len_chars().saturating_add(1){return Err(InvariantError::SelectionAnchorPastBufferEnd);}
                    if self.head() > buffer.len_chars().saturating_add(1){return Err(InvariantError::SelectionHeadPastBufferEnd);}
                }
                if self.anchor() == self.head(){return Err(InvariantError::BlockSelectionAnchorSameAsHead);}
                //
                if buffer.next_grapheme_char_index(self.range.start) == self.range.end{if self.extension_direction.is_some(){return Err(InvariantError::ExtensionDirectionIsSomeAndShouldBeNone);}}
                else if self.cursor(buffer, semantics.clone()) < self.anchor(){if self.extension_direction != Some(Direction::Backward){return Err(InvariantError::ExtensionDirectionShouldBeBackward);}}
                else{if self.extension_direction != Some(Direction::Forward){return Err(InvariantError::ExtensionDirectionShouldBeForward);}}
                //
            }
        }
        if self.cursor(buffer, semantics) > buffer.len_chars(){return Err(InvariantError::SelectionCursorPastBufferEnd);}
    
        Ok(())
    }

    //TODO?: does this belong in buffer.rs instead?...
    pub fn to_string(&self, buffer: &Buffer) -> String{
        buffer.slice(self.range.start, self.range.end)
    }

    #[cfg(test)] pub fn debug_over_buffer_content(&self, buffer: &Buffer, semantics: CursorSemantics) -> String{
        use unicode_segmentation::UnicodeSegmentation;

        let mut debug_string = String::new();
        //for (i, char) in buffer./*inner.*/chars().enumerate(){
        for (i, grapheme) in buffer.to_string().graphemes(true).enumerate(){
            if self.anchor() == i{
                debug_string.push('|');
            }
            if semantics == CursorSemantics::Block && (self.extension_direction == None || self.extension_direction == Some(Direction::Forward)){
                if self.cursor(buffer, semantics.clone()) == i{
                    debug_string.push(':');
                }
            }
            if self.head() == i{
                match self.extension_direction{
                    None | Some(Direction::Forward) => {
                        debug_string.push('>');
                    }
                    Some(Direction::Backward) => {
                        debug_string.push('<');
                    }
                }
            }
            //debug_string.push(char);
            debug_string.push_str(grapheme);
        }
        debug_string
    }

    pub fn anchor(&self) -> usize{
        match self.extension_direction{
            None | Some(Direction::Forward) => {self.range.start}
            Some(Direction::Backward) => {self.range.end}
        }
    }
    pub fn head(&self) -> usize{
        match self.extension_direction{
            None | Some(Direction::Forward) => {self.range.end}
            Some(Direction::Backward) => {self.range.start}
        }
    }
    pub fn cursor(&self, buffer: &Buffer, semantics: CursorSemantics) -> usize{
        match self.extension_direction{
            None | Some(Direction::Forward) => match semantics{
                CursorSemantics::Bar => self.head(),
                CursorSemantics::Block => buffer.previous_grapheme_char_index(self.head()),
            }
            Some(Direction::Backward) => self.head()
        }
    }
    /// Returns the char index of the start of the [`Selection`] from left to right.
    // note: not tested in selection_tests, and i don't think it should be because all relevant tests are done in range_tests module
    #[must_use] pub fn start(&self) -> usize{self.range.start}      //only needed for Selections::sort. figure out how to make that work without this...

    /// If self.anchor and self.cursor are known, this can be used to determine the correct extension direction
    pub fn direction(&self, buffer: &Buffer, semantics: CursorSemantics) -> Option<Direction>{//ExtensionDirection{
        //if self.cursor(buffer, semantics.clone()) > self.anchor(){ExtensionDirection::Forward}
        //else if self.cursor(buffer, semantics.clone()) < self.anchor(){ExtensionDirection::Backward}
        //else{ExtensionDirection::None}
        match semantics{
            CursorSemantics::Bar => {
                if self.range.start == self.range.end{None}
                else{self.extension_direction.clone()}
            }
            CursorSemantics::Block => {
                if buffer.next_grapheme_char_index(self.range.start) == self.range.end{None}
                else{self.extension_direction.clone()}
            }
        }
    }

    //could do pub fn extension() -> ExtensionDirection{} instead, and use if selection.extension() == ExtensionDirection::None
    pub fn is_extended(&self) -> bool{
        self.extension_direction == Some(Direction::Forward) || self.extension_direction == Some(Direction::Backward)
    }

    pub fn spans_multiple_lines(&self, buffer: &Buffer) -> bool{
        // ensure the selection does not exceed the length of the text
        if self.range.end > buffer.len_chars(){
            return false;
        }

        let start_line = buffer./*inner.*/char_to_line(self.range.start);
        let end_line = buffer./*inner.*/char_to_line(self.range.end);

        // if selection is not extended or is extended on the same line
        if !self.is_extended() || 
        start_line == end_line{
            return false;
        }
        // if selection extends to a newline char, but doesn't span multiple lines
        if end_line.saturating_sub(start_line) == 1 && 
        buffer./*inner.*/line_to_char(end_line) == self.range.end{
            return false;
        }

        // all other cases
        true
    }

    /// Returns a new [`Selection`] from the overlapping [`Range`]s of `self` and `other`, with a reasonable `stored_line_position` calculated.
    pub fn merge_overlapping(&self, other: &Selection, buffer: &Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
        //assert!(self.semantics == other.semantics)    //for future consideration...
        //assert!(self.text == other.text)  //for future consideration...
        if self.range.overlaps(&other.range){
            // perform indiscriminate merge to get selection range
            let new_range = self.range.merge(&other.range);
            let mut selection = Selection::new_from_range(
                Range::new(new_range.start, new_range.end), 
                match (self.extension_direction.clone(), other.extension_direction.clone()){
                    (None, None) => None,
                    (None, Some(Direction::Forward)) => Some(Direction::Forward),
                    (None, Some(Direction::Backward)) => Some(Direction::Backward),
                    (Some(Direction::Forward), None) => Some(Direction::Forward),
                    (Some(Direction::Forward), Some(Direction::Forward)) => Some(Direction::Forward),
                    (Some(Direction::Forward), Some(Direction::Backward)) => Some(Direction::Forward),  //this may still change in the future...if even possible
                    (Some(Direction::Backward), None) => Some(Direction::Backward),
                    (Some(Direction::Backward), Some(Direction::Forward)) => Some(Direction::Forward),  //this may still change in the future...if even possible
                    (Some(Direction::Backward), Some(Direction::Backward)) => Some(Direction::Backward)
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
    pub fn shift_and_extend(&mut self, amount: usize, buffer: &Buffer, semantics: CursorSemantics){ //-> Result<(), SelectionError>{
        for _ in 0..amount{
            if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(self, 1, buffer, None, semantics.clone()){
                *self = new_selection;
            }
        }
        if amount > 1{
            for _ in match semantics.clone(){   //match semantics to determine our iter range
                CursorSemantics::Bar => 0..amount,
                CursorSemantics::Block => 0..amount.saturating_sub(1)
            }{
                if let Ok(new_selection) = crate::utilities::extend_selection_right::selection_impl(self, 1, buffer, None, semantics.clone()){
                    *self = new_selection;
                }
            }
        }
    }

    /// Translates a [`Selection`] to a [Selection2d].
    #[must_use] pub fn selection_to_selection2d(&self, buffer: &Buffer, semantics: CursorSemantics) -> crate::selection2d::Selection2d{
        let line_number_head = buffer./*inner.*/char_to_line(self.cursor(buffer, semantics.clone()));
        let line_number_anchor = buffer./*inner.*/char_to_line(self.anchor());

        let head_line_start_idx = buffer./*inner.*/line_to_char(line_number_head);
        let anchor_line_start_idx = buffer./*inner.*/line_to_char(line_number_anchor);

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
    pub fn move_vertically(&self, amount: usize, buffer: &Buffer, movement: Movement, direction: Direction, semantics: CursorSemantics) -> Result<Self, SelectionError>{    //TODO: error if current_line + amount > text.len_lines, or if current_line < amount when moving backward
        if amount < 1{return Err(SelectionError::ResultsInSameState);}  //and this may make sense to be an assert. we want the calling function to ensure any input is valid...
        
        let mut selection = self.clone();
        
        let current_line = buffer.char_to_line(self.cursor(buffer, semantics.clone()));
        let goal_line_number = match direction{
            Direction::Forward => (current_line + amount).min(buffer.len_lines().saturating_sub(1)),
            Direction::Backward => current_line.saturating_sub(amount),
        };

        let start_of_line = buffer.line_to_char(goal_line_number);
        let line_width = buffer.line_width_chars(goal_line_number, false);
    
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
    pub fn move_horizontally(&self, amount: usize, buffer: &Buffer, movement: Movement, direction: Direction, semantics: CursorSemantics) -> Result<Self, SelectionError>{
        if amount < 1{return Err(SelectionError::ResultsInSameState);}     //and this may make sense to be an assert. we want the calling function to ensure any input is valid...
        
        let new_position = match direction{
            Direction::Forward => {
                let mut index = self.cursor(buffer, semantics.clone());
                for _ in 0..amount{
                    let next_grapheme_boundary_index = buffer.next_grapheme_char_index(index);
                    if index == next_grapheme_boundary_index{break;} //break out of loop early if we are already on the last grapheme
                    index = next_grapheme_boundary_index;
                }
                index.min(buffer.len_chars()) //ensures this does not move past text end      //could match on semantics, and ensure extend does index.min(previous_grapheme_index(text.len_chars()))
            }
            Direction::Backward => {
                let mut index = self.cursor(buffer, semantics.clone());
                for _ in 0..amount{
                    let previous_grapheme_boundary_index = buffer.previous_grapheme_char_index(index);
                    if index == previous_grapheme_boundary_index{break;}    //break out of loop early if we are already on the first grapheme
                    index = previous_grapheme_boundary_index;
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
    pub fn put_cursor(&self, to: usize, buffer: &Buffer, movement: Movement, semantics: CursorSemantics, update_stored_line_position: bool) -> Result<Self, SelectionError>{
        use core::cmp::Ord;
        let mut selection = self.clone();
        match (semantics.clone(), movement){
            (CursorSemantics::Bar, Movement::Move) => {
                let to = Ord::min(to, buffer.len_chars());
                //Selection::new(Range::new(to, to), ExtensionDirection::None)
                selection.range.start = to;
                selection.range.end = to;
                selection.extension_direction = None;
            }
            (CursorSemantics::Bar, Movement::Extend) => {
                let to = Ord::min(to, buffer.len_chars());
                let (start, end, direction) = if to < self.anchor(){
                    (to, self.anchor(), Some(Direction::Backward))
                }else{
                    (self.anchor(), to, Some(Direction::Forward))
                };
                //Selection::new(Range::new(start, end), direction)
                selection.range.start = start;
                selection.range.end = end;
                selection.extension_direction = direction;
            }
            (CursorSemantics::Block, Movement::Move) => {
                let to = Ord::min(to, buffer.len_chars());
                //Selection::new(Range::new(to, buffer.next_grapheme_boundary_index(to).min(buffer.len_chars().saturating_add(1))), ExtensionDirection::None)
                selection.range.start = to;
                selection.range.end = Ord::min(buffer.next_grapheme_char_index(to), buffer.len_chars().saturating_add(1));
                selection.extension_direction = None;
            }
            (CursorSemantics::Block, Movement::Extend) => {
                let to = Ord::min(to, buffer.previous_grapheme_char_index(buffer.len_chars()));
                let new_anchor = match self.extension_direction{
                    None | Some(Direction::Forward) => {
                        if to < self.anchor(){  //could also do self.range.start
                            //if let Some(char_at_cursor) = buffer.get_char(self.cursor(buffer, semantics.clone())){
                            //    if char_at_cursor == '\n'{self.anchor()}
                            //    else{buffer.next_grapheme_boundary_index(self.anchor()).min(buffer.len_chars())}
                            /*}else{*/buffer.next_grapheme_char_index(self.anchor()).min(buffer.len_chars())//}
                        }else{self.anchor()}
                    }
                    Some(Direction::Backward) => {
                        if to >= self.anchor(){buffer.previous_grapheme_char_index(self.anchor())} //could also do self.range.end
                        else{self.anchor()}
                    }
                };

                if new_anchor <= to{    //allowing one more char past text.len_chars() for block cursor
                    //Selection::new(Range::new(new_anchor, buffer.next_grapheme_boundary_index(to).min(buffer.len_chars().saturating_add(1))), ExtensionDirection::Forward)
                    selection.range.start = new_anchor;
                    selection.range.end = Ord::min(buffer.next_grapheme_char_index(to), buffer.len_chars().saturating_add(1));
                    //selection.direction = ExtensionDirection::Forward;
                    selection.extension_direction = if buffer.next_grapheme_char_index(selection.range.start) == selection.range.end{None}
                    else{Some(Direction::Forward)}
                }else{
                    //Selection::new(Range::new(to, new_anchor), ExtensionDirection::Backward)
                    selection.range.start = to;
                    selection.range.end = new_anchor;
                    //selection.direction = ExtensionDirection::Backward;
                    selection.extension_direction = if buffer.next_grapheme_char_index(selection.range.start) == selection.range.end{None}
                    else{Some(Direction::Backward)}
                }
            }
        };

        selection.stored_line_offset = if update_stored_line_position{    //TODO: this really ought to be handled by calling fn...
            Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())))
        }else{
            self.stored_line_offset
        };

        //selection.assert_invariants(buffer, semantics.clone());   //TODO: invariants_hold fn should be called by caller of this fn...
        Ok(selection)
    }
}

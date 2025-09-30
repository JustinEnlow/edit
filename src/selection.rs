use crate::{
    range::Range,
    buffer::Buffer,
    display_area::DisplayArea
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
#[derive(Debug, PartialEq, Clone)] pub enum CursorSemantics{Bar, Block}   //TODO?: change to SelectionSemantics{Exclusive, Inclusive}
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

    pub fn to_string(&self, buffer: &Buffer) -> String{     //maybe this should just be Result<String, ()> instead...
        if self.range.start >= buffer.len_chars() && self.range.end >= buffer.len_chars(){
            String::new()
        }else{
            let start = usize::min(self.range.start, buffer.len_chars());
            let end = usize::min(self.range.end, buffer.len_chars());
            buffer.slice(start, end)
        }
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
            if let Ok(new_selection) = move_cursor_left(self, 1, buffer, None, semantics.clone()){
                *self = new_selection;
            }
        }
        if amount > 1{
            for _ in match semantics.clone(){   //match semantics to determine our iter range
                CursorSemantics::Bar => 0..amount,
                CursorSemantics::Block => 0..amount.saturating_sub(1)
            }{
                if let Ok(new_selection) = extend_selection_right(self, 1, buffer, None, semantics.clone()){
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

/// Returns a new instance of [`Selection`] with the cursor set to specified 0-based line number.
pub fn move_to_line_number(
    selection: &Selection, 
    line_number: usize, //0-based
    buffer: &Buffer, 
    movement: Movement, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    assert!(line_number < buffer.len_lines());

    if line_number == buffer.char_to_line(selection.cursor(buffer, semantics.clone())){return Err(SelectionError::ResultsInSameState);}
    
    let current_line = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let (amount, direction) = if line_number < current_line{
        (current_line.saturating_sub(line_number), Direction::Backward)
    }else{
        (line_number.saturating_sub(current_line), Direction::Forward)
    };
    selection.move_vertically(amount, buffer, movement, direction, semantics)
}

/// Returns a new instance of [`Selection`] with cursor moved up.
pub fn move_cursor_up(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == 0{
        return Err(SelectionError::ResultsInSameState);
    }
    selection.move_vertically(count, buffer, Movement::Move, Direction::Backward, semantics)
}

/// Returns a new instance of [`Selection`] with cursor moved down.
pub fn move_cursor_down(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){
        return Err(SelectionError::ResultsInSameState);
    }
    selection.move_vertically(count, buffer, Movement::Move, Direction::Forward, semantics)
}

/// Returns a new instance of [`Selection`] with cursor moved left.
pub fn move_cursor_left(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    
    if !selection.is_extended() && selection.cursor(buffer, semantics.clone()) == 0{
        return Err(SelectionError::ResultsInSameState);
    }
    selection.move_horizontally(count, buffer, Movement::Move, Direction::Backward, semantics)
}

/// Returns a new instance of [`Selection`] with cursor moved right.
pub fn move_cursor_right(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    
    if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){
        return Err(SelectionError::ResultsInSameState);
    }
    selection.move_horizontally(count, buffer, Movement::Move, Direction::Forward, semantics)
}

/// Returns a new instance of [`Selection`] with cursor moved right to the nearest word boundary.
pub fn move_cursor_word_boundary_forward(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
    
    //let goal_index = buffer.next_word_boundary(selection.head());
    let mut goal_index = selection.head();
    for _ in 0..count{
        let next_word_boundary = buffer.next_word_boundary(selection.head());
        //goal_index = buffer.next_word_boundary(selection.head());
        if goal_index == next_word_boundary{break;} //break out of loop early if we are already on the last grapheme
        goal_index = next_word_boundary;
    }
    match semantics{
        CursorSemantics::Bar => {
            selection.put_cursor(goal_index, buffer, Movement::Move, semantics, true)
        }
        CursorSemantics::Block => {
            if goal_index == buffer.len_chars(){
                selection.put_cursor(goal_index, buffer, Movement::Move, semantics, true)
            }else{
                selection.put_cursor(buffer.previous_grapheme_char_index(goal_index), buffer, Movement::Move, semantics, true)
            }
        }
    }
}

/// Returns a new instance of [`Selection`] with cursor moved left to the nearest word boundary.
pub fn move_cursor_word_boundary_backward(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    
    //let goal_index = buffer.previous_word_boundary(selection.cursor(buffer, semantics.clone()));
    let mut goal_index = selection.cursor(buffer, semantics.clone());
    for _ in 0..count{
        let previous_word_boundary = buffer.previous_word_boundary(selection.cursor(buffer, semantics.clone()));
        //goal_index = buffer.previous_word_boundary(selection.cursor(buffer, semantics.clone()));
        if goal_index == previous_word_boundary{break;}  //break out of loop early if we are already on the first grapheme
        goal_index = previous_word_boundary;
    }
    selection.put_cursor(goal_index, buffer, Movement::Move, semantics, true)
}

/// Returns a new instance of [`Selection`] with cursor moved to end of line text.
pub fn move_cursor_line_end(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let mut selection = selection.clone();
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_width = buffer.line_width_chars(line_number, false);
    let line_start = buffer.line_to_char(line_number);
    let line_end = line_start.saturating_add(line_width);   //nth_next_grapheme_index(line_start, line_width, text)?

    if selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
    //selection.put_cursor(line_end, text, Movement::Move, semantics, true)
    
    selection.range.start = line_end;
    selection.range.end = match semantics.clone(){
        CursorSemantics::Bar => line_end.min(buffer.len_chars()),
        CursorSemantics::Block => buffer.next_grapheme_char_index(line_end).min(buffer.len_chars().saturating_add(1))
    };
    selection.extension_direction = None;
    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
    
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));

    Ok(selection)
}

/// Returns a new instance of [`Selection`] with cursor moved to absolute start of line.
pub fn move_cursor_line_start(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_start = buffer.line_to_char(line_number);

    if selection.cursor(buffer, semantics.clone()) == line_start && !selection.is_extended(){return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(line_start, buffer, Movement::Move, semantics.clone(), true)
}

/// Returns a new instance of [`Selection`] with cursor moved to line text start.
pub fn move_cursor_line_text_start(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_start = buffer.line_to_char(line_number);
    let text_start_offset = buffer.first_non_space_char_offset(line_number);
    let text_start = line_start.saturating_add(text_start_offset);  //nth_next_grapheme_index(line_start, text_start_offset, text)?

    if selection.cursor(buffer, semantics.clone()) == text_start && !selection.is_extended(){return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(text_start, buffer, Movement::Move, semantics, true)
}

/// Returns a new instance of [`Selection`] with cursor between absolute start of line and line text start.
pub fn move_cursor_home(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_start = buffer.line_to_char(line_number);
    let text_start_offset = buffer.first_non_space_char_offset(line_number);
    let text_start = line_start.saturating_add(text_start_offset);  //nth_next_grapheme_index(line_start, text_start_offset, text)?

    if selection.cursor(buffer, semantics.clone()) == text_start{crate::selection::move_cursor_line_start(selection, buffer, semantics)}
    else{crate::selection::move_cursor_line_text_start(selection, buffer, semantics)}
}

/// Returns a new instance of [`Selection`] with the cursor moved to the start of the buffer.
pub fn move_cursor_buffer_start(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(0, buffer, Movement::Move, semantics, true)
}

/// Returns a new instance of [`Selection`] with the cursor moved to the end of the buffer.
pub fn move_cursor_buffer_end(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(buffer.len_chars(), buffer, Movement::Move, semantics, true)
}

/// Returns a new instance of [`Selection`] with the cursor moved up by the height of `client_view`.
pub fn move_cursor_page_up(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    let client_view = match display_area{
        Some(client_view) => client_view,
        None => return Err(SelectionError::ResultsInSameState), //maybe need a better error
    };
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.move_vertically(
        count.saturating_mul(client_view.height.saturating_sub(1)),
        buffer, 
        Movement::Move, 
        Direction::Backward, 
        semantics
    )
}

/// Returns a new instance of [`Selection`] with the cursor moved down by the height of `client_view`.
pub fn move_cursor_page_down(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    let client_view = match display_area{
        Some(client_view) => client_view,
        None => return Err(SelectionError::ResultsInSameState), //maybe need a better error
    };
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}
    selection.move_vertically(
        count.saturating_mul(client_view.height.saturating_sub(1)),
        buffer, 
        Movement::Move, 
        Direction::Forward, 
        semantics
    )
}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended up.
pub fn extend_selection_up(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.move_vertically(count, buffer, Movement::Extend, Direction::Backward, semantics)
}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended down.
pub fn extend_selection_down(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    let last_line = buffer.len_lines().saturating_sub(1);
    if buffer.char_to_line(selection.range.start) == last_line
    || buffer.char_to_line(selection.range.end) == last_line
    || buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == last_line{return Err(SelectionError::ResultsInSameState);}

    selection.move_vertically(count, buffer, Movement::Extend, Direction::Forward, semantics)
}

pub fn extend_selection_left(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.move_horizontally(count, buffer, Movement::Extend, Direction::Backward, semantics)
}

pub fn extend_selection_right(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));

    if (
        selection.range.start == buffer.len_chars() || 
        selection.range.end == buffer.len_chars() || 
        selection.cursor(buffer, semantics.clone()) == buffer.len_chars()
    ) && (    //needs to be able to shrink selection if extension_direction is Backward
        selection.extension_direction.is_none() ||
        selection.extension_direction == Some(Direction::Forward)
    ){return Err(SelectionError::ResultsInSameState);}
    selection.move_horizontally(count, buffer, Movement::Extend, Direction::Forward, semantics)
}

/// Returns a new instance of [`Selection`] with cursor extended left to the nearest word boundary.
pub fn extend_selection_word_boundary_backward(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    
    //let goal_index = buffer.previous_word_boundary(selection.cursor(buffer, semantics.clone()));
    let mut goal_index = selection.cursor(buffer, semantics.clone());
    for _ in 0..count{
        let previous_word_boundary = buffer.previous_word_boundary(selection.cursor(buffer, semantics.clone()));
        //goal_index = buffer.previous_word_boundary(selection.cursor(buffer, semantics.clone()));
        if goal_index == previous_word_boundary{break;}  //break out of loop early if we are already on the first grapheme
        goal_index = previous_word_boundary;
    }
    selection.put_cursor(goal_index, buffer, Movement::Extend, semantics, true)
}

//TODO: this seems to be misbehaving when selection already extend left word boundary, and then extend right word boundary triggered.
//only when cursor over character that can be a beginning or ending word boundary...
/// Returns a new instance of [`Selection`] with cursor extended right to the nearest word boundary.
pub fn extend_selection_word_boundary_forward(
    selection: &Selection, 
    count: usize, 
    buffer: &Buffer, 
    display_area: Option<&DisplayArea>, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{  //TODO: ensure this can't extend past doc text end
    if count < 1{return Err(SelectionError::ResultsInSameState);}
    assert!(display_area.is_none());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.range.start == buffer.len_chars()
    || selection.range.end == buffer.len_chars()
    || selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
        
    //let goal_index = buffer.next_word_boundary(selection.head());
    let mut goal_index = selection.head();
    for _ in 0..count{
        let next_word_boundary = buffer.next_word_boundary(selection.head());
        //goal_index = buffer.next_word_boundary(selection.head());
        if goal_index == next_word_boundary{break;} //break out of loop early if we are already on the last grapheme
        goal_index = next_word_boundary;
    }
    match semantics{
        CursorSemantics::Bar => {
            selection.put_cursor(goal_index, buffer, Movement::Extend, semantics, true)
        }
        CursorSemantics::Block => {
            if goal_index == buffer.len_chars(){
                //self.put_cursor(goal_index, text, Movement::Extend, semantics, true)
                selection.put_cursor(buffer.previous_grapheme_char_index(buffer.len_chars()), buffer, Movement::Extend, semantics, true)
            }else{
                selection.put_cursor(
                    buffer.previous_grapheme_char_index(goal_index), 
                    buffer, 
                    Movement::Extend, 
                    semantics, 
                    true
                )
            }
        }
    }
}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended to the end of the current line.
pub fn extend_selection_line_end(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{    //TODO: ensure this can't extend past doc text end
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_width = buffer.line_width_chars(line_number, false);    //doesn't include newline
    let line_start = buffer.line_to_char(line_number);
    let line_end = line_start.saturating_add(line_width);   //index at end of line text, not including newline  //nth_next_grapheme_index(line_start, line_width, text)?

    match semantics{
        CursorSemantics::Bar => {
            if selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
            selection.put_cursor(line_end, buffer, Movement::Extend, semantics, true)
        }
        CursorSemantics::Block => {
            //if self.cursor(semantics) == line_end.saturating_sub(1)
            if selection.cursor(buffer, semantics.clone()) == buffer.previous_grapheme_char_index(line_end)
            || selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
            let start_line = buffer.char_to_line(selection.range.start);
            let end_line = buffer.char_to_line(selection.range.end);
            if selection.cursor(buffer, semantics.clone()) == selection.range.start && end_line > start_line{
                selection.put_cursor(line_end, buffer, Movement::Extend, semantics, true)  //put cursor over newline, if extending from a line below
            }else{
                //self.put_cursor(line_end.saturating_sub(1), text, Movement::Extend, semantics, true)
                selection.put_cursor(buffer.previous_grapheme_char_index(line_end), buffer, Movement::Extend, semantics, true)
            }
        }
    }
}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended to the start of the current line.
pub fn extend_selection_line_start(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_start = buffer.line_to_char(line_number);

    if selection.cursor(buffer, semantics.clone()) == line_start{return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(line_start, buffer, Movement::Extend, semantics, true)
}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended to the start of the text on the current line.
pub fn extend_selection_line_text_start(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_start = buffer.line_to_char(line_number);
    let text_start_offset = buffer.first_non_space_char_offset(line_number);
    let text_start = line_start.saturating_add(text_start_offset);  //nth_next_grapheme_index(line_start, text_start_offset, text)?

    if selection.cursor(buffer, semantics.clone()) == text_start{return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(text_start, buffer, Movement::Extend, semantics, true)
}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended to absolute start of line, or line text start, depending on [`Selection`] `head` position.
pub fn extend_selection_home(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_start = buffer.line_to_char(line_number);
    let text_start_offset = buffer.first_non_space_char_offset(line_number);
    let text_start = line_start.saturating_add(text_start_offset);  //nth_next_grapheme_index(line_start, text_start_offset, text)?

    if selection.cursor(buffer, semantics.clone()) == text_start{extend_selection_line_start(selection, buffer, semantics.clone())}
    else{extend_selection_line_text_start(selection, buffer, semantics)}
}

/// Returns a new instance of [`Selection`] with the selection extended to the start of the buffer.
pub fn extend_selection_buffer_start(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(0, buffer, Movement::Extend, semantics, true)
}

/// Returns a new instance of [`Selection`] with the selection extended to the end of the buffer.
pub fn extend_selection_buffer_end(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(buffer.len_chars(), buffer, Movement::Extend, semantics, true)
}

/// Returns a new instance of [`Selection`] with the selection extended up by the height of `client_view`.
pub fn extend_selection_page_up(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let client_view = match display_area{
        Some(client_view) => client_view,
        None => return Err(SelectionError::ResultsInSameState), //maybe need a better error
    };
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == 0{return Err(SelectionError::ResultsInSameState);}
    selection.move_vertically(
        count.saturating_mul(client_view.height.saturating_sub(1)),
        buffer, 
        Movement::Extend, 
        Direction::Backward, 
        semantics
    )
}

/// Returns a new instance of [`Selection`] with the selection extended down by the height of `client_view`.
pub fn extend_selection_page_down(selection: &Selection, count: usize, buffer: &crate::buffer::Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let client_view = match display_area{
        Some(client_view) => client_view,
        None => return Err(SelectionError::ResultsInSameState), //maybe need a better error
    };
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}
    selection.move_vertically(
        count.saturating_mul(client_view.height.saturating_sub(1)),
        buffer, 
        Movement::Extend, 
        Direction::Forward, 
        semantics
    )
}

/// Returns a new instance of [`Selection`] encompassing the current line.
//TODO: make pub fn select_line //should this include newline at end of line? //should this include indentation at start of line? //vscode includes both, as does kakoune
//TODO: if called on empty last line, this moves the selection to second to last line end, instead it should error
pub fn select_line(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    //vs code selects all spanned lines...  maybe caller can make that determination...
    if selection.spans_multiple_lines(buffer){return Err(SelectionError::SpansMultipleLines);}    //make specific error. SpansMultipleLines or something...
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}

    let line = buffer.char_to_line(selection.range.start);
    let line_start = buffer.line_to_char(line);
    let line_end = line_start + buffer.line_width_chars(line, true);

    if selection.range.start == line_start && selection.range.end == line_end{Err(SelectionError::ResultsInSameState)}
    else{
        let mut selection = selection.clone();
        selection.range.start = line_start;
        selection.range.end = line_end;
        selection.extension_direction = Some(Direction::Forward);
        //TODO?: maybe update stored line offset?...
        Ok(selection)
    }
}

/// Returns a new instance of [`Selection`] with [`Selection`] extended to encompass all text.
pub fn select_all(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{  //TODO: ensure this can't extend past doc text end
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if selection.range.start == 0 
    && (
        selection.range.end == buffer.len_chars() || 
        selection.range.end == buffer.len_chars().saturating_add(1)
    ){return Err(SelectionError::ResultsInSameState);}
    
    let selection = selection.put_cursor(0, buffer, Movement::Move, semantics.clone(), true)?;
    selection.put_cursor(
        match semantics{
            CursorSemantics::Bar => buffer.len_chars(), 
            CursorSemantics::Block => buffer.previous_grapheme_char_index(buffer.len_chars())
        }, 
        buffer, 
        Movement::Extend, 
        semantics, 
        true
    )
}

pub fn flip_direction(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    //use crate::selection::ExtensionDirection;
    //selection.assert_invariants(buffer, semantics.clone());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if !selection.is_extended(){return Err(SelectionError::ResultsInSameState)}
    //Ok(
    //    Selection::new(
    //        selection.range.clone(), 
    //        match selection.direction{
    //            Direction::Forward => {Direction::Backward}
    //            Direction::Backward => {Direction::Forward}
    //        }
    //    )
    //)
    let mut new_selection = selection.clone();
    new_selection.extension_direction = match selection.extension_direction{
        None/*ExtensionDirection::None*/ => return Err(SelectionError::ResultsInSameState),
        Some(Direction::Forward)/*ExtensionDirection::Forward*/ => Some(Direction::Backward)/*ExtensionDirection::Backward*/,
        Some(Direction::Backward)/*ExtensionDirection::Backward*/ => Some(Direction::Forward)/*ExtensionDirection::Forward*/
    };
    new_selection.stored_line_offset = Some(buffer.offset_from_line_start(new_selection.cursor(buffer, semantics)));
    Ok(new_selection)
}

#[must_use] pub fn surround(
    selection: &Selection, 
    buffer: &Buffer
) -> Vec<Selection>{
    //TODO: selection.assert_invariants(text, semantics);
    let mut surround_selections = Vec::new();
    if selection.range.start == buffer.len_chars(){return surround_selections;}
    //let first_selection = Selection::new(Range::new(selection.range.start, text_util::next_grapheme_index(selection.range.start, text)), Direction::Forward);
    let mut first_selection = selection.clone();
    first_selection.range.start = selection.range.start;
    first_selection.range.end = buffer.next_grapheme_char_index(selection.range.start);
    first_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;
    //let second_selection = Selection::new(Range::new(selection.range.end, text_util::next_grapheme_index(selection.range.end, text)), Direction::Forward);
    let mut second_selection = selection.clone();
    second_selection.range.start = selection.range.end;
    second_selection.range.end = buffer.next_grapheme_char_index(selection.range.end);
    second_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;

    surround_selections.push(first_selection);
    surround_selections.push(second_selection);
    surround_selections
}

//TODO: maybe this should be implemented with treesitter, so irrelevant pairs(like ' characters inside words(like don't)) aren't matched
//TODO: maybe front end should pass in their view of what is a valid surrounding pair, then we can match those...to make this as flexible as possible
//TODO: think about how surrounding quotation pairs should be handled
/// Returns a new pair of [`Selection`]s with each selection over the nearest surrounding grapheme pair, if possible
/// valid pairs:    //maybe add ':', '*'
/// { }
/// ( )
/// [ ]
/// < >
/// ' '
/// " "
#[must_use] pub fn nearest_surrounding_pair(
    selection: &Selection, 
    buffer: &Buffer
) -> Vec<Selection>{
    let mut rev_search_index = selection.range.start;
    'outer: loop{
        let current_char = buffer./*inner.*/char(rev_search_index);
        if is_opening_bracket(current_char){
            let opening_char = current_char;
            let closing_char = get_matching_closing_bracket(opening_char);
            let mut match_stack = Vec::new();
            let mut search_index = rev_search_index;
            'inner: loop{
                let current_char = buffer./*inner.*/char(search_index);
                if opening_char == closing_char{  //search before cursor for previous instance of char, then after cursor for next instance. ignore hierarchy because i'm not sure we can parse that...
                    if current_char == closing_char{
                        if match_stack.is_empty(){
                            match_stack.push(current_char);
                        }
                        else{
                            let mut first_selection = selection.clone();
                            first_selection.range.start = rev_search_index;
                            first_selection.range.end = buffer.next_grapheme_char_index(rev_search_index);
                            first_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;

                            let mut second_selection = selection.clone();
                            second_selection.range.start = search_index;
                            second_selection.range.end = buffer.next_grapheme_char_index(search_index);
                            second_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;
                            return vec![
                                //Selection::new(Range::new(rev_search_index, text_util::next_grapheme_index(rev_search_index, text)), Direction::Forward),
                                first_selection,
                                //Selection::new(Range::new(search_index, text_util::next_grapheme_index(search_index, text)), Direction::Forward)
                                second_selection
                            ];
                        }
                    }
                    else{/*do nothing. index will be incremented below...*/}
                }
                else{
                    if current_char == opening_char{
                        match_stack.push(current_char);
                    }
                    else if current_char == closing_char{
                        match_stack.pop();
                        if match_stack.is_empty(){
                            if search_index >= selection.range.start{
                                let mut first_selection = selection.clone();
                                first_selection.range.start = rev_search_index;
                                first_selection.range.end = buffer.next_grapheme_char_index(rev_search_index);
                                first_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;

                                let mut second_selection = selection.clone();
                                second_selection.range.start = search_index;
                                second_selection.range.end = buffer.next_grapheme_char_index(search_index);
                                second_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;
                                return vec![
                                    //Selection::new(Range::new(rev_search_index, text_util::next_grapheme_index(rev_search_index, text)), Direction::Forward),
                                    first_selection,
                                    //Selection::new(Range::new(search_index, text_util::next_grapheme_index(search_index, text)), Direction::Forward)
                                    second_selection
                                ];
                            }
                            else{break 'inner;}
                        }
                        else{/*do nothing. index will be incremented below...*/}
                    }
                }
                    
                search_index = search_index + 1;

                if search_index >= buffer.len_chars(){break 'outer;}
            }
        }
        //else{ //is else really needed here?...
            rev_search_index = rev_search_index.saturating_sub(1);
        //}

        if rev_search_index == 0{break 'outer;}
    }

    Vec::new()
}
fn is_opening_bracket(char: char) -> bool{  //TODO: this should prob be in text_util.rs
    char == '{'
    || char == '('
    || char == '['
    || char == '<'
    || char == '\''
    || char == '"'
}
fn get_matching_closing_bracket(char: char) -> char{    //TODO: this should prob be in text_util.rs
    if char == '{'{'}'}
    else if char == '('{')'}
    else if char == '['{']'}
    else if char == '<'{'>'}
    else if char == '\''{'\''}
    else if char == '"'{'"'}
    else{panic!();} //TODO: maybe return None, or an error?...
}

/// Returns a [`Vec`] of [`Selection`]s where the underlying text is a match for the `input` search string.
#[must_use] pub fn incremental_search_in_selection(
    selection: &Selection, 
    input: &str, 
    buffer: &Buffer
) -> Vec<Selection>{   //text should be the text within a selection, not the whole document text       //TODO: -> Result<Vec<Selection>>
    let mut selections = Vec::new();
    let start = selection.range.start;
    if let Ok(regex) = regex::Regex::new(input){
        //regex returns byte indices, and the current Selection impl uses char indices...
        for search_match in regex.find_iter(&buffer.to_string()[start..selection.range.end.min(buffer.len_chars())]){
            let mut new_selection = selection.clone();
            //if we used char indexing instead of byte indexing, we could use buffer.byte_to_char(search_match.start()).saturating_add(start)
            //new_selection.range.start = search_match.start().saturating_add(start);
            new_selection.range.start = buffer.byte_to_char(search_match.start()).saturating_add(start);
            //new_selection.range.end = search_match.end().saturating_add(start);
            new_selection.range.end = buffer.byte_to_char(search_match.end()).saturating_add(start);
            //new_selection.extension_direction = Some(Direction::Forward);//ExtensionDirection::Forward;
            new_selection.extension_direction = if buffer.next_grapheme_char_index(new_selection.range.start) == new_selection.range.end{None}
            else{Some(Direction::Forward)};
            selections.push(new_selection);
        }
    }
    //else{/*return error FailedToParseRegex*/} //no match found if regex parse fails
    selections  //if selections empty, no match found
}

/// Returns a [`Vec`] of [`Selection`]s containing each part of the current selection except the split pattern.
#[must_use] pub fn incremental_split_in_selection(
    selection: &Selection, 
    pattern: &str, 
    buffer: &Buffer
) -> Vec<Selection>{
    let mut selections = Vec::new();
    if let Ok(regex) = regex::Regex::new(pattern){
        let mut start = selection.range.start; //0;
        let mut found_split = false;
        // Iter over each split, and push the retained selection before it, if any...       TODO: test split at start of selection
        for split in regex.find_iter(&buffer./*inner.*/to_string()[selection.range.start..selection.range.end.min(buffer.len_chars())]){
            found_split = true;
            let selection_range = Range::new(start, split.start().saturating_add(selection.range.start));
            if selection_range.start < selection_range.end{
                let mut new_selection = selection.clone();
                new_selection.range.start = selection_range.start;
                new_selection.range.end = selection_range.end;
                //new_selection.extension_direction = Some(Direction::Forward);
                new_selection.extension_direction = if buffer.next_grapheme_char_index(new_selection.range.start) == new_selection.range.end{None}
                else{Some(Direction::Forward)};
                selections.push(new_selection);
            }
            start = split.end().saturating_add(selection.range.start);
        }
        // Handle any remaining text after the last split
        //if split found and end of last split < selection end
        if found_split && start < selection.range.end.min(buffer.len_chars()){
            let mut new_selection = selection.clone();
            new_selection.range.start = start;
            new_selection.range.end = selection.range.end.min(buffer.len_chars());
            //new_selection.extension_direction = Some(Direction::Forward);
            new_selection.extension_direction = if buffer.next_grapheme_char_index(new_selection.range.start) == new_selection.range.end{None}
            else{Some(Direction::Forward)};
            selections.push(new_selection);
        }
    }
    selections
}

//TODO: we should allow collapsing to anchor, or collapse to anchor collapse(&self, text: &Rope, semantics: CursorSemantics, collapse_target: Anchor)
/// Returns a new instance of [`Selection`] with `anchor` aligned with cursor.
pub fn collapse_selection_to_cursor(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if !selection.is_extended(){return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(selection.cursor(buffer, semantics.clone()), buffer, Movement::Move, semantics, true)
}

/// Returns a new instance of [`Selection`] with `cursor` aligned with anchor.
pub fn collapse_selection_to_anchor(
    selection: &Selection, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selection, SelectionError>{
    //selection.assert_invariants(buffer, semantics.clone());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    if !selection.is_extended(){return Err(SelectionError::ResultsInSameState);}
    let result = selection.put_cursor(
        if selection.direction(buffer, semantics.clone()) == Some(crate::selection::Direction::Backward){
            buffer.previous_grapheme_char_index(selection.anchor())
        }else{
            selection.anchor()
        }, 
        buffer, 
        Movement::Move, 
        semantics.clone(), 
        true
    );
    match result{
        Ok(selection) => {
            assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
            Ok(selection)
        }
        Err(e) => Err(e)
    }
}


// Returns a new instance of [`Selection`] with the [`Selection`] extended up by the height of `client_view`.
//pub fn extend_page_up(&self, text: &Rope, client_view: &View, semantics: CursorSemantics) -> Result<Self, SelectionError>{
//    self.assert_invariants(text, semantics);
//    if text.char_to_line(self.cursor(text, semantics)) == 0{return Err(SelectionError::ResultsInSameState);}
//    self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Extend, Direction::Backward, semantics)
//}
// Returns a new instance of [`Selection`] with the [`Selection`] extended down by the height of `client_view`.
//pub fn extend_page_down(&self, text: &Rope, client_view: &View, semantics: CursorSemantics) -> Result<Self, SelectionError>{    //TODO: ensure this can't extend past doc text end
//    self.assert_invariants(text, semantics);
//    //if text.char_to_line(self.cursor(text, semantics)) == text.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}
//    let last_line = text.len_lines().saturating_sub(1);    //do we need to satsub 2, so that we are checking last viable extend line, not last empty line?...
//    if text.char_to_line(self.range.start) == last_line
//    || text.char_to_line(self.range.end) == last_line
//    || text.char_to_line(self.cursor(text, semantics)) == last_line{return Err(SelectionError::ResultsInSameState);}
//
//    //let last_line = text.len_lines().saturating_sub(1);
//    let current_line = text.char_to_line(self.cursor(text, semantics));
//    
//    //ensure amount passed to move_vertically is always valid input
//    let amount = client_view.height().saturating_sub(1);
//    let max_amount = last_line.saturating_sub(current_line);
//    let saturated_amount = amount.min(max_amount);
//    if saturated_amount == 0{Err(SelectionError::ResultsInSameState)}
//    else{self.move_vertically(saturated_amount, text, Movement::Extend, Direction::Forward, semantics)}
//    //self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Extend, Direction::Forward, semantics)
//}
// Returns a new instance of [`Selection`] with the [`Selection`] extended to doc start.
//pub fn extend_doc_start(&self, text: &Rope, semantics: CursorSemantics) -> Result<Self, SelectionError>{
//    self.assert_invariants(text, semantics);
//    if self.cursor(text, semantics) == 0{return Err(SelectionError::ResultsInSameState);}
//    self.put_cursor(0, text, Movement::Extend, semantics, true)
//}
// Returns a new instance of [`Selection`] with the [`Selection`] extended to doc end.
//pub fn extend_doc_end(&self, text: &Rope, semantics: CursorSemantics) -> Result<Self, SelectionError>{  //TODO: ensure this can't extend past doc text end
//    self.assert_invariants(text, semantics);
//    if self.range.start == text.len_chars()
//    || self.range.end == text.len_chars()
//    || self.cursor(text, semantics) == text.len_chars(){return Err(SelectionError::ResultsInSameState);}
//    
//    self.put_cursor(
//        match semantics{
//            CursorSemantics::Bar => text.len_chars(), 
//            CursorSemantics::Block => text_util::previous_grapheme_index(text.len_chars(), text)
//        }, 
//        text, 
//        Movement::Extend, 
//        semantics, 
//        true
//    )
//
//    // or if we end up getting rid of put cursor...
//    //let mut selection = Selection::new(self.anchor(), text.len_chars());
//    //selection.stored_line_position = Some(text_util::offset_from_line_start(selection.cursor(text, semantics), text));
//    //Ok(selection)
//}
//
//
//
//TODO: make smart_select_grow  //grows selection to ecompass next largest text object(word -> long_word -> long_word+surrounding punctuation or whitespace -> inside brackets -> sentence -> line -> paragraph -> all)
//TODO: make smart_select_shrink    //opposite of above
//
//may still have this be a part of nearest_surrounding_pair instead...
//pub fn nearest_quote_pair(&self, text: &Rope) -> Vec<Selection>{
//    //something"idk"else
//    let mut rev_search_index = self.range.start;
//    'outer: loop{
//        let current_char = text.char(rev_search_index);
//        if Self::is_quote_char(current_char){
//            let quote_char = current_char;
//            let mut match_stack = Vec::new();
//            let mut search_index = rev_search_index;
//            'inner: loop{
//                let current_char = text.char(search_index);
//                if current_char == quote_char{
//                    if match_stack.is_empty(){
//                        match_stack.push(current_char);
//                    }
//                    else{
//                        return vec![
//                            Selection::new(Range::new(rev_search_index, text_util::next_grapheme_index(rev_search_index, text)), Direction::Forward),
//                            Selection::new(Range::new(search_index, text_util::next_grapheme_index(search_index, text)), Direction::Forward)
//                        ];
//                    }
//                }
//                search_index = search_index + 1;
//                if search_index >= text.len_chars(){break 'outer;}
//            }
//        }
//        rev_search_index = rev_search_index.saturating_sub(1);
//        if rev_search_index == 0{break 'outer;}
//    }
//    Vec::new()
//}
//fn is_quote_char(char: char) -> bool{
//    char == '\''
//    || char == '"'
//}
//
//TODO: impl and test
//TODO: future improvement: for each char search loop, spawn a thread to do the search, so we can process them simultaneously.
//TODO: error if searching backwards and reach previous selection range end, or if searching forward and reach next selection range start   //maybe this logic needs to be in selections
    //should operate over a rope slice from (start of doc if no previous selection, or previous selection end) to (end of doc text if no next selection, or next selection start)
// Returns a new [`Selection`] inside but excluding specified input char.
//pub fn select_inside_instances_of_single_char(&self, input: char, text: &Rope) -> Result<Self, SelectionError>{     //TODO: this is really more of a "search around selection for instances of single char"
//    let mut new_selection = self.clone();
//    
//    //second version
//    let mut found_backward = false;
//    //for (i, current_char) in text.slice(0..self.range.start).to_string().chars().rev().enumerate(){ //can this be done without converting to string?...
//    for (i, &current_char) in text.slice(0..self.range.start).chars().collect::<Vec<_>>().iter().rev().enumerate(){
//        if current_char == input{
//            new_selection.range.start = new_selection.range.start.saturating_sub(i);// - (i+1);
//            found_backward = true;
//            break;
//        }
//    }
//    
//    let mut found_forward = false;
//    for (i, current_char) in text.slice(self.range.end..).chars().enumerate(){
//        if current_char == input{
//            new_selection.range.end = new_selection.range.end.saturating_add(i);// + (i-1);
//            found_forward = true;
//            break;
//        }
//    }
//
//    if found_forward && found_backward{
//        Ok(new_selection)
//    }else{
//        Err(SelectionError::ResultsInSameState)
//    }
//}
// Returns a new [`Selection`] inside but excluding specified char pair.
//pub fn select_inside_pair(&self, leading_char: char, trailing_char: char, text: &Rope) -> Result<Self, SelectionError>{     //TODO: this is really more of a "search around selection for char pair"
//    let mut new_selection = self.clone();
//
//    let mut found_backward = false;
//    for (i, &current_char) in text.slice(0..self.range.start).chars().collect::<Vec<_>>().iter().rev().enumerate(){
//        println!("backward: {current_char} at {i}");
//        if current_char == leading_char{
//            new_selection.range.start = new_selection.range.start.saturating_sub(i);// - (i+1);
//            found_backward = true;
//            break;
//        }
//    }
//    let mut found_forward = false;
//    for (i, current_char) in text.slice(self.range.end..).chars().enumerate(){
//        println!("forward: {current_char} at {i}");
//        if current_char == trailing_char{
//            new_selection.range.end = new_selection.range.end.saturating_add(i);// + (i-1);
//            found_forward = true;
//            break;
//        }
//    }
//
//    if found_forward && found_backward{
//        Ok(new_selection)
//    }else{
//        Err(SelectionError::ResultsInSameState)
//    }
//}
//fn select_inside_text_object(){}    //for paragraphs, words, and the like
//
//TODO: make pub fn select_until    //extend selection until provided character/string is selected (should have one for forwards and one for backwards)

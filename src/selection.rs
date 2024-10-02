use ropey::Rope;
use crate::{
    Position,
    view::View,
    text_util
};



// if a block cursor is treated as a bar cursor with its head extended 1 grapheme to the right,
// i think these can be treated equally for cursor movements.(maybe excluding end of file?)
// however, they may need to behave differently for selection extension
#[derive(Clone, Copy)]
pub enum CursorSemantics{
    // default selection has width of 0
    Bar,
    // default selection has width of 1
    Block
}

#[derive(PartialEq, Debug)]
pub enum Direction{
    Forward,
    Backward,
}
#[derive(PartialEq)]
pub enum Movement{
    Extend,
    Move,
}

/// 1 dimensional representation of a single selection(between anchor and head) within a text rope.
/// a cursor is a selection with an anchor/head difference of 0 or 1(depending on cursor semantics)
/// Should ensure head/anchor are always within text bounds
#[derive(PartialEq, Clone, Debug)]
pub struct Selection{
    /// the stationary portion of a selection.
    anchor: usize,
    /// the mobile portion of a selection. this is the portion a user can move to extend selection
    head: usize,
    /// the offset from the start of the line self.head is on
    stored_line_position: Option<usize>,
}
impl Selection{
    /// Creates a new instance of [Selection].
    pub fn new(anchor: usize, head: usize) -> Self{
        Self{anchor, head, stored_line_position: None}
    }
    /// Creates an instance of [Selection] with a specified stored_line_position.
    /// Mainly used for testing.
    pub fn with_stored_line_position(anchor: usize, head: usize, stored_line_position: usize) -> Self{
        Self{anchor, head, stored_line_position: Some(stored_line_position)}
    }
    pub fn anchor(&self) -> usize{
        self.anchor
    }
    pub fn head(&self) -> usize{
        self.head
    }

    /// Start of the [Selection] from left to right.
    /// ```
    /// # use edit::selection::Selection;
    /// 
    /// fn test(selection: Selection, expected: usize) -> bool{
    ///     let result = selection.start();
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 4), 0));
    /// assert!(test(Selection::new(4, 0), 0));
    /// ```
    pub fn start(&self) -> usize{
        std::cmp::min(self.anchor, self.head)
    }
    /// End of the [Selection] from left to right.
    /// ```
    /// # use edit::selection::Selection;
    /// 
    /// fn test(selection: Selection, expected: usize) -> bool{
    ///     let result = selection.end();
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 4), 4));
    /// assert!(test(Selection::new(4, 0), 4));
    /// ```
    pub fn end(&self) -> usize{
        std::cmp::max(self.anchor, self.head)
    }

    /// Returns true if selection > 0 with bar cursor semantics, or 
    /// selection > 1 with block cursor semantics, or else returns false.
    /// ```
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(selection: Selection, expected: bool, semantics: CursorSemantics) -> bool{
    ///     let result = selection.is_extended(semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), false, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), true, CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 0), true, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), false, CursorSemantics::Block));
    /// //assert!(test(Selection::new(1, 0), false, CursorSemantics::Block)); //currently failing
    /// assert!(test(Selection::new(0, 2), true, CursorSemantics::Block));
    /// assert!(test(Selection::new(2, 0), true, CursorSemantics::Block));
    /// ```
    pub fn is_extended(&self, semantics: CursorSemantics) -> bool{
        self.anchor != self.cursor(semantics)
        //match semantics{
        //    CursorSemantics::Bar => self.len() > 0,   
        //    CursorSemantics::Block => self.len() > 1  //if selection is greater than one grapheme //currently uses char count though...
        //}
    }

    /// returns the direction of [Selection]
    /// ```
    /// # use edit::selection::{Selection, Direction, CursorSemantics};
    /// 
    /// fn test(selection: Selection, expected: Direction, semantics: CursorSemantics) -> bool{
    ///     let result = selection.direction(semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 0), Direction::Backward, CursorSemantics::Bar));
    /// //assert!(test(Selection::new(0, 0), Direction::Backward, CursorSemantics::Block)); //state should't be possible with block cursor semantics, so this failure is fine
    /// assert!(test(Selection::new(0, 1), Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 0), Direction::Backward, CursorSemantics::Block));
    /// ```
    pub fn direction(&self, semantics: CursorSemantics) -> Direction{
        if self.cursor(semantics) < self.anchor{
            Direction::Backward
        }else{
            Direction::Forward
        }
    }

    /// Sets [Selection] direction to specified direction.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Direction, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, text: &Rope, direction: Direction, semantics: CursorSemantics) -> bool{
    ///     selection.set_direction(direction, text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), &text, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), &text, Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 5), Selection::with_stored_line_position(5, 0, 0), &text, Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(5, 0), Selection::with_stored_line_position(0, 5, 1), &text, Direction::Forward, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(1, 0, 0), &text, Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 0), Selection::with_stored_line_position(0, 1, 0), &text, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 5), Selection::with_stored_line_position(5, 0, 0), &text, Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(5, 0), Selection::with_stored_line_position(0, 5, 0), &text, Direction::Forward, CursorSemantics::Block));
    /// ```
    pub fn set_direction(&mut self, direction: Direction, text: &Rope, semantics: CursorSemantics){
        match direction{
            Direction::Forward => {
                let new_anchor = self.start();
                let new_head = self.end();
                self.anchor = new_anchor;
                self.head = new_head;
            }
            Direction::Backward => {
                let new_anchor = self.end();
                let new_head = self.start();
                self.anchor = new_anchor;
                self.head = new_head;
            }
        }
        self.stored_line_position = Some(text_util::offset_from_line_start(self.cursor(semantics), text));
    }

    /// Checks self and other for overlap.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// fn test(selection: Selection, other: Selection, expected: bool) -> bool{
    ///     let result = selection.overlaps(other);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // test key: 
    /// //    selection1 anchor = [
    /// //    selection1 head   = ]
    /// //    selection2 anchor = <
    /// //    selection2 head   = >
    /// 
    /// // non zero width selections, no overlap
    /// assert!(test(Selection::new(0, 3), Selection::new(3, 6), false));   //[idk]<\nso>me\nshit\n
    /// assert!(test(Selection::new(0, 3), Selection::new(6, 3), false));   //[idk]>\nso<me\nshit\n
    /// assert!(test(Selection::new(3, 0), Selection::new(3, 6), false));   //]idk[<\nso>me\nshit\n
    /// assert!(test(Selection::new(3, 0), Selection::new(6, 3), false));   //]idk[>\nso<me\nshit\n
    /// assert!(test(Selection::new(3, 6), Selection::new(0, 3), false));   //<idk>[\nso]me\nshit\n
    /// assert!(test(Selection::new(3, 6), Selection::new(3, 0), false));   //>idk<[\nso]me\nshit\n
    /// assert!(test(Selection::new(6, 3), Selection::new(0, 3), false));   //<idk>]\nso[me\nshit\n
    /// assert!(test(Selection::new(6, 3), Selection::new(3, 0), false));   //>idk<]\nso[me\nshit\n
    /// 
    /// // non-zero-width selections, overlap.
    /// assert!(test(Selection::new(0, 4), Selection::new(3, 6), true));   //[idk<\n]so>me\nshit\n
    /// assert!(test(Selection::new(0, 4), Selection::new(6, 3), true));   //[idk>\n]so<me\nshit\n
    /// assert!(test(Selection::new(4, 0), Selection::new(3, 6), true));   //]idk<\n[so>me\nshit\n
    /// assert!(test(Selection::new(4, 0), Selection::new(6, 3), true));   //]idk>\n[so<me\nshit\n
    /// assert!(test(Selection::new(3, 6), Selection::new(0, 4), true));   //<idk[\n>so]me\nshit\n
    /// assert!(test(Selection::new(3, 6), Selection::new(4, 0), true));   //>idk[\n<so]me\nshit\n
    /// assert!(test(Selection::new(6, 3), Selection::new(0, 4), true));   //<idk]\n>so[me\nshit\n
    /// assert!(test(Selection::new(6, 3), Selection::new(4, 0), true));   //>idk]\n<so[me\nshit\n
    /// 
    /// // Zero-width and non-zero-width selections, no overlap.    //i think this should count as overlap...
    ///// assert!(test(Selection::new(0, 3), Selection::new(3, 3), false));   //[idk]<>\nsome\nshit\n
    ///// assert!(test(Selection::new(3, 0), Selection::new(3, 3), false));   //]idk[<>\nsome\nshit\n
    ///// assert!(test(Selection::new(3, 3), Selection::new(0, 3), false));   //<idk>[]\nsome\nshit\n
    ///// assert!(test(Selection::new(3, 3), Selection::new(3, 0), false));   //>idk<[]\nsome\nshit\n
    /// assert!(test(Selection::new(0, 3), Selection::new(3, 3), true));   //[idk<>]\nsome\nshit\n
    /// assert!(test(Selection::new(3, 0), Selection::new(3, 3), true));   //]idk<>[\nsome\nshit\n
    /// assert!(test(Selection::new(3, 3), Selection::new(0, 3), true));   //<idk[]>\nsome\nshit\n
    /// assert!(test(Selection::new(3, 3), Selection::new(3, 0), true));   //>idk[]<\nsome\nshit\n
    /// 
    /// // Zero-width and non-zero-width selections, overlap.
    /// assert!(test(Selection::new(1, 4), Selection::new(1, 1), true));    //i[<>dk\n]some\nshit\n
    /// assert!(test(Selection::new(4, 1), Selection::new(1, 1), true));    //i]<>dk\n[some\nshit\n
    /// assert!(test(Selection::new(1, 1), Selection::new(1, 4), true));    //i[<]dk\n>some\nshit\n
    /// assert!(test(Selection::new(1, 1), Selection::new(4, 1), true));    //i[>]dk\n<some\nshit\n
    /// assert!(test(Selection::new(1, 4), Selection::new(3, 3), true));    //i[dk<>\n]some\nshit\n
    /// assert!(test(Selection::new(4, 1), Selection::new(3, 3), true));    //i]dk<>\n[some\nshit\n
    /// assert!(test(Selection::new(3, 3), Selection::new(1, 4), true));    //i<dk[]\n>some\nshit\n
    /// assert!(test(Selection::new(3, 3), Selection::new(4, 1), true));    //i>dk[]\n<some\nshit\n
    /// 
    /// // zero-width selections, no overlap.
    /// assert!(test(Selection::new(0, 0), Selection::new(1, 1), false));   //[]i<>dk\nsome\nshit\n
    /// assert!(test(Selection::new(1, 1), Selection::new(0, 0), false));   //<>i[]dk\nsome\nshit\n
    /// 
    /// // zero-width selections, overlap.
    /// assert!(test(Selection::new(1, 1), Selection::new(1, 1), true));    //i[<>]dk\nsome\nshit\n
    /// ```
    pub fn overlaps(&self, other: Selection) -> bool{
        self.start() == other.start() || 
        self.end() == other.end() || 
        (self.end() > other.start() && other.end() > self.start())
    }

    /// Create a new [Selection] by merging self with other.
    ///// Indiscriminate merge. merges whether overlapping, consecutive, 
    ///// contained, or disconnected entirely.
    /// resultant selection should be guaranteed to be within text bounds 
    /// because this uses previously initialized selections.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// fn test(selection: Selection, other: Selection, expected: Selection, text: &Rope) -> bool{
    ///     let result = selection.merge(&other, text);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // when self.anchor > self.head && other.anchor > other.head
    /// assert!(test(Selection::new(4, 0), Selection::new(5, 1), Selection::with_stored_line_position(0, 5, 1), &text));
    /// assert!(test(Selection::new(5, 1), Selection::new(4, 0), Selection::with_stored_line_position(0, 5, 1), &text));
    /// 
    /// // when self.anchor < self.head && other.anchor < other.head
    /// assert!(test(Selection::new(0, 4), Selection::new(1, 5), Selection::with_stored_line_position(0, 5, 1), &text));
    /// assert!(test(Selection::new(1, 5), Selection::new(0, 4), Selection::with_stored_line_position(0, 5, 1), &text));
    /// 
    /// // when self.anchor > self.head && other.anchor < other.head
    /// assert!(test(Selection::new(4, 0), Selection::new(1, 5), Selection::with_stored_line_position(0, 5, 1), &text));
    /// assert!(test(Selection::new(1, 5), Selection::new(4, 0), Selection::with_stored_line_position(0, 5, 1), &text));
    /// 
    /// // when self.anchor < self.head && other.anchor > other.head
    /// assert!(test(Selection::new(0, 4), Selection::new(5, 1), Selection::with_stored_line_position(0, 5, 1), &text));
    /// assert!(test(Selection::new(5, 1), Selection::new(0, 4), Selection::with_stored_line_position(0, 5, 1), &text));
    /// 
    /// // consecutive
    /// assert!(test(Selection::new(0, 1), Selection::new(1, 2), Selection::with_stored_line_position(0, 2, 2), &text));
    /// assert!(test(Selection::new(1, 0), Selection::new(1, 2), Selection::with_stored_line_position(0, 2, 2), &text));
    /// assert!(test(Selection::new(1, 0), Selection::new(2, 1), Selection::with_stored_line_position(0, 2, 2), &text));
    /// assert!(test(Selection::new(0, 1), Selection::new(2, 1), Selection::with_stored_line_position(0, 2, 2), &text));
    /// assert!(test(Selection::new(1, 2), Selection::new(0, 1), Selection::with_stored_line_position(0, 2, 2), &text));
    /// assert!(test(Selection::new(2, 1), Selection::new(0, 1), Selection::with_stored_line_position(0, 2, 2), &text));
    /// assert!(test(Selection::new(2, 1), Selection::new(1, 0), Selection::with_stored_line_position(0, 2, 2), &text));
    /// assert!(test(Selection::new(1, 2), Selection::new(1, 0), Selection::with_stored_line_position(0, 2, 2), &text));
    ///
    /// // overlapping
    /// assert!(test(Selection::new(0, 2), Selection::new(1, 4), Selection::with_stored_line_position(0, 4, 0), &text));
    /// assert!(test(Selection::new(2, 0), Selection::new(1, 4), Selection::with_stored_line_position(0, 4, 0), &text));
    /// assert!(test(Selection::new(2, 0), Selection::new(4, 1), Selection::with_stored_line_position(0, 4, 0), &text));
    /// assert!(test(Selection::new(0, 2), Selection::new(4, 1), Selection::with_stored_line_position(0, 4, 0), &text));
    /// assert!(test(Selection::new(1, 4), Selection::new(0, 2), Selection::with_stored_line_position(0, 4, 0), &text));
    /// assert!(test(Selection::new(4, 1), Selection::new(0, 2), Selection::with_stored_line_position(0, 4, 0), &text));
    /// assert!(test(Selection::new(4, 1), Selection::new(2, 0), Selection::with_stored_line_position(0, 4, 0), &text));
    /// assert!(test(Selection::new(1, 4), Selection::new(2, 0), Selection::with_stored_line_position(0, 4, 0), &text));
    /// 
    /// // contained
    /// assert!(test(Selection::new(0, 6), Selection::new(2, 4), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(6, 0), Selection::new(2, 4), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(6, 0), Selection::new(4, 2), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(0, 6), Selection::new(4, 2), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(2, 4), Selection::new(0, 6), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(4, 2), Selection::new(0, 6), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(4, 2), Selection::new(6, 0), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(2, 4), Selection::new(6, 0), Selection::with_stored_line_position(0, 6, 2), &text));
    /// 
    /// // disconnected
    /// assert!(test(Selection::new(0, 2), Selection::new(4, 6), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(2, 0), Selection::new(4, 6), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(2, 0), Selection::new(6, 4), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(0, 2), Selection::new(6, 4), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(4, 6), Selection::new(0, 2), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(6, 4), Selection::new(0, 2), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(6, 4), Selection::new(2, 0), Selection::with_stored_line_position(0, 6, 2), &text));
    /// assert!(test(Selection::new(4, 6), Selection::new(2, 0), Selection::with_stored_line_position(0, 6, 2), &text));
    /// ```
    pub fn merge(&self, other: &Selection, text: &Rope) -> Selection{
        let anchor = self.start().min(other.start());
        let head = self.end().max(other.end());
        let stored_line_position = text_util::offset_from_line_start(head, text);   //self.cursor instead of head?
            
        Selection{anchor, head, stored_line_position: Some(stored_line_position)}
    }

    /////////////////////////////////// Alignment Methods ///////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////////////

    /////////////////////////////////// Block Cursor Methods ///////////////////////////////////
    
    /// Returns the char offset of the cursor.
    /// left side of cursor if block cursor semantics
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(selection: Selection, expected: usize, semantics: CursorSemantics) -> bool{
    ///     let result = selection.cursor(semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // key:
    /// // anchor             = [
    /// // head               = ]
    /// // block_virtual_head = :
    /// 
    /// assert!(test(Selection::new(1, 2), 1, CursorSemantics::Block));    //i[:d]k\nsome\nshit\n
    /// assert!(test(Selection::new(2, 1), 1, CursorSemantics::Block));    //i:]d[k\nsome\nshit\n
    /// assert!(test(Selection::new(2, 2), 1, CursorSemantics::Block));    //i:d][k\nsome\nshit\n   //though this state should be impossible with block cursor semantics
    /// assert!(test(Selection::new(0, 0), 0, CursorSemantics::Bar));    //[]idk\nsome\nshit\n
    /// ```
    pub fn cursor(&self, semantics: CursorSemantics) -> usize{
        match semantics{
            CursorSemantics::Bar => self.head,
            CursorSemantics::Block => {
                if self.head >= self.anchor{
                    self.head.saturating_sub(1)   //prev_grapheme_boundary(text, self.head)
                }else{
                    self.head
                }
            }
        }
    }

    /// Moves cursor to specified char offset in rope.
    /// Will shift anchor/head positions to accommodate Bar/Block cursor semantics.
    ///```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Movement, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, to: usize, text: &Rope, movement: Movement, semantics: CursorSemantics) -> bool{
    ///     selection.put_cursor(to, text, movement, semantics, true);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(5, 5, 1), 5, &text, Movement::Move, CursorSemantics::Bar));
    /// assert!(test(Selection::new(5, 5), Selection::with_stored_line_position(0, 0, 0), 0, &text, Movement::Move, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 5, 1), 5, &text, Movement::Extend, CursorSemantics::Bar));
    /// assert!(test(Selection::new(5, 5), Selection::with_stored_line_position(5, 0, 0), 0, &text, Movement::Extend, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(5, 6, 1), 5, &text, Movement::Move, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 0), Selection::with_stored_line_position(5, 6, 1), 5, &text, Movement::Move, CursorSemantics::Block));
    /// assert!(test(Selection::new(5, 6), Selection::with_stored_line_position(0, 1, 0), 0, &text, Movement::Move, CursorSemantics::Block));
    /// assert!(test(Selection::new(6, 5), Selection::with_stored_line_position(0, 1, 0), 0, &text, Movement::Move, CursorSemantics::Block));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 6, 1), 5, &text, Movement::Extend, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 0), Selection::with_stored_line_position(0, 6, 1), 5, &text, Movement::Extend, CursorSemantics::Block));
    /// assert!(test(Selection::new(5, 6), Selection::with_stored_line_position(6, 0, 0), 0, &text, Movement::Extend, CursorSemantics::Block));
    /// assert!(test(Selection::new(6, 5), Selection::with_stored_line_position(6, 0, 0), 0, &text, Movement::Extend, CursorSemantics::Block));
    /// 
    /// // test putting cursor at end of text
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(14, 14, 0), 14, &text, Movement::Move, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 14, 0), 14, &text, Movement::Extend, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(14, 15, 0), 14, &text, Movement::Move, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 15, 0), 14, &text, Movement::Extend, CursorSemantics::Block));
    /// ```
    pub fn put_cursor(&mut self, to: usize, text: &Rope, movement: Movement, semantics: CursorSemantics, update_stored_line_position: bool){    //could also just update stored_line_position in calling fn after this call...
        match semantics{
            CursorSemantics::Bar => {
                if movement == Movement::Move{  //intentionally disregarding Movement::Extend
                    self.anchor = to;
                }
                self.head = to;
            }
            CursorSemantics::Block => {
                match movement{
                    Movement::Move => {
                        self.anchor = to;
                        //self.head = to.saturating_add(1).min(text.len_chars());
                        self.head = to.saturating_add(1).min(text.len_chars().saturating_add(1));   //allowing one more char past text.len_chars() for block cursor
                    }
                    Movement::Extend => {
                        let new_anchor = if self.head >= self.anchor && to < self.anchor{
                            self.anchor.saturating_add(1).min(text.len_chars())
                        }else if self.head < self.anchor && to >= self.anchor{
                            self.anchor.saturating_sub(1)
                        }else{
                            self.anchor
                        };

                        if new_anchor <= to{
                            self.anchor = new_anchor;
                            //self.head = to.saturating_add(1).min(text.len_chars());
                            self.head = to.saturating_add(1).min(text.len_chars().saturating_add(1))    //allowing one more char past text.len_chars() for block cursor
                        }else{
                            self.anchor = new_anchor;
                            self.head = to;
                        }
                    }
                }
            }
        }
        if update_stored_line_position{
            self.stored_line_position = Some(text_util::offset_from_line_start(self.cursor(semantics), text));
        }
    }

    /////////////////////////////////// Movement Methods ///////////////////////////////////

    /// Moves the cursor vertically.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Movement, Direction, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, amount: usize, text: &Rope, movement: Movement, direction: Direction, semantics: CursorSemantics) -> bool{
    ///     selection.move_vertically(amount, text, movement, direction, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsomething\nelse\n");
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(4, 4, 0), 1, &text, Movement::Move, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(0, 0, 0), 1, &text, Movement::Move, Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 4, 0), 1, &text, Movement::Extend, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(4, 0, 0), 1, &text, Movement::Extend, Direction::Backward, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(4, 5, 0), 1, &text, Movement::Move, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(4, 5), Selection::with_stored_line_position(0, 1, 0), 1, &text, Movement::Move, Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 5, 0), 1, &text, Movement::Extend, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(4, 5), Selection::with_stored_line_position(5, 0, 0), 1, &text, Movement::Extend, Direction::Backward, CursorSemantics::Block));
    /// ```
    pub fn move_vertically(&mut self, amount: usize, text: &Rope, movement: Movement, direction: Direction, semantics: CursorSemantics){
        let goal_line_number = match direction{
            Direction::Forward => text.char_to_line(self.cursor(semantics)).saturating_add(amount).min(text.len_lines().saturating_sub(1)),
            Direction::Backward => text.char_to_line(self.cursor(semantics)).saturating_sub(amount)
        };
        
        let start_of_line = text.line_to_char(goal_line_number);
        let line_width = text_util::line_width_excluding_newline(text.line(goal_line_number));
        
        let stored_line_position = match self.stored_line_position{
            Some(stored_line_position) => stored_line_position,
            None => text_util::offset_from_line_start(self.cursor(semantics), text)
        };
        
        let new_position = if stored_line_position < line_width{
            start_of_line + stored_line_position
        }else{
            start_of_line + line_width
        };

        self.stored_line_position = Some(stored_line_position);
        self.put_cursor(new_position, text, movement, semantics, false);
    }

    /// Moves the cursor horizontally.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Movement, Direction, CursorSemantics};
    /// 
    /// let text = Rope::from("idk\nsomething\nelse\n");    //len 19
    /// 
    /// fn test(mut selection: Selection, expected: Selection, amount: usize, text: &Rope, movement: Movement, direction: Direction, semantics: CursorSemantics) -> bool{
    ///     selection.move_horizontally(amount, text, movement, direction, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(1, 1, 1), 1, &text, Movement::Move, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 1), Selection::with_stored_line_position(0, 0, 0), 1, &text, Movement::Move, Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 1, 1), 1, &text, Movement::Extend, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 1), Selection::with_stored_line_position(1, 0, 0), 1, &text, Movement::Extend, Direction::Backward, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(1, 2, 1), 1, &text, Movement::Move, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 2), Selection::with_stored_line_position(0, 1, 0), 1, &text, Movement::Move, Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 2, 1), 1, &text, Movement::Extend, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 2), Selection::with_stored_line_position(2, 0, 0), 1, &text, Movement::Extend, Direction::Backward, CursorSemantics::Block));
    /// 
    /// // handles moving to end of text correctly
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(19, 19, 0), 19, &text, Movement::Move, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 19, 0), 19, &text, Movement::Extend, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(19, 20, 0), 19, &text, Movement::Move, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 20, 0), 19, &text, Movement::Extend, Direction::Forward, CursorSemantics::Block));
    /// ```
    pub fn move_horizontally(&mut self, amount: usize, text: &Rope, movement: Movement, direction: Direction, semantics: CursorSemantics){
        let new_position = match direction{
            Direction::Forward => self.cursor(semantics).saturating_add(amount).min(text.len_chars()),    //ensures this does not move past text end
            Direction::Backward => self.cursor(semantics).saturating_sub(amount)
        };
        self.put_cursor(new_position, text, movement, semantics, true);
    }

    /// Aligns [Selection] anchor with head.
    /// #### Invariants:
    /// - selection is within text bounds
    /// # Examples
    /// ``` 
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, text: &Rope, semantics: CursorSemantics) -> bool{
    ///     selection.collapse(text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // head < anchor
    /// assert!(test(Selection::new(4, 0), Selection::with_stored_line_position(0, 0, 0), &text, CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 0), Selection::with_stored_line_position(0, 1, 0), &text, CursorSemantics::Block));
    /// 
    /// // anchor < head
    /// assert!(test(Selection::new(0, 4), Selection::with_stored_line_position(4, 4, 0), &text, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 4), Selection::with_stored_line_position(4, 5, 0), &text, CursorSemantics::Block));
    /// 
    /// // test setting cursor to end of text
    /// assert!(test(Selection::new(0, 14), Selection::with_stored_line_position(14, 14, 0), &text, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 14), Selection::with_stored_line_position(14, 15, 0), &text, CursorSemantics::Block));
    /// ```
    pub fn collapse(&mut self, text: &Rope, semantics: CursorSemantics){
        self.put_cursor(self.head, text, Movement::Move, semantics, true);
    }

    /// Moves cursor right.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection is within doc bounds
    /// - TODO: selection is grapheme aligned
    /// 
    /// # Examples
    /// ``` 
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// fn test(mut selection: Selection, expected: Selection, text: &Rope) -> bool{
    ///     selection.move_right(text);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("012\n");
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(4, 4, 0), &text));
    /// 
    /// // normal use
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(1, 1, 1), &text));
    /// 
    /// // new line resets stored line position
    /// let text = Rope::from("012\n0");
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(4, 4, 0), &text));
    /// ```
    pub fn move_right(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        self.move_horizontally(1, text, Movement::Move, Direction::Forward, CursorSemantics::Bar);
    }

    /// Moves cursor left.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection is within doc bounds
    /// - TODO: selection is grapheme aligned
    /// # Examples
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// fn test(mut selection: Selection, expected: Selection, text: &Rope) -> bool{
    ///     selection.move_left(text);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), &text));
    /// 
    /// // normal use
    /// assert!(test(Selection::new(2, 2), Selection::with_stored_line_position(1, 1, 1), &text));
    /// 
    /// // move to previous line resets stored line position
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(3, 3, 3), &text));
    /// ```
    pub fn move_left(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        self.move_horizontally(1, text, Movement::Move, Direction::Backward, CursorSemantics::Bar);
    }

    /// Moves cursor up.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection is within doc bounds
    /// - TODO: selection is grapheme aligned
    /// # Examples
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// fn test(mut selection: Selection, expected: Selection, text: &Rope) -> bool{
    ///     selection.move_up(text);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), &text));
    /// 
    /// // to shorter line
    /// assert!(test(Selection::new(13, 13), Selection::with_stored_line_position(3, 3, 9), &text));
    /// 
    /// // to longer line
    /// assert!(test(Selection::new(18, 18), Selection::with_stored_line_position(8, 8, 4), &text));
    /// ```
    pub fn move_up(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        self.move_vertically(1, text, Movement::Move, Direction::Backward, CursorSemantics::Bar);
    }

    /// Moves cursor down.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection stays within text bounds
    /// - selection preserves stored line position
    /// # Examples
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// fn test(mut selection: Selection, expected: Selection, text: &Rope) -> bool{
    ///     selection.move_down(text);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(14, 14), Selection::with_stored_line_position(14, 14, 0), &text));
    /// 
    /// // to longer line
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(7, 7, 3), &text));
    /// 
    /// // to shorter line
    /// let text = Rope::from("012\n0");
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(5, 5, 3), &text));
    /// ```
    pub fn move_down(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        self.move_vertically(1, text, Movement::Move, Direction::Forward, CursorSemantics::Bar);
    }

    /// Moves cursor to line end.
    /// #### Invariants:
    /// - selection is collapsed
    /// - stays within doc bounds
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\n");
    /// let mut selection = Selection::new(0, 0);                               //[]idk\n
    /// let expected_selection = Selection::with_stored_line_position(3, 3, 3); //idk[]\n
    /// selection.move_line_text_end(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_line_text_end(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        let line_number = text.char_to_line(self.head);
        let line = text.line(line_number);
        let line_width = text_util::line_width_excluding_newline(line);
        let line_start = text.line_to_char(line_number);
        let line_end = line_start.saturating_add(line_width);

        self.put_cursor(line_end, text, Movement::Move, CursorSemantics::Bar, true);
    }

    /// Moves cursor to absolute start of line, or start of line text, depending on cursor position.
    /// #### Invariants:
    /// - selection is collapsed
    /// - stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// fn test(mut selection: Selection, expected: Selection, text: &Rope) -> bool{
    ///     selection.move_home(text);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// let text = Rope::from("    idk\n");
    /// 
    /// // moves to text start when cursor past text start
    /// assert!(test(Selection::new(6, 6), Selection::with_stored_line_position(4, 4, 4), &text));
    /// 
    /// // moves to line start when cursor at text start
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(0, 0, 0), &text));
    /// 
    /// // moves to text start when cursor before text start
    /// assert!(test(Selection::new(1, 1), Selection::with_stored_line_position(4, 4, 4), &text));
    /// ```
    pub fn move_home(&mut self, text: &Rope){
        let line_number = text.char_to_line(self.head);
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        if self.head == text_start{
            self.move_line_start(text);
        }else{
            self.move_line_text_start(text);
        }
    }
    
    /// Moves to line start.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selection = Selection::new(3, 3);
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);
    /// selection.move_line_start(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_line_start(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        let line_number = text.char_to_line(self.head);
        let line_start = text.line_to_char(line_number);

        self.put_cursor(line_start, text, Movement::Move, CursorSemantics::Bar, true);
    }
    
    /// Moves to start of text on line.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("  idk\n");
    /// let mut selection = Selection::new(0, 0);
    /// let expected_selection = Selection::with_stored_line_position(2, 2, 2);
    /// selection.move_line_text_start(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_line_text_start(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        let line_number = text.char_to_line(self.head);
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        self.put_cursor(text_start, text, Movement::Move, CursorSemantics::Bar, true);
    }

    /// Moves cursor up by the height of client view.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection stays within doc bounds
    /// - selection preserves stored line position
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// # use edit::view::View;
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// let client_view = View::new(0, 0, 2, 2);
    /// let mut selection = Selection::new(6, 6);                        //idk\nso[]mething\nelse
    /// let expected_selection = Selection::with_stored_line_position(2, 2, 2); //id[]k\nsomething\nelse
    /// selection.move_page_up(&text, &client_view);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_page_up(&mut self, text: &Rope, client_view: &View){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Move, Direction::Backward, CursorSemantics::Bar);
    }

    /// Moves cursor down by the height of client view.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection stays within doc bounds
    /// - selection preserves stored line position
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// # use edit::view::View;
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// let client_view = View::new(0, 0, 2, 2);
    /// let mut selection = Selection::new(0, 0);                               //[]idk\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 0); //idk\n[]something\nelse
    /// selection.move_page_down(&text, &client_view);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_page_down(&mut self, text: &Rope, client_view: &View){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Move, Direction::Forward, CursorSemantics::Bar);
    }

    /// Moves cursor to the start of the document.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\n");
    /// let mut selection = Selection::new(12, 12);
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);
    /// selection.move_doc_start(&text);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_doc_start(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        self.put_cursor(0, text, Movement::Move, CursorSemantics::Bar, true);
    }

    /// Moves cursor to the end of the document.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsome\nshit");
    /// let mut selection = Selection::new(0, 0);                                   //[]idk\nsome\nshit
    /// let expected_selection = Selection::with_stored_line_position(13, 13, 4);   //idk\nsome\nshit[]
    /// selection.move_doc_end(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_doc_end(&mut self, text: &Rope){
        if self.is_extended(CursorSemantics::Bar){
            self.collapse(text, CursorSemantics::Bar);
        }
        self.put_cursor(text.len_chars(), text, Movement::Move, CursorSemantics::Bar, true);
    }

//TODO: test if selection extended left, extend selection right reduces selection
    /// Extends selection to the right.
    /// #### Invariants:
    /// - selection stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("012\n");
    /// 
    /// // stays within bounds
    /// let mut selection = Selection::new(4, 4);                        //012\n[]
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 0); //012\n[]
    /// selection.extend_right(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // normal use
    /// let mut selection = Selection::new(0, 0);                               //[]012\n
    /// let expected_selection = Selection::with_stored_line_position(0, 1, 1); //[0]12\n
    /// selection.extend_right(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // resets stored line position after new line
    /// let text = Rope::from("012\n0");
    /// let mut selection = Selection::new(3, 3);                        //012[]\n0
    /// let expected_selection = Selection::with_stored_line_position(3, 4, 0); //012[\n]0
    /// selection.extend_right(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // debugging    //ensures repeated extension, and extension beyond a new line
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selection = Selection::new(0, 0);                               //[]idk\nsome\nshit\n
    /// let expected_selection = Selection::with_stored_line_position(0, 4, 0); //idk\n[]some\nshit\n
    /// selection.extend_right(&text);
    /// selection.extend_right(&text);
    /// selection.extend_right(&text);
    /// selection.extend_right(&text);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_right(&mut self, text: &Rope){
        self.move_horizontally(1, text, Movement::Extend, Direction::Forward, CursorSemantics::Bar);
    }

//TODO: test if selection extended right, extend selection left reduces selection
    /// Extends selection to the left.
    /// #### Invariants:
    /// - selection stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// 
    /// // stays within doc bounds
    /// let mut selection = Selection::new(0, 0);       //[]idk\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);  //[]idk\nsomething\nelse
    /// selection.extend_left(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // normal use
    /// let mut selection = Selection::new(2, 2);                        //id[]k\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(2, 1, 1); //i]d[k\nsomething\nelse
    /// selection.extend_left(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// //updates stored line position on line change
    /// let mut selection = Selection::new(4, 4);                        //idk\n[]something\nelse
    /// let expected_selection = Selection::with_stored_line_position(4, 3, 3); //idk]\n[something\nelse
    /// selection.extend_left(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_left(&mut self, text: &Rope){
        self.move_horizontally(1, text, Movement::Extend, Direction::Backward, CursorSemantics::Bar);
    }

    /// Extends selection up.
    /// #### Invariants:
    /// - selection stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// 
    /// // stays within doc bounds
    /// let mut selection = Selection::new(0, 0);       //[]idk\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);  //[]idk\nsomething\nelse
    /// selection.extend_up(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // to shorter line
    /// let mut selection = Selection::new(13, 13);                          //idk\nsomething[]\nelse
    /// let expected_selection = Selection::with_stored_line_position(13, 3, 9);    //idk]\nsomething[\nelse
    /// selection.extend_up(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // to longer line
    /// let mut selection = Selection::new(18, 18);                          //idk\nsomething\nelse[]
    /// let expected_selection = Selection::with_stored_line_position(18, 8, 4);    //idk\nsome]thing\nelse[
    /// selection.extend_up(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_up(&mut self, text: &Rope){
        self.move_vertically(1, text, Movement::Extend, Direction::Backward, CursorSemantics::Bar);
    }

    /// Extends selection down.
    /// #### Invariants:
    /// - selection stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// // stays within doc bounds
    /// let text = Rope::from("012\n");
    /// let mut selection = Selection::new(4, 4);                        //012\n[]
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 0); //012\n[]
    /// selection.extend_down(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // to shorter line
    /// let text = Rope::from("012\n0");
    /// let mut selection = Selection::new(3, 3);                        //012[]\n0
    /// let expected_selection = Selection::with_stored_line_position(3, 5, 3); //012[\n0]
    /// selection.extend_down(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // to longer line
    /// let text = Rope::from("idk\nsomething\nelse");
    /// let mut selection = Selection::new(3, 3);                        //idk[]\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(3, 7, 3); //idk[\nsom]ething\nelse
    /// selection.extend_down(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_down(&mut self, text: &Rope){
        self.move_vertically(1, text, Movement::Extend, Direction::Forward, CursorSemantics::Bar);
    }

    /// Extend selection to end of line text
    /// #### Invariants:
    /// - stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\n");
    /// let mut selection = Selection::new(0, 0);                               //[]idk\n
    /// let expected_selection = Selection::with_stored_line_position(0, 3, 3); //[idk]\n
    /// selection.extend_line_text_end(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_line_text_end(&mut self, text: &Rope){
        let line_number = text.char_to_line(self.head);
        let line = text.line(line_number);
        let line_width = text_util::line_width_excluding_newline(line);
        let line_start = text.line_to_char(line_number);
        let line_end = line_start.saturating_add(line_width);

        self.put_cursor(line_end, text, Movement::Extend, CursorSemantics::Bar, true);  //line_end.saturating_sub(1) for block cursor
    }

    /// Extends [Selection] to absolute start of line, or line text start, depending on [Selection] head position.
    /// #### Invariants:
    /// - selection stays within doc bounds
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("    idk\n");
    /// 
    /// // extends selection to text start when head past text start
    /// let mut selection = Selection::new(6, 6);                        //    id[]k\n
    /// let expected_selection = Selection::with_stored_line_position(6, 4, 4); //    ]id[k\n
    /// selection.extend_home(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // extends selection to line start when head at text start
    /// let mut selection = Selection::new(4, 4);                        //    []idk\n
    /// let expected_selection = Selection::with_stored_line_position(4, 0, 0); //]    [idk\n
    /// selection.extend_home(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // extends selection to text start when head before text start
    /// let mut selection = Selection::new(1, 1);                        // []   idk\n
    /// let expected_selection = Selection::with_stored_line_position(1, 4, 4); // [   ]idk\n
    /// selection.extend_home(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_home(&mut self, text: &Rope){
        let line_number = text.char_to_line(self.head);
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        if self.head == text_start{
            self.extend_line_start(text);
        }else{
            self.extend_line_text_start(text);
        }
    }
    
    /// Extends [Selection] to start of line.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selection = Selection::new(3, 3);
    /// let expected_selection = Selection::with_stored_line_position(3, 0, 0);
    /// selection.extend_line_start(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_line_start(&mut self, text: &Rope){
        let line_number = text.char_to_line(self.head);
        let line_start = text.line_to_char(line_number);

        self.put_cursor(line_start, text, Movement::Extend, CursorSemantics::Bar, true);
    }
    
    /// Extends [Selection] to start of text in line.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("  idk\n");
    /// let mut selection = Selection::new(0, 0);
    /// let expected_selection = Selection::with_stored_line_position(0, 2, 2);
    /// selection.extend_line_text_start(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_line_text_start(&mut self, text: &Rope){
        let line_number = text.char_to_line(self.head);
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        self.put_cursor(text_start, text, Movement::Extend, CursorSemantics::Bar, true);
    }
    
    /// Extends [Selection] up by the height of client view.
    /// #### Invariants:
    /// - selection stays within doc bounds
    /// - selection preserves stored line position
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// # use edit::view::View;
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// let client_view = View::new(0, 0, 2, 2);
    /// let mut selection = Selection::new(6, 6);                        //idk\nso[]mething\nelse
    /// let expected_selection = Selection::with_stored_line_position(6, 2, 2); //id]k\nso[mething\nelse
    /// selection.extend_page_up(&text, &client_view);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_page_up(&mut self, text: &Rope, client_view: &View){
        self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Extend, Direction::Backward, CursorSemantics::Bar);
    }
    
    /// Extends [Selection] down by the height of client view.
    /// #### Invariants:
    /// - selection stays within doc bounds
    /// - selection preserves stored line position
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// # use edit::view::View;
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// let client_view = View::new(0, 0, 2, 2);
    /// let mut selection = Selection::new(0, 0);                               //[]idk\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(0, 4, 0); //[idk\n]something\nelse
    /// selection.extend_page_down(&text, &client_view);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_page_down(&mut self, text: &Rope, client_view: &View){
        self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Extend, Direction::Forward, CursorSemantics::Bar);
    }
    
    /// Extends [Selection] to doc start.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selection = Selection::new(9, 9);
    /// let expected_selection = Selection::with_stored_line_position(9, 0, 0);
    /// selection.extend_doc_start(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_doc_start(&mut self, text: &Rope){
        self.put_cursor(0, text, Movement::Extend, CursorSemantics::Bar, true);
    }
    
    /// Extends [Selection] to doc end.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selection = Selection::new(0, 0);
    /// let expected_selection = Selection::with_stored_line_position(0, 14, 0);
    /// selection.extend_doc_end(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_doc_end(&mut self, text: &Rope){
        self.put_cursor(text.len_chars(), text, Movement::Extend, CursorSemantics::Bar, true);
    }

    /// Translates a [Selection] to a [Selection2d]
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selection2d};
    /// # use edit::Position;
    /// 
    /// fn test(selection: Selection, expected: Selection2d, text: &Rope) -> bool{
    ///     let result = selection.selection_to_selection2d(text);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsomething");
    /// 
    /// // when selection head/anchor same, and on same line
    /// //id[]k
    /// //something
    /// assert!(test(Selection::new(2, 2), Selection2d::new(Position::new(2, 0), Position::new(2, 0)), &text)); //id[]k\nsomething
    /// 
    /// // when selection head/anchor different, but on same line
    /// //i[d]k
    /// //something
    /// assert!(test(Selection::new(1, 2), Selection2d::new(Position::new(2, 0), Position::new(1, 0)), &text)); //i[d]k\nsomething
    /// 
    /// // when selection head/anchor same, but on new line
    /// //idk
    /// //[]something
    /// assert!(test(Selection::new(4, 4), Selection2d::new(Position::new(0, 1), Position::new(0, 1)), &text)); //idk\n[]something
    /// 
    /// // when selection head/anchor different, and on different lines
    /// //id[k
    /// //s]omething
    /// assert!(test(Selection::new(2, 5), Selection2d::new(Position::new(1, 1), Position::new(2, 0)), &text)); //id[k\ns]omething
    /// ```
    pub fn selection_to_selection2d(&self, text: &Rope) -> Selection2d{
        let line_number_head = text.char_to_line(self.head);
        let line_number_anchor = text.char_to_line(self.anchor);

        let head_line_start_idx = text.line_to_char(line_number_head);
        let anchor_line_start_idx = text.line_to_char(line_number_anchor);

        Selection2d::new(
            Position::new(
                self.head - head_line_start_idx, 
                line_number_head
            ), 
            Position::new(
                self.anchor - anchor_line_start_idx, 
                line_number_anchor
            )
        )
    }
}



/// 2 dimensional representation of a single selection(between anchor and head) within document text
#[derive(Default, PartialEq, Debug)]
pub struct Selection2d{
    head: Position,
    anchor: Position,
}
impl Selection2d{
    pub fn new(head: Position, anchor: Position) -> Self{
        Self{
            head,
            anchor
        }
    }
    pub fn head(&self) -> &Position{
        &self.head
    }
    pub fn anchor(&self) -> &Position{
        &self.anchor
    }
}



/// A collection of [Selection]s. 
/// used in place of [Vec]<[Selection]> to ensure certain guarantees are enforced
/// ## Goal Guarantees:
/// - will always contain at least 1 {Selection}
/// - all {Selection}s are grapheme aligned
/// - all {Selection}s are sorted by increasing position in document
/// - all overlapping {Selection}s are merged
    //should this be handled in {Selection}?
/// - head and anchor are always within text boundaries for each selection
    //
/// - ...prob others i haven't thought of yet
#[derive(Debug, PartialEq, Clone)]
pub struct Selections{
    selections: Vec<Selection>,
    primary_selection_index: usize,
}
impl Selections{
    /// Returns new [Selections] from provided input.
    /// #### Invariants:
    /// - will alway contain at least one [Selection]
    /// - [Selection]s are grapheme aligned
    /// - [Selection]s are sorted by ascending position in doc
    /// - overlapping [Selection]s are merged
    /// - all [Selection]s are within doc boundaries
    /// 
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// // sorts and merges overlapping
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(2, 4),    // i d[k \n]s o m e \n s h i t \n
    ///     Selection::new(0, 5),    //[i d k \n s]o m e \n s h i t \n
    ///     Selection::new(3, 6)     // i d k[\n s o]m e \n s h i t \n
    /// ], 0, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::with_stored_line_position(0, 6, 2)     //[i d k \n s o]m e \n s h i t \n
    /// ], 0, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn new(mut selections: Vec<Selection>, mut primary_selection_index: usize, text: &Rope) -> Self{
        if selections.is_empty(){
            selections = vec![Selection::new(0, 0)];
            primary_selection_index = 0;
        }

        let mut selections = Self{
            selections,
            primary_selection_index,
        };

        // selections.grapheme_align();
        selections.sort();
        selections.merge_overlapping(text);

        selections
    }
    pub fn primary_selection_index(&self) -> usize{
        self.primary_selection_index
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Selection>{
        self.selections.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Selection>{
        self.selections.iter_mut()
    }
    pub fn pop(&mut self) -> Option<Selection>{
        //TODO: figure out how to determine what to set primary_selection_index to
        if self.selections.len() == 1{
            None
        }else{
            self.selections.pop()
        }
    }

    /// Prepends a [Selection] to the front of [Self], and assigns 0 to self.primary_selection_index
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![Selection::new(4, 4)], 0, &text);
    /// selections.push_front(Selection::new(0, 0));
    /// let expected_selections = Selections::new(vec![Selection::new(0, 0), Selection::new(4, 4)], 0, &text);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn push_front(&mut self, selection: Selection){
        self.selections.insert(0, selection);
        self.primary_selection_index = 0;
    }
    
    /// Appends a [Selection] to the back of [Self], and assigns its index to self.primary_selection_index
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![Selection::new(0, 0)], 0, &text); //[]idk\nsome\nshit\n
    /// selections.push(Selection::new(4, 4));   //[]idk\n[]some\nshit\n
    /// let expected_selections = Selections::new(vec![Selection::new(0, 0), Selection::new(4, 4)], 1, &text);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn push(&mut self, selection: Selection){
        self.selections.push(selection);
        self.primary_selection_index = self.selections.len().saturating_sub(1);
    }
    
    /// Returns the [Selection] at primary_selection_index as a reference
    pub fn primary(&self) -> &Selection{
        &self.selections[self.primary_selection_index]
    }
    pub fn first(&self) -> &Selection{
        // unwrapping because we ensure at least one selection is always present
        self.selections.first().unwrap()
    }
    pub fn first_mut(&mut self) -> &mut Selection{
        self.selections.first_mut().unwrap()
    }
    pub fn last(&self) -> &Selection{
        // unwrapping because we ensure at least one selection is always present
        self.selections.last().unwrap()
    }

    /// Sorts each [Selection] in [Selections] by position.
    /// #### Invariants:
    /// - preserves primary selection through the sorting process
    /// 
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(2, 4),
    ///     Selection::new(0, 5),
    ///     Selection::new(3, 6)
    /// ], 0, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::new(0, 5),
    ///     Selection::new(2, 4),
    ///     Selection::new(3, 6)
    /// ], 1, &text);
    /// selections.sort();
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn sort(&mut self){
        if self.selections.len() < 2{
            return;
        }

        let primary = self.selections[self.primary_selection_index].clone();
        self.selections.sort_unstable_by_key(Selection::start);
        self.primary_selection_index = self
            .selections
            .iter()
            .position(|selection| selection.clone() == primary)
            .unwrap();
    }

    /// Merges overlapping [Selection]s.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 2),    //[i d]k \n s o m e \n s h i t \n
    ///     Selection::new(1, 4),    // i[d k \n]s o m e \n s h i t \n
    ///     Selection::new(5, 7),    // i d k \n s[o m]e \n s h i t \n
    ///     Selection::new(8, 10),   // i d k \n s o m e[\n s]h i t \n
    ///     Selection::new(9, 12)    // i d k \n s o m e \n[s h i]t \n
    /// ], 4, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::with_stored_line_position(0, 4, 0),    //[i d k \n]s o m e \n s h i t \n
    ///     Selection::new(5, 7),    // i d k \n s[o m]e \n s h i t \n
    ///     Selection::with_stored_line_position(8, 12, 3)    // i d k \n s o m e[\n s h i]t \n
    /// ], 2, &text);
    /// selections.merge_overlapping(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn merge_overlapping(&mut self, text: &Rope){
        if self.selections.len() < 2{
            return;
        }

        let mut primary = self.selections[self.primary_selection_index].clone();
        self.selections.dedup_by(|current_selection, prev_selection| {
            if prev_selection.overlaps(current_selection.clone()){
                let new_selection = current_selection.merge(prev_selection, text);
                if prev_selection == &primary || current_selection == &primary{
                    primary = new_selection.clone();
                }
                *prev_selection = new_selection;
                true
            }else{
                false
            }
        });

        self.primary_selection_index = self
            .selections
            .iter()
            .position(|selection| selection.clone() == primary)
            .unwrap();
    }

    /// Removes all selections except Selection at primary_selection_index
    /// #### Invariants:
    /// - selections holds single selection
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![Selection::new(0, 0), Selection::new(4, 4)], 1, &text);
    /// selections.clear_non_primary_selections();
    /// assert!(selections == Selections::new(vec![Selection::new(4, 4)], 0, &text));
    /// ```
    pub fn clear_non_primary_selections(&mut self){
        self.selections = vec![self.selections[self.primary_selection_index].clone()];
        self.primary_selection_index = 0;
    }

    //TODO: return head and anchor positions
    //TODO: return Vec<Position> document cursor positions
    pub fn cursor_positions(&self, text: &Rope) -> Position{
        let cursor = self.last();
        let document_cursor = cursor.selection_to_selection2d(text);
        
        Position::new(
            document_cursor.head().x().saturating_add(1), 
            document_cursor.head().y().saturating_add(1)
        )
    }
}

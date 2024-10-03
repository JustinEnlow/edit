use ropey::Rope;
use crate::{
    Position,
    view::View,
    text_util
};



// if a block cursor is treated as a bar cursor with its head extended 1 grapheme to the right,
// i think these can be treated equally for cursor movements.(maybe excluding end of file?)
// however, they may need to behave differently for selection extension
#[derive(Clone, Copy, Debug)]
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
    pub fn new(anchor: usize, head: usize) -> Self{ // could init with cursor semantics: (anchor: usize, cursor: usize, semantics: CursorSemantics)
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
    /// assert!(test(Selection::new(1, 1), Direction::Backward, CursorSemantics::Block));   //but this state could be possible maybe?
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
    /// fn test(mut selection: Selection, expected: Selection, direction: Direction, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.set_direction(direction, &text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 5), Selection::with_stored_line_position(5, 0, 0), Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(5, 0), Selection::with_stored_line_position(0, 5, 1), Direction::Forward, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(1, 0, 0), Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 0), Selection::with_stored_line_position(0, 1, 0), Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 5), Selection::with_stored_line_position(5, 0, 0), Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(5, 0), Selection::with_stored_line_position(0, 5, 0), Direction::Forward, CursorSemantics::Block));
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
    /// assert!(test(Selection::new(0, 0), 0, CursorSemantics::Bar));    //[]idk\nsome\nshit\n      //|>idk\nsome\nshit\n
    /// assert!(test(Selection::new(1, 2), 1, CursorSemantics::Block));    //i[:d]k\nsome\nshit\n   //i|:d>k\nsome\nshit\n
    /// assert!(test(Selection::new(2, 1), 1, CursorSemantics::Block));    //i:]d[k\nsome\nshit\n   //i:<d|k\nsome\nshit\n
    /// assert!(test(Selection::new(2, 2), 1, CursorSemantics::Block));    //i:d][k\nsome\nshit\n   //i:d<|k\nsome\nshit\n  //though this state should be impossible with block cursor semantics
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
    /// fn test(mut selection: Selection, expected: Selection, to: usize, movement: Movement, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.put_cursor(to, &text, movement, semantics, true);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(5, 5, 1), 5, Movement::Move, CursorSemantics::Bar));
    /// assert!(test(Selection::new(5, 5), Selection::with_stored_line_position(0, 0, 0), 0, Movement::Move, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 5, 1), 5, Movement::Extend, CursorSemantics::Bar));
    /// assert!(test(Selection::new(5, 5), Selection::with_stored_line_position(5, 0, 0), 0, Movement::Extend, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(5, 6, 1), 5, Movement::Move, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 0), Selection::with_stored_line_position(5, 6, 1), 5, Movement::Move, CursorSemantics::Block));
    /// assert!(test(Selection::new(5, 6), Selection::with_stored_line_position(0, 1, 0), 0, Movement::Move, CursorSemantics::Block));
    /// assert!(test(Selection::new(6, 5), Selection::with_stored_line_position(0, 1, 0), 0, Movement::Move, CursorSemantics::Block));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 6, 1), 5, Movement::Extend, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 0), Selection::with_stored_line_position(0, 6, 1), 5, Movement::Extend, CursorSemantics::Block));
    /// assert!(test(Selection::new(5, 6), Selection::with_stored_line_position(6, 0, 0), 0, Movement::Extend, CursorSemantics::Block));
    /// assert!(test(Selection::new(6, 5), Selection::with_stored_line_position(6, 0, 0), 0, Movement::Extend, CursorSemantics::Block));
    /// 
    /// // test putting cursor at end of text
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(14, 14, 0), 14, Movement::Move, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 14, 0), 14, Movement::Extend, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(14, 15, 0), 14, Movement::Move, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 15, 0), 14, Movement::Extend, CursorSemantics::Block));
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
    /// fn test(mut selection: Selection, expected: Selection, amount: usize, movement: Movement, direction: Direction, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse\n");
    ///     selection.move_vertically(amount, &text, movement, direction, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(4, 4, 0), 1, Movement::Move, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(0, 0, 0), 1, Movement::Move, Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 4, 0), 1, Movement::Extend, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(4, 0, 0), 1, Movement::Extend, Direction::Backward, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(4, 5, 0), 1, Movement::Move, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(4, 5), Selection::with_stored_line_position(0, 1, 0), 1, Movement::Move, Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 5, 0), 1, Movement::Extend, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(4, 5), Selection::with_stored_line_position(5, 0, 0), 1, Movement::Extend, Direction::Backward, CursorSemantics::Block));
    /// 
    /// // handles moving/extending to text bounds correctly
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(19, 19, 0), 19, Movement::Move, Direction::Forward, CursorSemantics::Bar));  //idk\nsomething\nelse\n[]
    /// assert!(test(Selection::new(19, 19), Selection::with_stored_line_position(0, 0, 0), 19, Movement::Move, Direction::Backward, CursorSemantics::Bar));    //[]idk\nsomething\nelse\n
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 19, 0), 19, Movement::Extend, Direction::Forward, CursorSemantics::Bar));  //idk\nsomething\nelse\n[]
    /// assert!(test(Selection::new(19, 19), Selection::with_stored_line_position(19, 0, 0), 19, Movement::Extend, Direction::Backward, CursorSemantics::Bar));    //[]idk\nsomething\nelse\n
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(19, 20, 0), 19, Movement::Move, Direction::Forward, CursorSemantics::Block));   //idk\nsomething\nelse\n|: >    //is this the desired functionality?...
    /// assert!(test(Selection::new(19, 20), Selection::with_stored_line_position(0, 1, 0), 19, Movement::Move, Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 20, 0), 19, Movement::Extend, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(19, 20), Selection::with_stored_line_position(19, 0, 0), 19, Movement::Extend, Direction::Backward, CursorSemantics::Block));
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
    /// 
    /// fn test(mut selection: Selection, expected: Selection, amount: usize, movement: Movement, direction: Direction, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse\n");    //len 19
    ///     selection.move_horizontally(amount, &text, movement, direction, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(1, 1, 1), 1, Movement::Move, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 1), Selection::with_stored_line_position(0, 0, 0), 1, Movement::Move, Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 1, 1), 1, Movement::Extend, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 1), Selection::with_stored_line_position(1, 0, 0), 1, Movement::Extend, Direction::Backward, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(1, 2, 1), 1, Movement::Move, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 2), Selection::with_stored_line_position(0, 1, 0), 1, Movement::Move, Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 2, 1), 1, Movement::Extend, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(1, 2), Selection::with_stored_line_position(2, 0, 0), 1, Movement::Extend, Direction::Backward, CursorSemantics::Block));
    /// 
    /// // handles moving/extending to text bounds correctly
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(19, 19, 0), 19, Movement::Move, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(19, 19), Selection::with_stored_line_position(0, 0, 0), 19, Movement::Move, Direction::Backward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 19, 0), 19, Movement::Extend, Direction::Forward, CursorSemantics::Bar));
    /// assert!(test(Selection::new(19, 19), Selection::with_stored_line_position(19, 0, 0), 19, Movement::Extend, Direction::Backward, CursorSemantics::Bar));
    /// 
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(19, 20, 0), 19, Movement::Move, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(19, 20), Selection::with_stored_line_position(0, 1, 0), 19, Movement::Move, Direction::Backward, CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 20, 0), 19, Movement::Extend, Direction::Forward, CursorSemantics::Block));
    /// assert!(test(Selection::new(19, 20), Selection::with_stored_line_position(19, 0, 0), 19, Movement::Extend, Direction::Backward, CursorSemantics::Block));   //:<idk\nsomething\nelse\n|
    /// ```
    pub fn move_horizontally(&mut self, amount: usize, text: &Rope, movement: Movement, direction: Direction, semantics: CursorSemantics){
        let new_position = match direction{
            Direction::Forward => self.cursor(semantics).saturating_add(amount).min(text.len_chars()),    //ensures this does not move past text end
            Direction::Backward => self.cursor(semantics).saturating_sub(amount)
        };
        self.put_cursor(new_position, text, movement, semantics, true);
    }

     /// Moves cursor to specified line number.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Movement};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, line_number: usize, movement: Movement) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     selection.set_from_line_number(line_number, &text, movement);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// 
    /// // normal use
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(14, 14, 0), 2, Movement::Move));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 14, 0), 2, Movement::Extend));
    /// 
    /// // when line num > doc length, selection set to end of text
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(14, 14, 0), 5, Movement::Move));
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 14, 0), 5, Movement::Extend));
    /// 
    /// // restricts cursor to line end when stored_line_position > line width
    /// assert!(test(Selection::new(13, 13), Selection::with_stored_line_position(3, 3, 9), 0, Movement::Move));
    /// assert!(test(Selection::new(13, 13), Selection::with_stored_line_position(13, 3, 9), 0, Movement::Extend));
    /// ```
    // TODO: handle block cursor semantics?
    pub fn set_from_line_number(&mut self, line_number: usize, text: &Rope, movement: Movement){
        let line_number = line_number.min(text.len_lines().saturating_sub(1));  //restrict line_number to doc length(-1 because len_lines is 1 based)
        let current_line = text.char_to_line(self.cursor(CursorSemantics::Bar));
        
        let (amount, direction) = if line_number < current_line{
            (current_line.saturating_sub(line_number), Direction::Backward)
        }else{
            (line_number.saturating_sub(current_line), Direction::Forward)
        };
    
        self.move_vertically(amount, &text, movement, direction, CursorSemantics::Bar);
    }

    /// Aligns [Selection] anchor with head.
    /// ``` 
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.collapse(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // head < anchor
    /// assert!(test(Selection::new(4, 0), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));   //<idk\n|some\nshit\n   //<|idk\nsome\nshit\n
    /// assert!(test(Selection::new(4, 0), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block)); //:<idk\n|some\nshit\n  //|:i>dk\nsome\nshit\n
    /// 
    /// // anchor < head
    /// assert!(test(Selection::new(0, 4), Selection::with_stored_line_position(4, 4, 0), CursorSemantics::Bar));   //|idk\n>some\nshit\n   //idk\n|>some\nshit\n
    /// assert!(test(Selection::new(0, 4), Selection::with_stored_line_position(3, 4, 3), CursorSemantics::Block)); //|idk\n>some\nshit\n   //idk|:\n>some\nshit\n
    /// 
    /// // test setting cursor to end of text
    /// assert!(test(Selection::new(0, 14), Selection::with_stored_line_position(14, 14, 0), CursorSemantics::Bar));    //|idk\nsome\nshit\n>   //idk\nsome\nshit\n|>
    /// assert!(test(Selection::new(0, 14), Selection::with_stored_line_position(13, 14, 4), CursorSemantics::Block));  //|idk\nsome\nshit:\n>  //idk\nsome\nshit|:\n>
    /// ```
    pub fn collapse(&mut self, text: &Rope, semantics: CursorSemantics){
        self.put_cursor(self.cursor(semantics), text, Movement::Move, semantics, true);
    }

    /// Moves cursor right.
    /// ``` 
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.move_right(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(14, 14), Selection::with_stored_line_position(14, 14, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(14, 15), Selection::with_stored_line_position(14, 15, 0), CursorSemantics::Block));
    /// 
    /// // normal use
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(1, 1, 1), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(1, 2, 1), CursorSemantics::Block));
    /// 
    /// // new line resets stored line position
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(4, 4, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 4), Selection::with_stored_line_position(4, 5, 0), CursorSemantics::Block));
    /// 
    /// // with selection extended, collapses selection, then performs move
    /// assert!(test(Selection::new(0, 3), Selection::with_stored_line_position(4, 4, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 0), Selection::with_stored_line_position(1, 1, 1), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 3), Selection::with_stored_line_position(3, 4, 3), CursorSemantics::Block));
    /// assert!(test(Selection::new(3, 0), Selection::with_stored_line_position(1, 2, 1), CursorSemantics::Block));
    /// ```
    pub fn move_right(&mut self, text: &Rope, semantics: CursorSemantics){
        self.move_horizontally(1, text, Movement::Move, Direction::Forward, semantics);
    }

    /// Moves cursor left.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     selection.move_left(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// 
    /// // normal use
    /// assert!(test(Selection::new(1, 1), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 2), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// 
    /// // move to previous line resets stored line position
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(3, 3, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 5), Selection::with_stored_line_position(3, 4, 3), CursorSemantics::Block));
    /// 
    /// // with selection extended, collapses selection, then performs move
    /// assert!(test(Selection::new(1, 4), Selection::with_stored_line_position(3, 3, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 1), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 4), Selection::with_stored_line_position(2, 3, 2), CursorSemantics::Block));  // i[d k:\n]s o m e t h i n g \n e l s e
    ///                                                                                                              // i d[k]\n s o m e t h i n g \n e l s e
    /// assert!(test(Selection::new(4, 1), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));  // i]d k \n[s o m e t h i n g \n e l s e
    ///                                                                                                              //[i]d k \n s o m e t h i n g \n e l s e
    /// ```
    pub fn move_left(&mut self, text: &Rope, semantics: CursorSemantics){
        self.move_horizontally(1, text, Movement::Move, Direction::Backward, semantics);
    }

    /// Moves cursor up.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     selection.move_up(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// 
    /// // to shorter line
    /// assert!(test(Selection::new(13, 13), Selection::with_stored_line_position(3, 3, 9), CursorSemantics::Bar));
    /// assert!(test(Selection::new(13, 14), Selection::with_stored_line_position(3, 4, 9), CursorSemantics::Block));
    /// 
    /// // to longer line
    /// assert!(test(Selection::new(18, 18), Selection::with_stored_line_position(8, 8, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(18, 19), Selection::with_stored_line_position(8, 9, 4), CursorSemantics::Block));
    /// 
    /// // with selection extended, collapses selection, then performs move
    /// assert!(test(Selection::new(4, 14), Selection::with_stored_line_position(4, 4, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(14, 4), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 14), Selection::with_stored_line_position(3, 4, 9), CursorSemantics::Block));
    /// assert!(test(Selection::new(14, 4), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// ```
    pub fn move_up(&mut self, text: &Rope, semantics: CursorSemantics){
        self.move_vertically(1, text, Movement::Move, Direction::Backward, semantics);
    }

    /// Moves cursor down.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     selection.move_down(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(18, 18), Selection::with_stored_line_position(18, 18, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(18, 19), Selection::with_stored_line_position(18, 19, 4), CursorSemantics::Block));
    /// 
    /// // to longer line
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(7, 7, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 4), Selection::with_stored_line_position(7, 8, 3), CursorSemantics::Block));
    /// 
    /// // to shorter line
    /// assert!(test(Selection::new(13, 13), Selection::with_stored_line_position(18, 18, 9), CursorSemantics::Bar));
    /// assert!(test(Selection::new(13, 14), Selection::with_stored_line_position(18, 19, 9), CursorSemantics::Block));
    /// 
    /// // with selection extended, collapses selection, then performs move
    /// assert!(test(Selection::new(0, 4), Selection::with_stored_line_position(14, 14, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 0), Selection::with_stored_line_position(4, 4, 0), CursorSemantics::Bar));
    /// //[i d k \n]s o m e \n s h i t \n
    /// // i d k \n s o m[e]\n s h i t \n
    /// assert!(test(Selection::new(0, 4), Selection::with_stored_line_position(7, 8, 3), CursorSemantics::Block));
    /// assert!(test(Selection::new(4, 0), Selection::with_stored_line_position(4, 5, 0), CursorSemantics::Block));
    /// ```
    // //TODO: figure out single string for tests
    pub fn move_down(&mut self, text: &Rope, semantics: CursorSemantics){
        self.move_vertically(1, text, Movement::Move, Direction::Forward, semantics);
    }

    /// Moves cursor to line end.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse\n");
    ///     selection.move_line_text_end(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(3, 3, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(3, 4, 3), CursorSemantics::Block));
    /// assert!(test(Selection::new(3, 4), Selection::with_stored_line_position(3, 4, 3), CursorSemantics::Block)); //verify repeated calls result in appropriate behavior
    /// 
    /// // with selection extended, collapse and move
    /// assert!(test(Selection::new(0, 2), Selection::with_stored_line_position(3, 3, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(2, 0), Selection::with_stored_line_position(3, 3, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 2), Selection::with_stored_line_position(3, 4, 3), CursorSemantics::Block));
    /// assert!(test(Selection::new(2, 0), Selection::with_stored_line_position(3, 4, 3), CursorSemantics::Block));
    /// ```
    pub fn move_line_text_end(&mut self, text: &Rope, semantics: CursorSemantics){
        let line_number = text.char_to_line(self.cursor(semantics));
        let line = text.line(line_number);
        let line_width = text_util::line_width_excluding_newline(line);
        let line_start = text.line_to_char(line_number);
        let line_end = line_start.saturating_add(line_width);

        self.put_cursor(line_end, text, Movement::Move, semantics, true);
    }

    /// Moves cursor to absolute start of line, or start of line text, depending on cursor position.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("    idk\n");
    ///     selection.move_home(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // moves to text start when cursor past text start
    /// assert!(test(Selection::new(6, 6), Selection::with_stored_line_position(4, 4, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(6, 7), Selection::with_stored_line_position(4, 5, 4), CursorSemantics::Block));
    /// 
    /// // moves to line start when cursor at text start
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 5), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// 
    /// // moves to text start when cursor before text start
    /// assert!(test(Selection::new(1, 1), Selection::with_stored_line_position(4, 4, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 2), Selection::with_stored_line_position(4, 5, 4), CursorSemantics::Block));
    /// 
    /// // with selection extended, collapse and move
    /// assert!(test(Selection::new(0, 5), Selection::with_stored_line_position(4, 4, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 3), Selection::with_stored_line_position(4, 4, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 4), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(5, 0), Selection::with_stored_line_position(4, 4, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 6), Selection::with_stored_line_position(4, 5, 4), CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 4), Selection::with_stored_line_position(4, 5, 4), CursorSemantics::Block));
    /// assert!(test(Selection::new(0, 5), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// assert!(test(Selection::new(5, 0), Selection::with_stored_line_position(4, 5, 4), CursorSemantics::Block));
    /// ```
    pub fn move_home(&mut self, text: &Rope, semantics: CursorSemantics){
        let line_number = text.char_to_line(self.cursor(semantics));
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        if self.cursor(semantics) == text_start{
            self.move_line_start(text, semantics);
        }else{
            self.move_line_text_start(text, semantics);
        }
    }
    
    /// Moves to line start.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.move_line_start(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 4), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// ```
    pub fn move_line_start(&mut self, text: &Rope, semantics: CursorSemantics){
        let line_number = text.char_to_line(self.cursor(semantics));
        let line_start = text.line_to_char(line_number);

        self.put_cursor(line_start, text, Movement::Move, semantics, true);
    }
    
    /// Moves to start of text on line.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// let text = Rope::from("  idk\n");
    /// let mut selection = Selection::new(0, 0);
    /// let expected_selection = Selection::with_stored_line_position(2, 2, 2);
    /// selection.move_line_text_start(&text, CursorSemantics::Bar);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_line_text_start(&mut self, text: &Rope, semantics: CursorSemantics){
        let line_number = text.char_to_line(self.cursor(semantics));
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        self.put_cursor(text_start, text, Movement::Move, semantics, true);
    }

    /// Moves cursor up by the height of client view.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// # use edit::view::View;
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// let client_view = View::new(0, 0, 2, 2);
    /// let mut selection = Selection::new(6, 6);                        //idk\nso[]mething\nelse
    /// let expected_selection = Selection::with_stored_line_position(2, 2, 2); //id[]k\nsomething\nelse
    /// selection.move_page_up(&text, &client_view, CursorSemantics::Bar);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_page_up(&mut self, text: &Rope, client_view: &View, semantics: CursorSemantics){
        self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Move, Direction::Backward, semantics);
    }

    /// Moves cursor down by the height of client view.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// # use edit::view::View;
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// let client_view = View::new(0, 0, 2, 2);
    /// let mut selection = Selection::new(0, 0);                               //[]idk\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 0); //idk\n[]something\nelse
    /// selection.move_page_down(&text, &client_view, CursorSemantics::Bar);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_page_down(&mut self, text: &Rope, client_view: &View, semantics: CursorSemantics){
        self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Move, Direction::Forward, semantics);
    }

    /// Moves cursor to the start of the document.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// let text = Rope::from("idk\n");
    /// let mut selection = Selection::new(12, 12);
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);
    /// selection.move_doc_start(&text, CursorSemantics::Bar);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_doc_start(&mut self, text: &Rope, semantics: CursorSemantics){
        self.put_cursor(0, text, Movement::Move, semantics, true);
    }

    /// Moves cursor to the end of the document.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// let text = Rope::from("idk\nsome\nshit");
    /// let mut selection = Selection::new(0, 0);                                   //[]idk\nsome\nshit
    /// let expected_selection = Selection::with_stored_line_position(13, 13, 4);   //idk\nsome\nshit[]
    /// selection.move_doc_end(&text, CursorSemantics::Bar);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_doc_end(&mut self, text: &Rope, semantics: CursorSemantics){
        self.put_cursor(text.len_chars(), text, Movement::Move, semantics, true);
    }

    /// Extends selection to the right.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.extend_right(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // stays within bounds
    /// assert!(test(Selection::new(14, 14), Selection::with_stored_line_position(14, 14, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(14, 15), Selection::with_stored_line_position(14, 15, 0), CursorSemantics::Block));
    /// 
    /// // normal use
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 1, 1), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 2, 1), CursorSemantics::Block));
    /// 
    /// // resets stored line position after new line
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(3, 4, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 4), Selection::with_stored_line_position(3, 5, 0), CursorSemantics::Block));
    /// 
    /// // previously extended
    /// assert!(test(Selection::new(0, 3), Selection::with_stored_line_position(0, 4, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 0), Selection::with_stored_line_position(3, 1, 1), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 3), Selection::with_stored_line_position(0, 4, 3), CursorSemantics::Block));
    /// assert!(test(Selection::new(3, 0), Selection::with_stored_line_position(3, 1, 1), CursorSemantics::Block));
    /// ```
    pub fn extend_right(&mut self, text: &Rope, semantics: CursorSemantics){
        self.move_horizontally(1, text, Movement::Extend, Direction::Forward, semantics);
    }

    /// Extends selection to the left.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     selection.extend_left(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// 
    /// // normal use
    /// assert!(test(Selection::new(2, 2), Selection::with_stored_line_position(2, 1, 1), CursorSemantics::Bar));
    /// assert!(test(Selection::new(2, 3), Selection::with_stored_line_position(3, 1, 1), CursorSemantics::Block)); //id[:k]\nsomething\nelse   //i:]dk[\nsomething\nelse
    /// 
    /// //updates stored line position on line change
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(4, 3, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 5), Selection::with_stored_line_position(5, 3, 3), CursorSemantics::Block)); //idk\n[s]omething\nelse    //idk:]\ns[omething\nelse
    /// 
    /// //previously extended
    /// assert!(test(Selection::new(0, 3), Selection::with_stored_line_position(0, 2, 2), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 1), Selection::with_stored_line_position(3, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 3), Selection::with_stored_line_position(0, 2, 1), CursorSemantics::Block)); //[id:k]\nsomething\nelse   //[i:d]k\nsomething\nelse
    /// assert!(test(Selection::new(3, 1), Selection::with_stored_line_position(3, 0, 0), CursorSemantics::Block)); //i:]dk[\nsomething\nelse   //:]idk[\nsomething\nelse
    /// ```
    pub fn extend_left(&mut self, text: &Rope, semantics: CursorSemantics){
        self.move_horizontally(1, text, Movement::Extend, Direction::Backward, semantics);
    }

    /// Extends selection up.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     selection.extend_up(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 1, 0), CursorSemantics::Block));
    /// 
    /// // to shorter line
    /// assert!(test(Selection::new(13, 13), Selection::with_stored_line_position(13, 3, 9), CursorSemantics::Bar));
    /// assert!(test(Selection::new(13, 14), Selection::with_stored_line_position(14, 3, 9), CursorSemantics::Block));  //idk\nsomething[:\n]else    //idk:]\nsomething\n[else
    /// 
    /// // to longer line
    /// assert!(test(Selection::new(18, 18), Selection::with_stored_line_position(18, 8, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(18, 19), Selection::with_stored_line_position(18, 8, 4), CursorSemantics::Block));  //idk\nsomething\nelse[: ]   //idk\nsome:]thing\nelse[
    /// ```
    pub fn extend_up(&mut self, text: &Rope, semantics: CursorSemantics){
        self.move_vertically(1, text, Movement::Extend, Direction::Backward, semantics);
    }

    /// Extends selection down.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     selection.extend_down(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // stays within doc bounds
    /// assert!(test(Selection::new(18, 18), Selection::with_stored_line_position(18, 18, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(18, 19), Selection::with_stored_line_position(18, 19, 4), CursorSemantics::Block));
    /// 
    /// // to shorter line
    /// assert!(test(Selection::new(13, 13), Selection::with_stored_line_position(13, 18, 9), CursorSemantics::Bar));
    /// assert!(test(Selection::new(13, 14), Selection::with_stored_line_position(13, 19, 9), CursorSemantics::Block)); //idk\nsomething[:\n]else    //idk\nsomething[\nelse: ]   //does anything need to be done about the overextension here?
    /// 
    /// // to longer line
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(3, 7, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 4), Selection::with_stored_line_position(3, 8, 3), CursorSemantics::Block)); //idk[:\n]something\nelse    //idk[\nsom:e]thing\nelse
    /// ```
    pub fn extend_down(&mut self, text: &Rope, semantics: CursorSemantics){
        self.move_vertically(1, text, Movement::Extend, Direction::Forward, semantics);
    }

    /// Extend selection to end of line text.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\n");
    ///     selection.extend_line_text_end(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 3, 3), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 3, 2), CursorSemantics::Block));
    /// ```
    pub fn extend_line_text_end(&mut self, text: &Rope, semantics: CursorSemantics){
        let line_number = text.char_to_line(self.head);
        let line = text.line(line_number);
        let line_width = text_util::line_width_excluding_newline(line);
        let line_start = text.line_to_char(line_number);
        let line_end = /*line_start.saturating_add(line_width);*/ match semantics{
            CursorSemantics::Bar => line_start.saturating_add(line_width),
            CursorSemantics::Block => line_start.saturating_add(line_width).saturating_sub(1)
        };

        self.put_cursor(line_end, text, Movement::Extend, semantics, true);  //line_end.saturating_sub(1) for block cursor
    }

    /// Extends [Selection] to absolute start of line, or line text start, depending on [Selection] head position.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("    idk\n");
    ///     selection.extend_home(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// // extends selection to text start when head past text start
    /// assert!(test(Selection::new(6, 6), Selection::with_stored_line_position(6, 4, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(6, 7), Selection::with_stored_line_position(7, 4, 4), CursorSemantics::Block));
    /// 
    /// // extends selection to line start when head at text start
    /// assert!(test(Selection::new(4, 4), Selection::with_stored_line_position(4, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(4, 5), Selection::with_stored_line_position(5, 0, 0), CursorSemantics::Block)); //    [:i]dk\n  //:]    i[dk\n
    /// 
    /// // extends selection to text start when head before text start
    /// assert!(test(Selection::new(1, 1), Selection::with_stored_line_position(1, 4, 4), CursorSemantics::Bar));
    /// assert!(test(Selection::new(1, 2), Selection::with_stored_line_position(1, 5, 4), CursorSemantics::Block)); // [: ]  idk\n  // [   :i]dk\n
    /// ```
    pub fn extend_home(&mut self, text: &Rope, semantics: CursorSemantics){
        let line_number = text.char_to_line(self.cursor(semantics));
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        if self.cursor(semantics) == text_start{
            self.extend_line_start(text, semantics);
        }else{
            self.extend_line_text_start(text, semantics);
        }
    }
    
    /// Extends [Selection] to start of line.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.extend_line_start(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(3, 3), Selection::with_stored_line_position(3, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(3, 4), Selection::with_stored_line_position(4, 0, 0), CursorSemantics::Block)); //idk[\n]some\nshit\n   //:]idk\n[some\nshit\n
    /// ```
    pub fn extend_line_start(&mut self, text: &Rope, semantics: CursorSemantics){
        let line_number = text.char_to_line(self.cursor(semantics));
        let line_start = text.line_to_char(line_number);

        self.put_cursor(line_start, text, Movement::Extend, semantics, true);
    }
    
    /// Extends [Selection] to start of text in line.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("  idk\n");
    ///     selection.extend_line_text_start(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 2, 2), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 3, 2), CursorSemantics::Block));
    /// ```
    pub fn extend_line_text_start(&mut self, text: &Rope, semantics: CursorSemantics){
        let line_number = text.char_to_line(self.cursor(semantics));
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        self.put_cursor(text_start, text, Movement::Extend, semantics, true);
    }
    
    /// Extends [Selection] up by the height of client view.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// # use edit::view::View;
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     let client_view = View::new(0, 0, 2, 2);
    ///     selection.extend_page_up(&text, &client_view, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(6, 6), Selection::with_stored_line_position(6, 2, 2), CursorSemantics::Bar));
    /// assert!(test(Selection::new(6, 7), Selection::with_stored_line_position(7, 2, 2), CursorSemantics::Block)); //idk\nso[m]ething\nelse    //id:]k\nsom[ething\nelse
    /// ```
    pub fn extend_page_up(&mut self, text: &Rope, client_view: &View, semantics: CursorSemantics){
        self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Extend, Direction::Backward, semantics);
    }
    
    /// Extends [Selection] down by the height of client view.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// # use edit::view::View;
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsomething\nelse");
    ///     let client_view = View::new(0, 0, 2, 2);
    ///     selection.extend_page_down(&text, &client_view, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 4, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 5, 0), CursorSemantics::Block)); //[i]dk\nsomething\nelse    //[idk\n:s]omething\nelse
    /// ```
    pub fn extend_page_down(&mut self, text: &Rope, client_view: &View, semantics: CursorSemantics){
        self.move_vertically(client_view.height().saturating_sub(1), text, Movement::Extend, Direction::Forward, semantics);
    }
    
    /// Extends [Selection] to doc start.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.extend_doc_start(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(9, 9), Selection::with_stored_line_position(9, 0, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(9, 10), Selection::with_stored_line_position(10, 0, 0), CursorSemantics::Block));   //idk\nsome\n[s]hit\n   //:]idk\nsome\ns[hit\n
    /// ```
    pub fn extend_doc_start(&mut self, text: &Rope, semantics: CursorSemantics){
        self.put_cursor(0, text, Movement::Extend, semantics, true);
    }
    
    /// Extends [Selection] to doc end.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.extend_doc_end(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 14, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 15, 0), CursorSemantics::Block));
    /// ```
    pub fn extend_doc_end(&mut self, text: &Rope, semantics: CursorSemantics){
        self.put_cursor(text.len_chars(), text, Movement::Extend, semantics, true);
    }

    /// Selects all text.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// fn test(mut selection: Selection, expected: Selection, semantics: CursorSemantics) -> bool{
    ///     let text = Rope::from("idk\nsome\nshit\n");
    ///     selection.select_all(&text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, selection);
    ///     selection == expected
    /// }
    /// 
    /// assert!(test(Selection::new(0, 0), Selection::with_stored_line_position(0, 14, 0), CursorSemantics::Bar));
    /// assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 15, 0), CursorSemantics::Block));
    /// //assert!(test(Selection::new(0, 1), Selection::with_stored_line_position(0, 14, 4), CursorSemantics::Block));
    /// ```
    pub fn select_all(&mut self, text: &Rope, semantics: CursorSemantics){
        self.put_cursor(0, &text, Movement::Move, semantics, true);
        self.put_cursor(text.len_chars(), &text, Movement::Extend, semantics, true);
        //match semantics{    //needed to handle overextension of block cursor at end of text
        //    CursorSemantics::Bar => self.put_cursor(text.len_chars(), &text, Movement::Extend, semantics, true),
        //    CursorSemantics::Block => self.put_cursor(text.len_chars().saturating_sub(1), &text, Movement::Extend, semantics, true),
        //}
    }

    /// Translates a [Selection] to a [Selection2d]
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selection2d, CursorSemantics};
    /// # use edit::Position;
    /// 
    /// fn test(selection: Selection, expected: Selection2d, text: &Rope, semantics: CursorSemantics) -> bool{
    ///     let result = selection.selection_to_selection2d(text, semantics);
    ///     println!("expected: {:#?}\ngot: {:#?}\n", expected, result);
    ///     result == expected
    /// }
    /// 
    /// let text = Rope::from("idk\nsomething");
    /// 
    /// // when selection head/anchor same, and on same line
    /// //id[]k
    /// //something
    /// assert!(test(Selection::new(2, 2), Selection2d::new(Position::new(2, 0), Position::new(2, 0)), &text, CursorSemantics::Bar)); //id[]k\nsomething
    /// assert!(test(Selection::new(2, 3), Selection2d::new(Position::new(2, 0), Position::new(2, 0)), &text, CursorSemantics::Block));
    /// 
    /// // when selection head/anchor different, but on same line
    /// //i[d]k
    /// //something
    /// assert!(test(Selection::new(1, 2), Selection2d::new(Position::new(2, 0), Position::new(1, 0)), &text, CursorSemantics::Bar)); //i[d]k\nsomething
    /// assert!(test(Selection::new(1, 3), Selection2d::new(Position::new(2, 0), Position::new(1, 0)), &text, CursorSemantics::Block));
    /// 
    /// // when selection head/anchor same, but on new line
    /// //idk
    /// //[]something
    /// assert!(test(Selection::new(4, 4), Selection2d::new(Position::new(0, 1), Position::new(0, 1)), &text, CursorSemantics::Bar)); //idk\n[]something
    /// assert!(test(Selection::new(4, 5), Selection2d::new(Position::new(0, 1), Position::new(0, 1)), &text, CursorSemantics::Block));
    /// 
    /// // when selection head/anchor different, and on different lines
    /// //id[k
    /// //s]omething
    /// assert!(test(Selection::new(2, 5), Selection2d::new(Position::new(1, 1), Position::new(2, 0)), &text, CursorSemantics::Bar)); //id[k\ns]omething
    /// assert!(test(Selection::new(2, 6), Selection2d::new(Position::new(1, 1), Position::new(2, 0)), &text, CursorSemantics::Block));
    /// ```
    pub fn selection_to_selection2d(&self, text: &Rope, semantics: CursorSemantics) -> Selection2d{
        //let line_number_head = text.char_to_line(self.head);
        let line_number_head = text.char_to_line(self.cursor(semantics));
        let line_number_anchor = text.char_to_line(self.anchor);

        let head_line_start_idx = text.line_to_char(line_number_head);
        let anchor_line_start_idx = text.line_to_char(line_number_anchor);

        Selection2d::new(
            Position::new(
                //self.head - head_line_start_idx, 
                self.cursor(semantics).saturating_sub(head_line_start_idx),
                line_number_head
            ), 
            Position::new(
                //self.anchor - anchor_line_start_idx, 
                self.anchor.saturating_sub(anchor_line_start_idx),
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
    pub fn cursor_positions(&self, text: &Rope, semantics: CursorSemantics) -> Position{
        let cursor = self.last();
        let document_cursor = cursor.selection_to_selection2d(text, semantics);
        
        Position::new(
            document_cursor.head().x().saturating_add(1), 
            document_cursor.head().y().saturating_add(1)
        )
    }
}

use ropey::Rope;
use crate::Position;
use crate::view::View;
use crate::text_util;

/// User configuration for semantic meaning of cursor(should this be defined by front end instead?)
//const CURSOR_SEMANTICS: CursorSemantics = CursorSemantics::Bar;

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

#[derive(PartialEq)]
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
    pub fn point(head: usize) -> Self{
        Self::new(head, head)
    }
    pub fn anchor(&self) -> usize{
        self.anchor
    }
    pub fn head(&self) -> usize{
        self.head
    }

    /// Start of the [Selection] from left to right.
    pub fn start(&self) -> usize{
        std::cmp::min(self.anchor, self.head)
    }
    /// End of the [Selection] from left to right.
    pub fn end(&self) -> usize{
        std::cmp::max(self.anchor, self.head)
    }

    /// ```
    /// # use edit::selection::Selection;
    /// 
    /// assert!(Selection::new(0, 0).is_extended() == false);
    /// assert!(Selection::new(0, 1).is_extended() == true);
    /// ```
    //TODO: handle block cursor semantics
    pub fn is_extended(&self) -> bool{
        self.anchor != self.head
        //self.anchor != self.cursor(semantics)  //with cursor returning either self.head, or the grapheme immediately before self.head
    }

    /// returns the direction of [Selection]
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Direction};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// let selection = Selection::new(0, 5);
    /// assert!(selection.direction() == Direction::Forward);
    /// 
    /// let selection = Selection::new(5, 0);
    /// assert!(selection.direction() == Direction::Backward);
    /// ```
    pub fn direction(&self) -> Direction{
        if self.head < self.anchor{ //< self.cursor()?
            Direction::Backward
        }else{
            Direction::Forward
        }
    }

    /// Sets [Selection] direction to specified direction.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Direction};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// let mut selection = Selection::new(0, 5);
    /// let expected_selection = Selection::with_stored_line_position(5, 0, 0);
    /// selection.set_direction(Direction::Backward, &text);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// let mut selection = Selection::new(5, 0);
    /// let expected_selection = Selection::with_stored_line_position(0, 5, 1);
    /// selection.set_direction(Direction::Forward, &text);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn set_direction(&mut self, direction: Direction, text: &Rope){
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
        self.stored_line_position = Some(text_util::offset_from_line_start(self.head, text));
    }

    /// Checks self and other for overlap.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
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
    /// assert!(Selection::new(0, 3).overlaps(Selection::new(3, 6)) == false);   //[idk]<\nso>me\nshit\n
    /// assert!(Selection::new(0, 3).overlaps(Selection::new(6, 3)) == false);   //[idk]>\nso<me\nshit\n
    /// assert!(Selection::new(3, 0).overlaps(Selection::new(3, 6)) == false);   //]idk[<\nso>me\nshit\n
    /// assert!(Selection::new(3, 0).overlaps(Selection::new(6, 3)) == false);   //]idk[>\nso<me\nshit\n
    /// assert!(Selection::new(3, 6).overlaps(Selection::new(0, 3)) == false);   //<idk>[\nso]me\nshit\n
    /// assert!(Selection::new(3, 6).overlaps(Selection::new(3, 0)) == false);   //>idk<[\nso]me\nshit\n
    /// assert!(Selection::new(6, 3).overlaps(Selection::new(0, 3)) == false);   //<idk>]\nso[me\nshit\n
    /// assert!(Selection::new(6, 3).overlaps(Selection::new(3, 0)) == false);   //>idk<]\nso[me\nshit\n
    /// 
    /// // non-zero-width selections, overlap.
    /// assert!(Selection::new(0, 4).overlaps(Selection::new(3, 6)));   //[idk<\n]so>me\nshit\n
    /// assert!(Selection::new(0, 4).overlaps(Selection::new(6, 3)));   //[idk>\n]so<me\nshit\n
    /// assert!(Selection::new(4, 0).overlaps(Selection::new(3, 6)));   //]idk<\n[so>me\nshit\n
    /// assert!(Selection::new(4, 0).overlaps(Selection::new(6, 3)));   //]idk>\n[so<me\nshit\n
    /// assert!(Selection::new(3, 6).overlaps(Selection::new(0, 4)));   //<idk[\n>so]me\nshit\n
    /// assert!(Selection::new(3, 6).overlaps(Selection::new(4, 0)));   //>idk[\n<so]me\nshit\n
    /// assert!(Selection::new(6, 3).overlaps(Selection::new(0, 4)));   //<idk]\n>so[me\nshit\n
    /// assert!(Selection::new(6, 3).overlaps(Selection::new(4, 0)));   //>idk]\n<so[me\nshit\n
    /// 
    /// // Zero-width and non-zero-width selections, no overlap.
    /// assert!(Selection::new(0, 3).overlaps(Selection::new(3, 3)) == false);   //[idk]<>\nsome\nshit\n
    /// assert!(Selection::new(3, 0).overlaps(Selection::new(3, 3)) == false);   //]idk[<>\nsome\nshit\n
    /// assert!(Selection::new(3, 3).overlaps(Selection::new(0, 3)) == false);   //<idk>[]\nsome\nshit\n
    /// assert!(Selection::new(3, 3).overlaps(Selection::new(3, 0)) == false);   //>idk<[]\nsome\nshit\n
    /// 
    /// // Zero-width and non-zero-width selections, overlap.
    /// assert!(Selection::new(1, 4).overlaps(Selection::new(1, 1)));    //i[<>dk\n]some\nshit\n
    /// assert!(Selection::new(4, 1).overlaps(Selection::new(1, 1)));    //i]<>dk\n[some\nshit\n
    /// assert!(Selection::new(1, 1).overlaps(Selection::new(1, 4)));    //i[<]dk\n>some\nshit\n
    /// assert!(Selection::new(1, 1).overlaps(Selection::new(4, 1)));    //i[>]dk\n<some\nshit\n
    /// 
    /// assert!(Selection::new(1, 4).overlaps(Selection::new(3, 3)));    //i[dk<>\n]some\nshit\n
    /// assert!(Selection::new(4, 1).overlaps(Selection::new(3, 3)));    //i]dk<>\n[some\nshit\n
    /// assert!(Selection::new(3, 3).overlaps(Selection::new(1, 4)));    //i<dk[]\n>some\nshit\n
    /// assert!(Selection::new(3, 3).overlaps(Selection::new(4, 1)));    //i>dk[]\n<some\nshit\n
    /// 
    /// // zero-width selections, no overlap.
    /// assert!(Selection::new(0, 0).overlaps(Selection::new(1, 1)) == false);   //[]i<>dk\nsome\nshit\n
    /// assert!(Selection::new(1, 1).overlaps(Selection::new(0, 0)) == false);   //<>i[]dk\nsome\nshit\n
    /// 
    /// // zero-width selections, overlap.
    /// assert!(Selection::new(1, 1).overlaps(Selection::new(1, 1)));    //i[<>]dk\nsome\nshit\n
    /// ```
    pub fn overlaps(&self, other: Selection) -> bool{
        self.start() == other.start() || (self.end() > other.start() && other.end() > self.start())
    }

    //pub fn extend(&self, from: usize, to: usize) -> Self {
    //    debug_assert!(from <= to);
//
    //    if self.anchor <= self.head {
    //        Self{
    //            anchor: self.anchor.min(from),
    //            head: self.head.max(to),
    //            stored_line_position: None,
    //        }
    //    } else {
    //        Self {
    //            anchor: self.anchor.max(to),
    //            head: self.head.min(from),
    //            stored_line_position: None,
    //        }
    //    }
    //}

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
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // when self.anchor > self.head && other.anchor > other.head
    /// let selection1 = Selection::new(4, 0);
    /// let selection2 = Selection::new(5, 1);
    /// let expected_selection = Selection::with_stored_line_position(0, 5, 1);
    /// let selection = selection1.merge(&selection2, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// let selection = selection2.merge(&selection1, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// // when self.anchor < self.head && other.anchor < other.head
    /// let selection1 = Selection::new(0, 4);   //[i dk\n]s ome\nshit\n
    /// let selection2 = Selection::new(1, 5);   // i[dk\n s]ome\nshit\n
    /// let expected_selection = Selection::with_stored_line_position(0, 5, 1);
    /// let selection = selection1.merge(&selection2, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// let selection = selection2.merge(&selection1, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// // when self.anchor > self.head && other.anchor < other.head
    /// let selection1 = Selection::new(4, 0);   //]i dk\n[s ome\nshit\n
    /// let selection2 = Selection::new(1, 5);   // i[dk\n s]ome\nshit\n
    /// let expected_selection = Selection::with_stored_line_position(0, 5, 1);
    /// let selection = selection1.merge(&selection2, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// let selection = selection2.merge(&selection1, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// // when self.anchor < self.head && other.anchor > other.head
    /// let selection1 = Selection::new(0, 4);   //[i dk\n]s ome\nshit\n
    /// let selection2 = Selection::new(5, 1);   // i]dk\n s[ome\nshit\n
    /// let expected_selection = Selection::with_stored_line_position(0, 5, 1);
    /// let selection = selection1.merge(&selection2, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// let selection = selection2.merge(&selection1, &text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// // consecutive
    /// assert!(Selection::merge(&Selection::new(0, 1), &Selection::new(1, 2), &text) == Selection::with_stored_line_position(0, 2, 2));
    /// assert!(Selection::merge(&Selection::new(1, 0), &Selection::new(1, 2), &text) == Selection::with_stored_line_position(0, 2, 2));
    /// assert!(Selection::merge(&Selection::new(1, 0), &Selection::new(2, 1), &text) == Selection::with_stored_line_position(0, 2, 2));
    /// assert!(Selection::merge(&Selection::new(0, 1), &Selection::new(2, 1), &text) == Selection::with_stored_line_position(0, 2, 2));
    ///
    /// assert!(Selection::merge(&Selection::new(1, 2), &Selection::new(0, 1), &text) == Selection::with_stored_line_position(0, 2, 2));
    /// assert!(Selection::merge(&Selection::new(2, 1), &Selection::new(0, 1), &text) == Selection::with_stored_line_position(0, 2, 2));
    /// assert!(Selection::merge(&Selection::new(2, 1), &Selection::new(1, 0), &text) == Selection::with_stored_line_position(0, 2, 2));
    /// assert!(Selection::merge(&Selection::new(1, 2), &Selection::new(1, 0), &text) == Selection::with_stored_line_position(0, 2, 2));
    ///
    /// // overlapping
    /// assert!(Selection::merge(&Selection::new(0, 2), &Selection::new(1, 4), &text) == Selection::with_stored_line_position(0, 4, 0));
    /// assert!(Selection::merge(&Selection::new(2, 0), &Selection::new(1, 4), &text) == Selection::with_stored_line_position(0, 4, 0));
    /// assert!(Selection::merge(&Selection::new(2, 0), &Selection::new(4, 1), &text) == Selection::with_stored_line_position(0, 4, 0));
    /// assert!(Selection::merge(&Selection::new(0, 2), &Selection::new(4, 1), &text) == Selection::with_stored_line_position(0, 4, 0));
    /// 
    /// assert!(Selection::merge(&Selection::new(1, 4), &Selection::new(0, 2), &text) == Selection::with_stored_line_position(0, 4, 0));
    /// assert!(Selection::merge(&Selection::new(4, 1), &Selection::new(0, 2), &text) == Selection::with_stored_line_position(0, 4, 0));
    /// assert!(Selection::merge(&Selection::new(4, 1), &Selection::new(2, 0), &text) == Selection::with_stored_line_position(0, 4, 0));
    /// assert!(Selection::merge(&Selection::new(1, 4), &Selection::new(2, 0), &text) == Selection::with_stored_line_position(0, 4, 0));
    /// 
    /// // contained
    /// assert!(Selection::merge(&Selection::new(0, 6), &Selection::new(2, 4), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(6, 0), &Selection::new(2, 4), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(6, 0), &Selection::new(4, 2), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(0, 6), &Selection::new(4, 2), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// 
    /// assert!(Selection::merge(&Selection::new(2, 4), &Selection::new(0, 6), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(4, 2), &Selection::new(0, 6), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(4, 2), &Selection::new(6, 0), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(2, 4), &Selection::new(6, 0), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// 
    /// // disconnected
    /// assert!(Selection::merge(&Selection::new(0, 2), &Selection::new(4, 6), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(2, 0), &Selection::new(4, 6), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(2, 0), &Selection::new(6, 4), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(0, 2), &Selection::new(6, 4), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// 
    /// assert!(Selection::merge(&Selection::new(4, 6), &Selection::new(0, 2), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(6, 4), &Selection::new(0, 2), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(6, 4), &Selection::new(2, 0), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// assert!(Selection::merge(&Selection::new(4, 6), &Selection::new(2, 0), &text) == Selection::with_stored_line_position(0, 6, 2));
    /// ```
    pub fn merge(&self, other: &Selection, text: &Rope) -> Selection{
        let anchor = self.start().min(other.start());
        let head = self.end().max(other.end());
        let stored_line_position = text_util::offset_from_line_start(head, text);
            
        Selection{anchor, head, stored_line_position: Some(stored_line_position)}
    }

    /////////////////////////////////// Alignment Methods ///////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////////////

    /////////////////////////////////// Block Cursor Methods ///////////////////////////////////
    
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, CursorSemantics};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // key:
    /// // anchor             = [
    /// // head               = ]
    /// // block_virtual_head = :
    /// 
    /// let selection = Selection::new(1, 2);    //i[:d]k\nsome\nshit\n
    /// assert!(selection.cursor(CursorSemantics::Block) == 1);
    /// 
    /// let selection = Selection::new(2, 1);    //i:]d[k\nsome\nshit\n
    /// assert!(selection.cursor(CursorSemantics::Block) == 1);
    /// 
    /// let selection = Selection::new(2, 2);    //i:d][k\nsome\nshit\n
    /// assert!(selection.cursor(CursorSemantics::Block) == 1);
    /// 
    /// let selection = Selection::new(0, 0);    //[]idk\nsome\nshit\n
    /// assert!(selection.cursor(CursorSemantics::Bar) == 0);
    /// ```
    // not sure if this impl is going to stay this way...
    pub fn cursor(&self, semantics: CursorSemantics) -> usize{
        match semantics{
            CursorSemantics::Bar => self.head,
            CursorSemantics::Block => {
                if self.head >= self.anchor{
                    self.head - 1   //prev_grapheme_boundary(text, self.head)
                }else{
                    self.head
                }
            }
        }
    }

    // head('>') >= anchor('|') && char_idx(':') < anchor
    // i d k:\n|s o m e>\n s h i t \n
    // head < anchor && char_idx >= anchor
    // i d k \n<s o m e|\n s:h i t \n
    //pub fn put_cursor(self, text: &Rope, char_idx: usize, movement: Movement) -> Self{
    //    match movement{
    //        Movement::Extend => {
    //            let anchor = if self.head >= self.anchor && char_idx < self.anchor {
    //                next_grapheme_boundary(text, self.anchor)
    //            } else if self.head < self.anchor && char_idx >= self.anchor {
    //                prev_grapheme_boundary(text, self.anchor)
    //            } else {
    //                self.anchor
    //            };
    //
    //            if anchor <= char_idx {
    //                Self::new(anchor, next_grapheme_boundary(text, char_idx))
    //            } else {
    //                Self::new(anchor, char_idx)
    //            }
    //        }
    //        Movement::Move => {
    //            Self::Point(char_idx)
    //        }
    //    }
    //}

    /////////////////////////////////// Movement Methods ///////////////////////////////////
    //TODO: may be able to use stored_line_position as Option<usize> if fns that use stored_line_position calculate
    //it at the start of the fn, then assign a new one if changed...

    fn move_vertically(selection: Selection, amount: usize, text: &Rope, movement: Movement, direction: Direction) -> Selection{
        let goal_line_number = match direction{
            Direction::Forward => text.char_to_line(selection.head).saturating_add(amount),
            Direction::Backward => text.char_to_line(selection.head).saturating_sub(amount)
        };

        let (start_of_line, line_width) = if goal_line_number < text.len_lines(){
            (
                text.line_to_char(goal_line_number),
                text_util::line_width_excluding_newline(text.line(goal_line_number))
            )
        }else{
            (
                text.line_to_char(text.len_lines().saturating_sub(1)),
                text_util::line_width_excluding_newline(text.line(text.len_lines().saturating_sub(1)))
            )
        };


        //let stored_line_position = text_util::offset_from_line_start(selection.head(), text);
        let stored_line_position = match selection.stored_line_position{
            Some(stored_line_position) => stored_line_position,
            None => text_util::offset_from_line_start(selection.head, text)
        };

        if stored_line_position < line_width{
            let new_position = start_of_line + stored_line_position;
            match movement{
                Movement::Move => Selection{
                    anchor: new_position, 
                    head: new_position, 
                    stored_line_position: Some(stored_line_position)//Some(text_util::offset_from_line_start(new_position, text))
                },//Selection::new(new_position, new_position, text),
                Movement::Extend => Selection{
                    anchor: selection.anchor, 
                    head: new_position, 
                    stored_line_position: Some(stored_line_position)//Some(text_util::offset_from_line_start(new_position, text))
                }//Selection::new(selection.anchor(), new_position, text)
            }
        }else{
            let new_position = start_of_line + line_width;
            match movement{
                Movement::Move => Selection{
                    anchor: new_position, 
                    head: new_position, 
                    stored_line_position: Some(stored_line_position)//Some(text_util::offset_from_line_start(selection.head(), text))
                },//Selection::with_stored_line_position(new_position, new_position, selection.stored_line_position(), text),
                Movement::Extend => Selection{
                    anchor: selection.anchor, 
                    head: new_position, 
                    stored_line_position: Some(stored_line_position)//Some(text_util::offset_from_line_start(selection.head(), text))
                }//Selection::with_stored_line_position(selection.anchor(), new_position, selection.stored_line_position(), text)
            }
        }
    }
    //should work with either bar or block cursor semantics
    fn move_horizontally(selection: Selection, amount: usize, text: &Rope, movement: Movement, direction: Direction) -> Selection{
        let new_position = match direction{
            Direction::Forward => selection.head.saturating_add(amount).min(text.len_chars()),    //ensures this does not move past text end
            Direction::Backward => selection.head.saturating_sub(amount)
        };

        match movement{
            Movement::Move => Selection{
                anchor: new_position,
                head: new_position,
                stored_line_position: Some(text_util::offset_from_line_start(new_position, text))
            },//Selection::new(new_position, new_position, text),
            Movement::Extend => Selection{
                anchor: selection.anchor,
                head: new_position,
                stored_line_position: Some(text_util::offset_from_line_start(new_position, text))
            }//Selection::new(selection.anchor(), new_position, text)
        }
    }

    /// Moves cursor to specified char index.
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// 
    /// // zero width selection
    /// let mut selection = Selection::new(0, 0);
    /// selection.move_to(0, &text);
    /// assert!(selection == Selection::with_stored_line_position(0, 0, 0));
    /// selection.move_to(4, &text);
    /// assert!(selection == Selection::with_stored_line_position(4, 4, 0));
    /// 
    /// // non-zero width selection head > anchor
    /// let mut selection = Selection::new(3, 6);
    /// selection.move_to(0, &text);
    /// assert!(selection == Selection::with_stored_line_position(0, 0, 0));
    /// selection.move_to(4, &text);
    /// assert!(selection == Selection::with_stored_line_position(4, 4, 0));
    /// 
    /// // non-zero width selection head < anchor
    /// let mut selection = Selection::new(6, 3);
    /// selection.move_to(0, &text);
    /// assert!(selection == Selection::with_stored_line_position(0, 0, 0));
    /// selection.move_to(4, &text);
    /// assert!(selection == Selection::with_stored_line_position(4, 4, 0));
    /// ```
    pub fn move_to(&mut self, to: usize, text: &Rope){
        if to <= text.len_chars(){
            self.anchor = to;
            self.head = to;
            self.stored_line_position = Some(text_util::offset_from_line_start(self.head, text));
        }
    }

    /// Moves cursor to specified line number.
    /// #### Invariants:
    /// - selection is collapsed
    /// - selection stays within doc bounds
    /// - preserves stored line position
    /// - TODO: ensure head/anchor are grapheme aligned
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Movement};
    /// 
    /// let text = Rope::from("idk\nsomething\nelse");
    /// 
    /// // normal use
    /// let mut selection = Selection::new(0, 0);                          //[]idk\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(14, 14, 0); //idk\nsomething\n[]else
    /// let line_number: usize = 2;
    /// selection.set_from_line_number(line_number, &text, Movement::Move);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// // no change when line num > doc length
    /// let mut selection = Selection::new(0, 0);        //[]idk\nsomething\nelse
    /// let expected_selection = Selection::new(0, 0);   //[]idk\nsomething\nelse
    /// let line_number: usize = 5;
    /// selection.set_from_line_number(line_number, &text, Movement::Move);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// // restricts cursor to line end when stored_line_position > line width
    /// let mut selection = Selection::new(13, 13);                      //idk\nsomething[]\nelse
    /// let expected_selection = Selection::with_stored_line_position(3, 3, 9); //idk[]\nsomething\nelse
    /// let line_number: usize = 0;
    /// selection.set_from_line_number(line_number, &text, Movement::Move);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn set_from_line_number(&mut self, line_number: usize, text: &Rope, movement: Movement){
        if line_number < text.len_lines(){ //is len lines 1 based?
            let start_of_line = text.line_to_char(line_number);
            let line = text.line(line_number);
            let line_width = text_util::line_width_excluding_newline(line);

            let stored_line_position = text_util::offset_from_line_start(self.head, text);

            if stored_line_position < line_width{
                if movement == Movement::Move{
                    self.anchor = start_of_line + stored_line_position;
                }
                self.head = start_of_line + stored_line_position;
                self.stored_line_position = Some(stored_line_position);
            }else{
                if movement == Movement::Move{
                    self.anchor = start_of_line + line_width;
                }
                self.head = start_of_line + line_width;
                self.stored_line_position = Some(stored_line_position);
            }
        }
    }

    /// Aligns [Selection] anchor with head.
    /// #### Invariants:
    /// - selection is within text bounds
    /// # Examples
    /// ``` 
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\n");
    /// 
    /// // head < anchor
    /// let mut selection = Selection::new(4, 0);        //]idk\n[
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);   //[]idk\n
    /// selection.collapse(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // anchor < head
    /// let mut selection = Selection::new(0, 4);        //[idk\n]
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 0);   //idk\n[]
    /// selection.collapse(&text);
    /// assert!(selection == expected_selection);
    /// ```
    // TODO: figure out block cursor semantics
    pub fn collapse(&mut self, text: &Rope){
        self.anchor = self.head;
        self.stored_line_position = Some(text_util::offset_from_line_start(self.head, text));
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
    /// let text = Rope::from("012\n");
    /// 
    /// // stays within doc bounds
    /// let mut selection = Selection::new(4, 4);                        //012\n[]
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 0); //012\n[]
    /// selection.move_right(&text);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// // normal use
    /// let mut selection = Selection::new(0, 0);        //[]012\n
    /// let expected_selection = Selection::with_stored_line_position(1, 1, 1); //0[]12\n
    /// selection.move_right(&text);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// 
    /// // new line resets stored line position
    /// let text = Rope::from("012\n0");
    /// let mut selection = Selection::new(3, 3);        //012[]\n0
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 0); //012\n[]0
    /// selection.move_right(&text);
    /// println!("expected: {:#?}\ngot: {:#?}\n", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_right(&mut self, text: &Rope){
        if self.is_extended(){
            self.collapse(&text);
        }
        *self = Selection::move_horizontally(self.clone(), 1, text, Movement::Move, Direction::Forward)
        //TODO: we can ensure block cursor does not go past doc end by checking against self.end() instead of head or anchor
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
    /// let text = Rope::from("idk\nsomething\nelse");
    /// 
    /// // stays within doc bounds
    /// let mut selection = Selection::new(0, 0);       //[]idk\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);  //[]idk\nsomething\nelse
    /// selection.move_left(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // normal use
    /// let mut selection = Selection::new(2, 2);                        //id[]k\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(1, 1, 1); //i[]dk\nsomething\nelse
    /// selection.move_left(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // move to previous line resets stored line position
    /// let mut selection = Selection::new(4, 4);                        //idk\n[]something\nelse
    /// let expected_selection = Selection::with_stored_line_position(3, 3, 3); //idk[]\nsomething\nelse
    /// selection.move_left(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_left(&mut self, text: &Rope){
        if self.is_extended(){
            self.collapse(&text);
        }
        *self = Selection::move_horizontally(self.clone(), 1, text, Movement::Move, Direction::Backward)
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
    /// let text = Rope::from("idk\nsomething\nelse");
    /// 
    /// // stays within doc bounds
    /// let mut selection = Selection::new(0, 0);       //[]idk\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);  //[]idk\nsomething\nelse
    /// selection.move_up(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // to shorter line
    /// let mut selection = Selection::new(13, 13);                      //idk\nsomething[]\nelse
    /// let expected_selection = Selection::with_stored_line_position(3, 3, 9); //idk[]\nsomething\nelse
    /// selection.move_up(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // to longer line
    /// let mut selection = Selection::new(18, 18);                      //idk\nsomething\nelse[]
    /// let expected_selection = Selection::with_stored_line_position(8, 8, 4); //idk\nsome[]thing\nelse
    /// selection.move_up(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_up(&mut self, text: &Rope){
        if self.is_extended(){
            self.collapse(&text);
        }
        *self = Selection::move_vertically(self.clone(), 1, text, Movement::Move, Direction::Backward)
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
    /// // stays within doc bounds
    /// let text = Rope::from("012\n");
    /// let mut selection = Selection::new(4, 4);      //012\n[]
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 0); //012\n[]
    /// selection.move_down(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // to shorter line
    /// let text = Rope::from("012\n0");
    /// let mut selection = Selection::new(3, 3);                        //012[]\n0
    /// let expected_selection = Selection::with_stored_line_position(5, 5, 3); //012\n0[]
    /// selection.move_down(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // to longer line
    /// let text = Rope::from("idk\nsomething\nelse");
    /// let mut selection = Selection::new(3, 3);                        //idk[]\nsomething\nelse
    /// let expected_selection = Selection::with_stored_line_position(7, 7, 3); //idk\nsom[]ething\nelse
    /// selection.move_down(&text);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn move_down(&mut self, text: &Rope){
        if self.is_extended(){
            self.collapse(&text);
        }
        *self = Selection::move_vertically(self.clone(), 1, text, Movement::Move, Direction::Forward)
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
        if self.is_extended(){
            self.collapse(&text);
        }
        let line_number = text.char_to_line(self.head);
        let line = text.line(line_number);
        let line_width = text_util::line_width_excluding_newline(line);
        let line_start = text.line_to_char(line_number);
        let line_end = line_start.saturating_add(line_width);

        self.head = line_end;
        self.anchor = line_end;
        self.stored_line_position = Some(line_end.saturating_sub(line_start));
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
    /// let text = Rope::from("    idk\n");
    /// 
    /// // moves to text start when cursor past text start
    /// let mut selection = Selection::new(6, 6);                        //    id[]k\n
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 4); //    []idk\n
    /// selection.move_home(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // moves to line start when cursor at text start
    /// let mut selection = Selection::new(4, 4); //    []idk\n
    /// let expected_selection = Selection::with_stored_line_position(0, 0, 0);   //[]    idk\n
    /// selection.move_home(&text);
    /// assert!(selection == expected_selection);
    /// 
    /// // moves to text start when cursor before text start
    /// let mut selection = Selection::new(1, 1);                        // []   idk\n
    /// let expected_selection = Selection::with_stored_line_position(4, 4, 4); //    []idk\n
    /// selection.move_home(&text);
    /// assert!(selection == expected_selection);
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
        if self.is_extended(){
            self.collapse(&text);
        }
        let line_number = text.char_to_line(self.head);
        let line_start = text.line_to_char(line_number);

        self.head = line_start;
        self.anchor = line_start;
        self.stored_line_position = Some(self.head.saturating_sub(line_start));
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
        if self.is_extended(){
            self.collapse(&text);
        }
        let line_number = text.char_to_line(self.head);
        let line_start = text.line_to_char(line_number);
        let text_start_offset = text_util::first_non_whitespace_character_offset(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        self.head = text_start;
        self.anchor = text_start;
        self.stored_line_position = Some(self.head.saturating_sub(line_start));
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
        if self.is_extended(){
            self.collapse(&text);
        }
        *self = Selection::move_vertically(self.clone(), client_view.height().saturating_sub(1), text, Movement::Move, Direction::Backward)
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
        if self.is_extended(){
            self.collapse(&text);
        }
        *self = Selection::move_vertically(self.clone(), client_view.height().saturating_sub(1), text, Movement::Move, Direction::Forward)
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
        if self.is_extended(){
            self.collapse(&text);
        }
        self.head = 0;
        self.anchor = 0;
        self.stored_line_position = Some(0);
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
        if self.is_extended(){
            self.collapse(text);
        }
        self.head = text.len_chars();
        self.anchor = text.len_chars();
        self.stored_line_position = Some(text_util::offset_from_line_start(self.head, text));
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
    /// ```
    pub fn extend_right(&mut self, text: &Rope){
        *self = Selection::move_horizontally(self.clone(), 1, text, Movement::Extend, Direction::Forward)
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
        *self = Selection::move_horizontally(self.clone(), 1, text, Movement::Extend, Direction::Backward)
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
        *self = Selection::move_vertically(self.clone(), 1, text, Movement::Extend, Direction::Backward)
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
        *self = Selection::move_vertically(self.clone(), 1, text, Movement::Extend, Direction::Forward)
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

        self.head = line_end;
        self.stored_line_position = Some(line_end.saturating_sub(line_start));
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
            //self.head = line_start;
            self.extend_line_start(text);
        }else{
            //self.head = text_start;
            self.extend_line_text_start(text);
        }
        //self.stored_line_position = self.head.saturating_sub(line_start);
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

        self.head = line_start;
        self.stored_line_position = Some(self.head.saturating_sub(line_start));
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

        self.head = text_start;
        self.stored_line_position = Some(self.head.saturating_sub(line_start));
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
        *self = Selection::move_vertically(self.clone(), client_view.height().saturating_sub(1), text, Movement::Extend, Direction::Backward)
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
        *self = Selection::move_vertically(self.clone(), client_view.height().saturating_sub(1), text, Movement::Extend, Direction::Forward)
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
    /// selection.extend_doc_start();
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn extend_doc_start(&mut self){
        self.head = 0;
        self.stored_line_position = Some(0);
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
        self.head = text.len_chars();
        self.stored_line_position = Some(text_util::offset_from_line_start(self.head, text));
    }
    
    /// Extends selection to encompass whole doc.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::Selection;
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selection = Selection::new(4, 4);
    /// let expected_selection = Selection::with_stored_line_position(0, 14, 0);
    /// selection.select_all(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selection, selection);
    /// assert!(selection == expected_selection);
    /// ```
    pub fn select_all(&mut self, text: &Rope){
        self.anchor = 0;
        self.head = text.len_chars();
        self.stored_line_position = Some(text_util::offset_from_line_start(self.head, text));
    }

    /// Translates a [Selection] to a [Selection2d]
    /// #### Invariants:
    /// - TODO
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selection2d};
    /// # use edit::Position;
    /// 
    /// let text = Rope::from("idk\nsomething");
    /// 
    /// // when selection head/anchor same, and on same line
    /// let selection = Selection::new(2, 2);  //id[]k\nsomething
    /// let doc_cursor = selection.selection_to_selection2d(&text);
    /// let expected_doc_cursor = Selection2d::new(Position::new(2, 0), Position::new(2, 0));
    /// /*
    /// id[]k
    /// something
    /// */
    /// assert!(doc_cursor == expected_doc_cursor);
    /// 
    /// // when selection head/anchor different, but on same line
    /// let selection = Selection::new(1, 2);  //i[d]k\nsomething
    /// let doc_cursor = selection.selection_to_selection2d(&text);
    /// let expected_doc_cursor = Selection2d::new(Position::new(2, 0), Position::new(1, 0));
    /// /*
    /// i[d]k
    /// something
    /// */
    /// assert!(doc_cursor == expected_doc_cursor);
    /// 
    /// // when selection head/anchor same, but on new line
    /// let selection = Selection::new(4, 4);  //idk\n[]something
    /// let doc_cursor = selection.selection_to_selection2d(&text);
    /// let expected_doc_cursor = Selection2d::new(Position::new(0, 1), Position::new(0, 1));
    /// /*
    /// idk
    /// []something
    /// */
    /// assert!(doc_cursor == expected_doc_cursor);
    /// 
    /// // when selection head/anchor different, and on different lines
    /// let selection = Selection::new(2, 5);  //id[k\ns]omething
    /// let doc_cursor = selection.selection_to_selection2d(&text);
    /// let expected_doc_cursor = Selection2d::new(Position::new(1, 1), Position::new(2, 0));
    /// /*
    /// id[k
    /// s]omething
    /// */
    /// assert!(doc_cursor == expected_doc_cursor);
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

    /// Moves cursors up.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(4, 4),
    ///     Selection::new(9, 9)
    /// ], 0, &text);
    /// selections.move_cursors_up(&text);
    /// assert!(selections == Selections::new(vec![
    ///     Selection::with_stored_line_position(0, 0, 0),
    ///     Selection::with_stored_line_position(4, 4, 0)
    /// ], 0, &text));
    /// ```
    pub fn move_cursors_up(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.move_up(text);
        }
        // should merge overlapping be done here?
    }

    /// Moves cursors down.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 0),
    ///     Selection::new(4, 4)
    /// ], 0, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::with_stored_line_position(4, 4, 0),
    ///     Selection::with_stored_line_position(9, 9, 0)
    /// ], 0, &text);
    /// selections.move_cursors_down(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn move_cursors_down(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.move_down(text);
        }
    }

    /// Moves cursors right.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 0),
    ///     Selection::new(4, 4),
    ///     Selection::new(9, 9)
    /// ], 0, &text);
    /// selections.move_cursors_right(&text);
    /// assert!(selections == Selections::new(vec![
    ///     Selection::with_stored_line_position(1, 1, 1),
    ///     Selection::with_stored_line_position(5, 5, 1),
    ///     Selection::with_stored_line_position(10, 10, 1)
    /// ], 0, &text));
    /// ```
    pub fn move_cursors_right(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.move_right(text);
        }
    }

    /// Moves cursors left.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 0),
    ///     Selection::new(4, 4)
    /// ], 0, &text);
    /// selections.move_cursors_left(&text);
    /// assert!(selections == Selections::new(vec![
    ///     Selection::with_stored_line_position(0, 0, 0),
    ///     Selection::with_stored_line_position(3, 3, 3)
    /// ], 0, &text));
    /// ```
    pub fn move_cursors_left(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.move_left(text);
        }
    }

    pub fn move_cursors_page_up(&mut self, text: &Rope, client_view: &View){
        for selection in self.selections.iter_mut(){
            selection.move_page_up(text, client_view);
        }
    }

    pub fn move_cursors_page_down(&mut self, text: &Rope, client_view: &View){
        for selection in self.selections.iter_mut(){
            selection.move_page_down(text, client_view);
        }
    }

    /// Moves cursors to start of line, or start of text, depending on cursor position.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(8, 8),
    ///     Selection::new(13, 13)
    /// ], 0, &text);
    /// selections.move_cursors_home(&text);
    /// assert!(selections == Selections::new(vec![
    ///     Selection::with_stored_line_position(4, 4, 0),
    ///     Selection::with_stored_line_position(9, 9, 0)
    /// ], 0, &text));
    /// ```
    pub fn move_cursors_home(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.move_home(text);
        }
    }

    /// Moves cursors to the end of a line of text.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 0),
    ///     Selection::new(4, 4)
    /// ], 0, &text);
    /// selections.move_cursors_end(&text);
    /// assert!(selections == Selections::new(vec![
    ///     Selection::with_stored_line_position(3, 3, 3),
    ///     Selection::with_stored_line_position(8, 8, 4)
    /// ], 0, &text));
    /// ```
    pub fn move_cursors_end(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.move_line_text_end(text);
        }
    }

    /// Moves cursors to the start of the document.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(4, 4),
    ///     Selection::new(13, 13)
    /// ], 0, &text);
    /// selections.move_cursors_document_start(&text);
    /// assert!(selections == Selections::new(vec![Selection::with_stored_line_position(0, 0, 0)], 0, &text));
    /// ```
    pub fn move_cursors_document_start(&mut self, text: &Rope){
        self.clear_non_primary_selections();
        self.first_mut().move_doc_start(&text);
    }

    /// Moves cursor to the end of the document.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 0),
    ///     Selection::new(4, 4)
    /// ], 0, &text);
    /// selections.move_cursors_document_end(&text);
    /// assert!(selections == Selections::new(vec![Selection::with_stored_line_position(14, 14, 0)], 0, &text));
    /// ```
    pub fn move_cursors_document_end(&mut self, text: &Rope){
        self.clear_non_primary_selections();
        self.first_mut().move_doc_end(text);
    }

    /// Extends document [Selection]s to the right.
    /// #### Invariants:
    /// - TODO
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 0),
    ///     Selection::new(4, 4)
    /// ], 0, &text);
    /// selections.extend_selections_right(&text);
    /// assert!(selections == Selections::new(vec![
    ///     Selection::with_stored_line_position(0, 1, 1),
    ///     Selection::with_stored_line_position(4, 5, 1)
    /// ], 0, &text));
    /// ```
    pub fn extend_selections_right(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.extend_right(text);
        }
    }

    /// Extends document [Selection]s to the left.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(1, 1),
    ///     Selection::new(5, 5)
    /// ], 0, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::with_stored_line_position(1, 0, 0),
    ///     Selection::with_stored_line_position(5, 4, 0)
    /// ], 0, &text);
    /// selections.extend_selections_left(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn extend_selections_left(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.extend_left(text);
        }
    }

    /// Extends document [Selection]s up one line.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(4, 4),
    ///     Selection::new(9, 9)
    /// ], 0, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::with_stored_line_position(4, 0, 0),
    ///     Selection::with_stored_line_position(9, 4, 0)
    /// ], 0, &text);
    /// selections.extend_selections_up(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn extend_selections_up(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.extend_up(text);
        }
    }

    /// Extends document [Selection]s down one line.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    /// 
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 0),
    ///     Selection::new(4, 4)
    /// ], 0, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::with_stored_line_position(0, 4, 0),
    ///     Selection::with_stored_line_position(4, 9, 0)
    /// ], 0, &text);
    /// selections.extend_selections_down(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn extend_selections_down(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.extend_down(text);
        }
    }

    /// Extends document [Selection]s to line start or start of line text, depending on cursor position in line.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    ///
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(3, 3),
    ///     Selection::new(8, 8)
    /// ], 0, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::with_stored_line_position(3, 0, 0),
    ///     Selection::with_stored_line_position(8, 4, 0)
    /// ], 0, &text);
    /// selections.extend_selections_home(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn extend_selections_home(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.extend_home(text);
        }
    }

    /// Extends document [Selection]s to the end of a line.
    /// # Example
    /// ```
    /// # use ropey::Rope;
    /// # use edit::selection::{Selection, Selections};
    ///
    /// let text = Rope::from("idk\nsome\nshit\n");
    /// let mut selections = Selections::new(vec![
    ///     Selection::new(0, 0),
    ///     Selection::new(4, 4)
    /// ], 0, &text);
    /// let expected_selections = Selections::new(vec![
    ///     Selection::with_stored_line_position(0, 3, 3),
    ///     Selection::with_stored_line_position(4, 8, 4)
    /// ], 0, &text);
    /// selections.extend_selections_end(&text);
    /// println!("expected: {:#?}\ngot: {:#?}", expected_selections, selections);
    /// assert!(selections == expected_selections);
    /// ```
    pub fn extend_selections_end(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.extend_line_text_end(text);
        }
    }

    pub fn extend_selections_page_up(&mut self, text: &Rope, client_view: &View){
        for selection in self.selections.iter_mut(){
            selection.extend_page_up(text, client_view);
        }
    }
    pub fn extend_selections_page_down(&mut self, text: &Rope, client_view: &View){
        for selection in self.selections.iter_mut(){
            selection.extend_page_down(text, client_view);
        }
    }

    pub fn select_all(&mut self, text: &Rope){
        self.clear_non_primary_selections();
        self.first_mut().select_all(text);
    }

    pub fn go_to(&mut self, text: &Rope, line_number: usize){
        self.clear_non_primary_selections();
        self.first_mut().set_from_line_number(
            line_number, 
            text, 
            crate::selection::Movement::Move
        );
    }
    
    // adds selection on line above first selection
    //pub fn add_selection_above(&mut self, text: RopeSlice){
    //    let topmost_selection = self.first();
    //    // get line_number of previous line.
    //    let prev_line_number = text.char_to_line(topmost_selection.head()).saturating_sub(1);
    //    // get anchor, head, stored_line_position for new selection
    //    let anchor = unimplemented!();
    //    let head = unimplemented!();
    //    let stored_line_position = topmost_selection.stored_line_position();
    //    let selection = Selection::new(anchor, head, stored_line_position)
    //    // push new selection to front of selections
    //}
    // adds selections on line below last selection
    //pub fn add_selection_below(&mut self, text: RopeSlice){}

    pub fn collapse_selections(&mut self, text: &Rope){
        for selection in self.selections.iter_mut(){
            selection.collapse(&text);
        }
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

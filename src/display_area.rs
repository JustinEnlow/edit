//use ropey::Rope;
use crate::range::Range;
use crate::selection::{CursorSemantics, Selection};
use crate::selection2d::Selection2d;
use crate::selections::Selections;
use crate::position::Position;

// note: not relevant to current implementation. just my intuition regarding how this ought to be considered
// in the context of a view(or view block), a range should represent an index pair over terminal cells.
// this may conflict with the buffers idea of a range representing an index pair over graphemes(which can span multiple terminal cells)

#[derive(Debug, PartialEq)]
pub enum DisplayAreaError{
    ResultsInSameState,
    InvalidInput,
}
/// The dimensions of the area a client has for displaying a document
/// origin is top left
/// 
/// the client should be the single source of truth for width + height, so maybe those should be passed in to relevant functions instead...
/// 
///// however, `horizontal_start` + `vertical_start` need to be held in core, because the client does not have a full view of the
///// text buffer, and some core functionality needs to modify these values
///// this issue could prob be alleviated if the client can get a full view of the text buffer...
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DisplayArea{
    /// from left to right
    pub horizontal_start: usize,
    /// from top to bottom
    pub vertical_start: usize,
    pub width: usize,
    pub height: usize,
}
impl DisplayArea{
    /// Returns a new instance of [`View`] from provided inputs.
    #[must_use] pub fn new(horizontal_start: usize, vertical_start: usize, width: usize, height: usize) -> Self{
        Self{horizontal_start, vertical_start, width, height}
    }
    pub fn set_size(&mut self, width: usize, height: usize){
        self.width = width;
        self.height = height;
    }
    #[must_use] pub fn height(&self) -> usize{
        self.height
    }
    #[must_use] pub fn horizontal_start(&self) -> usize{
        self.horizontal_start
    }

    /// Returns a `bool` indicating whether the [`View`] should be scrolled or not. If `head` of primary [`Selection2d`]
    /// is outside [`View`] boundaries, [`View`] should be scrolled.
    /// # Panics
    /// when `selection` is invalid.
    #[must_use] pub fn should_scroll(&self, selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> bool{
        assert!(selection.cursor(buffer, semantics.clone()) <= buffer.len_chars());

        let cursor = selection.selection_to_selection2d(buffer, semantics);
        let cursor_y = cursor.head().y;
        let cursor_x = cursor.head().x;

        let within_vertical_bounds = cursor_y >= self.vertical_start && cursor_y < self.vertical_start.saturating_add(self.height);
        let within_horizontal_bounds = cursor_x >= self.horizontal_start && cursor_x < self.horizontal_start.saturating_add(self.width);

        !(within_vertical_bounds && within_horizontal_bounds)
    }

    /// Returns a new instance of [`View`] with `horizontal_start` and/or `vertical_start` shifted to keep `head` of
    /// [`Selection`] in [`View`].
    /// Can follow any specified selection, not just primary selection.
    /// # Panics
    /// when `selection` is invalid.
    #[must_use] pub fn scroll_following_cursor(&self, selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Self{
        assert!(selection.cursor(buffer, semantics.clone()) <= buffer.len_chars());

        let cursor = selection.selection_to_selection2d(buffer, semantics);
        let cursor_y = cursor.head().y;
        let cursor_x = cursor.head().x;

        let mut new_view = self.clone();

        // Adjust vertical view based on cursor position
        if cursor_y < self.vertical_start{
            new_view.vertical_start = cursor_y;
        }else if cursor_y >= self.vertical_start.saturating_add(self.height){
            new_view.vertical_start = cursor_y.saturating_sub(self.height).saturating_add(1);
        }

        // Adjust horizontal view based on cursor position
        if cursor_x < self.horizontal_start{
            new_view.horizontal_start = cursor_x;
        }else if cursor_x >= self.horizontal_start.saturating_add(self.width){
            new_view.horizontal_start = cursor_x.saturating_sub(self.width).saturating_add(1);
        }

        new_view
    }

    /// Returns a `String` containing the text that can be contained within [`View`] boundaries.
    // TODO: need to handle displaying TAB_WIDTH spaces instead of a "\t" character.
    // TODO: test
    //pub fn text(&self, text: &Rope) -> String{
    //    // preallocate memory for String based on expected size
    //    let mut client_view_text = String::with_capacity(self.height * (self.width + 1));   //+1 for added new line
    //
    //    let vertical_range = self.vertical_start..self.vertical_start + self.height;
    //    let horizontal_range = self.horizontal_start..self.horizontal_start + self.width;
    //
    //    for (y, line) in text.lines().enumerate(){
    //        if !vertical_range.contains(&y){
    //            continue;
    //        }
    //
    //        let bounded_line: String = line.chars()
    //            .enumerate()
    //            .filter_map(|(x, char)|{
    //                if horizontal_range.contains(&x) && char != '\n'{
    //                    Some(char)
    //                }else{
    //                    None
    //                }
    //            })
    //            .collect();
    //
    //        client_view_text.push_str(&bounded_line);
    //        client_view_text.push('\n'); // Append newline after each line
    //    }
    //
    //    client_view_text
    //}
    // returns text using view blocks, but may be harder to implement hard tab handling, or other wide characters
    #[must_use] pub fn text(&self, buffer: &crate::buffer::Buffer) -> String{  //TODO: take args space_replacement: Option<char>, newline_replacement: Option<char>, tab_replacement: Option<char>  //this would allow us to display a different character in place of tabs/spaces/newlines so their use is more obvious in the frontend app
        let view_blocks = self.view_blocks(buffer, false);
        let mut client_view_text = String::new();

        for view_block in &view_blocks{ //view_blocks.iter(){   //change suggested by clippy lint
            //client_view_text.push_str(&buffer.inner.slice(view_block.start..view_block.end).to_string());
            client_view_text.push_str(&buffer.slice(view_block.start, view_block.end));
            client_view_text.push('\n');
        }
    
        client_view_text
    }
    

    /// Returns a `String` containing the line numbers of the text that can be contained within [`View`] boundaries.
    #[must_use] pub fn line_numbers(&self, buffer: &crate::buffer::Buffer) -> String{
        //enhance performance by building the string using a vector and then joining it at the end
        let mut line_numbers_vec = Vec::with_capacity(self.height);

        let vertical_range = self.vertical_start..self.vertical_start + self.height;

        for (y, _) in buffer.inner.lines().enumerate(){
            if vertical_range.contains(&y){
                line_numbers_vec.push((y + 1).to_string()); // Convert number to string
            }
        }

        line_numbers_vec.join("\n") // Join with newline
    }
    /// Returns a `String` containing the line numbers, relative to the primary cursor, of the text that can be contained within [`View`] boundaries.
    #[must_use] pub fn relative_line_numbers(&self, _buffer: &crate::buffer::Buffer) -> String{
        String::new()
    }

    /// Returns a [`Vec`] of [`Selection2d`]s that represent [`Selection`]s with any portion of itself within the boundaries of [`View`].
    /// Returned selections should be in screen space coordinates.
    /// Assumes selections are already sorted and merged.
    #[must_use] pub fn selections(&self, selections: &Selections, buffer: &crate::buffer::Buffer) -> Vec<Selection2d>{
        let view_blocks = self.view_blocks(buffer, true); //make sure to adjust tests to include newline
        let mut selections_in_view = Vec::with_capacity(view_blocks.len() * self.width);

        for (y, view_block) in view_blocks.iter().enumerate(){
            let view_start = view_block.start;
            //let mut intersected = false;
            for selection in selections.iter(){
                if let Some(selected_in_view) = view_block.intersection(&selection.range){
                    // add intersecting to list //this represents a selection in view bounds
                    let new_anchor = Position::new(selected_in_view.start - view_start, y); //TODO: sat_sub
                    let new_head = Position::new(selected_in_view.end - view_start, y); //TODO: sat_sub
                    selections_in_view.push(Selection2d::new(new_anchor, new_head));
                    //intersected = true;
                }
            }

            // ATTENTION!!! commenting this out until using the editor proves this is needed for some reason...
            //if !intersected{
            //    // retain non intersecting view_blocks  //this represents no selection in view              //TODO: why retain non intersecting view blocks?...
            //    selections_in_view.push(Selection2d::new(Position::new(0, y), Position::new(0, y)));
            //}
        }

        selections_in_view
    }
    //TODO: maybe have fn selection(&self, selection: &Selection, buffer: &Buffer) -> Option<Selection2d>
    //this would be useful for util_bar text_box view, but could also be done for all [Selection]s in selections, to replace fn selections

    /// Maps a [`View`] as a [`Vec`] of [`Range`]s over a text rope.
    /// This transforms the idea of a view from 2d to 1d, one view block per terminal row, and trims excess empty cells from each row.
    // should this include newlines('\n') in its width calculation? maybe pass in include_newline bool?
    // we want to highlight newlines as well
    // but that may mess with the logic for "empty" lines...idk
    #[must_use] pub fn view_blocks(&self, buffer: &crate::buffer::Buffer, include_newline: bool) -> Vec<Range>{
        let mut view_blocks = Vec::new();
        let vertical_range = self.vertical_start..self.vertical_start + self.height;

        for (y, _) in buffer.inner.lines().enumerate(){
            // only include lines in vertical bounds
            if vertical_range.contains(&y){
                let line_start = buffer.inner.line_to_char(y);
                let line_width = buffer.line_width(y, include_newline);//crate::text_util::line_width(line, include_newline);
                let line_end = line_start + line_width;
                
                let mut view_start = line_start + self.horizontal_start;    //start view at horizontal offset of view
                let mut view_end = view_start + line_width.min(self.width); //restrict view width

                if line_end < view_start{   //handle view shifted right past end of line text(includes empty lines)
                    view_start = line_start;
                    view_end = line_start;  //zero width output ranges represent a line with no text in view bounds
                }
                else if line_end < view_end{
                    view_end = line_end;
                }
                view_blocks.push(Range::new(view_start, view_end));
            }
        }

        view_blocks
    }
    
    // translates a document cursor position to a client view cursor position. if outside client view, returns None
    fn cursor_position(doc_cursor: &Selection2d, client_view: &DisplayArea) -> Option<Position>{
        let head_x = doc_cursor.head().x;
        let head_y = doc_cursor.head().y;

        let in_horizontal_bounds = head_x >= client_view.horizontal_start
            && head_x < client_view.horizontal_start.saturating_add(client_view.width);
    
        let in_vertical_bounds = head_y >= client_view.vertical_start
            && head_y < client_view.vertical_start.saturating_add(client_view.height);

        if in_horizontal_bounds && in_vertical_bounds{
            Some(Position::new(
                head_x.saturating_sub(client_view.horizontal_start),
                head_y.saturating_sub(client_view.vertical_start),
            ))
        }else{
            None
        }
    }
    /// Returns [`Position`] of primary cursor if it is within [`View`] boundaries, or None otherwise.
    #[must_use] pub fn primary_cursor_position(&self, buffer: &crate::buffer::Buffer, selections: &Selections, semantics: CursorSemantics) -> Option<Position>{
        let primary = selections.primary();
        Self::cursor_position(&primary.selection_to_selection2d(buffer, semantics), self)
    }
    /// Returns [`Position`]s of cursors that are within [`View`] boundaries, or an empty vec otherwise.
    #[must_use] pub fn cursor_positions(&self, buffer: &crate::buffer::Buffer, selections: &Selections, semantics: CursorSemantics) -> Vec<Position>{
        selections.iter()
            .filter_map(|cursor|{
                Self::cursor_position(&cursor.selection_to_selection2d(buffer, semantics.clone()), self)
            })
            .collect()
    }
}

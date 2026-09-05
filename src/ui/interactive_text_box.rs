//TODO: util bar view should scroll differently than buffer view
// when inserting text, view scroll is ok
// when backspacing text, view should move with cursor

//TODO: impl cut/copy/paste for text box

use crate::{
    buffer::Buffer,
    selection::{self, Selection, CursorSemantics},
};



pub struct InteractiveTextBox{
    pub buffer: Buffer,
    pub text_is_valid: bool,
    pub selection: Selection,
    pub display_area_horizontal_start: usize,
    pub display_area_vertical_start: usize,
}
impl Default for InteractiveTextBox{
    fn default() -> Self{
        let buffer = Buffer::new("", None, false);
        Self{
            buffer: buffer.clone(),
            text_is_valid: false,
            selection: Selection::new(
                0..buffer.next_grapheme_boundary_byte_offset(0),
                None,
                &buffer, 
                CursorSemantics::Block
            ),
            display_area_horizontal_start: 0,
            display_area_vertical_start: 0
        }
    }
}
impl InteractiveTextBox{
    pub fn cursor_position(&self) -> u16{
        self.selection.cursor(&self.buffer, CursorSemantics::Block) as u16
    }
    pub fn clear(&mut self){
        *self = Self::default();
    }
    pub fn insert_char(&mut self, char: char){
        //if self.selection.is_extended(){
        //    self.delete();    
        //}
        //let text = self.buffer.clone();
        //let mut new_text = text.clone();
        //new_text.insert(self.selection.cursor(&text, CURSOR_SEMANTICS), &char.to_string());
        //self.buffer = new_text;
        //if let Ok(new_selection) = crate::utilities::move_cursor_right::selection_impl(&self.selection, 1, &self.buffer, None, CURSOR_SEMANTICS){
        //    self.selection = new_selection;
        //}

        // figure out how to use buffer.apply_insert/replace here...
        if self.selection.is_extended(){
            //self.buffer.apply_replace(&char.to_string(), &mut self.selection, CursorSemantics::Block);
            crate::application::apply_replace(&mut self.buffer, &char.to_string(), &mut self.selection, CursorSemantics::Block);
        }else{
            //self.buffer.apply_insert(&char.to_string(), &mut self.selection, CursorSemantics::Block);
            crate::application::apply_insert(&mut self.buffer, &char.to_string(), &mut self.selection, CursorSemantics::Block);
        }
        //
    }
    pub fn delete(&mut self){
        //let text = self.buffer.clone();
        //let mut new_text = self.buffer.clone();
    //
        //match self.selection.cursor(&text, CURSOR_SEMANTICS).cmp(&self.selection.anchor()){
        //    Ordering::Less => {
        //        //new_text.remove(self.selection.head()..self.selection.anchor());
        //        new_text.remove(self.selection.head(), self.selection.anchor());
        //        if let Ok(new_selection) = self.selection.put_cursor(self.selection.cursor(&text, CURSOR_SEMANTICS), &text, Movement::Move, CURSOR_SEMANTICS, true){
        //            self.selection = new_selection;
        //        }
        //    }
        //    Ordering::Greater => {
        //        if self.selection.cursor(&text, CURSOR_SEMANTICS) == text.len_chars(){
        //            //new_text.remove(self.selection.anchor()..self.selection.cursor(&text, CURSOR_SEMANTICS));
        //            new_text.remove(self.selection.anchor(), self.selection.cursor(&text, CURSOR_SEMANTICS));
        //        }
        //        else{
        //            //new_text.remove(self.selection.anchor()..self.selection.head());
        //            new_text.remove(self.selection.anchor(), self.selection.head());
        //        }
        //        if let Ok(new_selection) = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CURSOR_SEMANTICS, true){
        //            self.selection = new_selection;
        //        }
        //    }
        //    Ordering::Equal => {
        //        if self.selection.cursor(&text, CURSOR_SEMANTICS) == text.len_chars(){}    //do nothing
        //        else{
        //            //new_text.remove(self.selection.anchor()..self.selection.head());
        //            new_text.remove(self.selection.anchor(), self.selection.head());
        //            if let Ok(new_selection) = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CURSOR_SEMANTICS, true){
        //                self.selection = new_selection;
        //            }
        //        }
        //    }
        //}
    //
        //self.buffer = new_text;

        // figure out how to use buffer.apply_delete here...
        //match self.selection.cursor(&self.buffer, CURSOR_SEMANTICS).cmp(&self.selection.anchor()){
        //    Ordering::Less => {
        //        self.buffer.apply_delete(&mut self.selection, CURSOR_SEMANTICS);
        //        //if let Ok(new_selection) = self.selection.put_cursor(self.selection.cursor(&self.buffer, CURSOR_SEMANTICS), &self.buffer, Movement::Move, CURSOR_SEMANTICS, true){
        //        //    self.selection = new_selection;
        //        //}
        //    }
        //    Ordering::Greater => {
        //        if self.selection.cursor(&self.buffer, CURSOR_SEMANTICS) == self.buffer.len_chars(){
        //            self.buffer.apply_delete(&mut self.selection, CURSOR_SEMANTICS);
        //        }
        //        else{
        //            self.buffer.apply_delete(&mut self.selection, CURSOR_SEMANTICS);
        //        }
        //        //if let Ok(new_selection) = self.selection.put_cursor(self.selection.anchor(), &self.buffer, Movement::Move, CURSOR_SEMANTICS, true){
        //        //    self.selection = new_selection;
        //        //}
        //    }
        //    Ordering::Equal => {
        //        if self.selection.cursor(&self.buffer, CURSOR_SEMANTICS) == self.buffer.len_chars(){}    //do nothing
        //        else{
        //            self.buffer.apply_delete(&mut self.selection, CURSOR_SEMANTICS);
        //            //if let Ok(new_selection) = self.selection.put_cursor(self.selection.anchor(), &self.buffer, Movement::Move, CURSOR_SEMANTICS, true){
        //            //    self.selection = new_selection;
        //            //}
        //        }
        //    }
        //}
        //self.buffer.apply_delete(&mut self.selection, CursorSemantics::Block);
        crate::application::apply_delete(&mut self.buffer, &mut self.selection, CursorSemantics::Block);
        //
    }
    #[allow(clippy::collapsible_else_if)]
    pub fn backspace(&mut self){
        let semantics = CursorSemantics::Block;
        if self.selection.is_extended(){
            self.delete();
        }else{
            if self.selection.cursor(&self.buffer, semantics) > 0{
                //if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(&self.selection, 1, &self.buffer, None, CURSOR_SEMANTICS){
                if let Ok(new_selection) = selection::move_cursor_left(&self.selection, 1, &self.buffer, None, CursorSemantics::Block){
                    self.selection = new_selection;
                }
                self.delete();
            }
        }
    }



    pub fn extend_selection_end(&mut self){
        //if let Ok(new_selection) = crate::utilities::extend_selection_line_end::selection_impl(&self.selection, &self.buffer, CURSOR_SEMANTICS){
        if let Ok(new_selection) = selection::extend_selection_line_end(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_home(&mut self){
        //if let Ok(new_selection) = crate::utilities::extend_selection_home::selection_impl(&self.selection, &self.buffer, CURSOR_SEMANTICS){
        if let Ok(new_selection) = selection::extend_selection_home(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_left(&mut self){
        //if let Ok(new_selection) = crate::utilities::extend_selection_left::selection_impl(&self.selection, 1, &self.buffer, None, CURSOR_SEMANTICS){
        if let Ok(new_selection) = selection::extend_selection_left(&self.selection, 1, &self.buffer, None, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_right(&mut self){
        //if let Ok(new_selection) = crate::utilities::extend_selection_right::selection_impl(&self.selection, 1, &self.buffer, None, CURSOR_SEMANTICS){
        if let Ok(new_selection) = selection::extend_selection_right(&self.selection, 1, &self.buffer, None, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_left(&mut self){
        //if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(&self.selection, 1, &self.buffer, None, CURSOR_SEMANTICS){
        if let Ok(new_selection) = selection::move_cursor_left(&self.selection, 1, &self.buffer, None, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_line_end(&mut self){
        //if let Ok(new_selection) = crate::utilities::move_cursor_line_end::selection_impl(&self.selection, &self.buffer, CURSOR_SEMANTICS){
        if let Ok(new_selection) = selection::move_cursor_line_end(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_line_start(&mut self){
        //if let Ok(new_selection) = crate::utilities::move_cursor_home::selection_impl(&self.selection, &self.buffer, CURSOR_SEMANTICS){
        if let Ok(new_selection) = selection::move_cursor_home(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_right(&mut self){
        //if let Ok(new_selection) = crate::utilities::move_cursor_right::selection_impl(&self.selection, 1, &self.buffer, None, CURSOR_SEMANTICS){
        if let Ok(new_selection) = selection::move_cursor_right(&self.selection, 1, &self.buffer, None, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
}

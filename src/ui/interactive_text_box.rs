use crate::{
    range::Range,
    selection::{CursorSemantics, Movement, Selection, ExtensionDirection}, 
    view::View, 
};
use std::cmp::Ordering;



pub struct InteractiveTextBox{
    pub buffer: crate::buffer::Buffer,
    pub text_is_valid: bool,
    pub selection: Selection,
    pub view: View
}
impl Default for InteractiveTextBox{
    fn default() -> Self{
        let buffer = crate::buffer::Buffer::new("", None, false);
        Self{
            buffer: buffer.clone(),
            text_is_valid: false,
            selection: Selection::new_from_range(Range::new(0, 1), ExtensionDirection::None, &buffer, crate::config::CURSOR_SEMANTICS),//Selection::new(Range::new(0, 1), Direction::Forward),
            view: View::new(0, 0, 0, 1)
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
        if self.selection.is_extended(){
            self.delete();
        }
        let text = self.buffer.clone();
        let mut new_text = text.clone();
        new_text.insert(self.selection.cursor(&text, CursorSemantics::Block), &char.to_string());
        self.buffer = new_text;
        if let Ok(new_selection) = crate::utilities::move_cursor_right::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn delete(&mut self){
        let text = self.buffer.clone();
        let mut new_text = self.buffer.clone();
    
        match self.selection.cursor(&text, CursorSemantics::Block).cmp(&self.selection.anchor()){
            Ordering::Less => {
                new_text.remove(self.selection.head()..self.selection.anchor());
                if let Ok(new_selection) = self.selection.put_cursor(self.selection.cursor(&text, CursorSemantics::Block), &text, Movement::Move, CursorSemantics::Block, true){
                    self.selection = new_selection;
                }
            }
            Ordering::Greater => {
                if self.selection.cursor(&text, CursorSemantics::Block) == text.len_chars(){
                    new_text.remove(self.selection.anchor()..self.selection.cursor(&text, CursorSemantics::Block));
                }
                else{
                    new_text.remove(self.selection.anchor()..self.selection.head());
                }
                if let Ok(new_selection) = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true){
                    self.selection = new_selection;
                }
            }
            Ordering::Equal => {
                if self.selection.cursor(&text, CursorSemantics::Block) == text.len_chars(){}    //do nothing
                else{
                    new_text.remove(self.selection.anchor()..self.selection.head());
                    if let Ok(new_selection) = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true){
                        self.selection = new_selection;
                    }
                }
            }
        }
    
        self.buffer = new_text;
    }
    #[allow(clippy::collapsible_else_if)]
    pub fn backspace(&mut self){
        let semantics = CursorSemantics::Block;
        if self.selection.is_extended(){
            self.delete();
        }else{
            if self.selection.cursor(&self.buffer, semantics) > 0{
                if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
                    self.selection = new_selection;
                }
                self.delete();
            }
        }
    }



    pub fn extend_selection_end(&mut self){
        if let Ok(new_selection) = crate::utilities::extend_selection_line_end::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_home(&mut self){
        if let Ok(new_selection) = crate::utilities::extend_selection_home::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_left(&mut self){
        if let Ok(new_selection) = crate::utilities::extend_selection_left::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_right(&mut self){
        if let Ok(new_selection) = crate::utilities::extend_selection_right::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_left(&mut self){
        if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_line_end(&mut self){
        if let Ok(new_selection) = crate::utilities::move_cursor_line_end::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_line_start(&mut self){
        if let Ok(new_selection) = crate::utilities::move_cursor_home::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_right(&mut self){
        if let Ok(new_selection) = crate::utilities::move_cursor_right::selection_impl(&self.selection, &self.buffer, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
}

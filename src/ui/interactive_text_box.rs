use edit_core::{
    selection::{CursorSemantics, Movement, Selection}, 
    view::View, 
};
use ropey::Rope;
use std::cmp::Ordering;



pub struct InteractiveTextBox{
    pub text: Rope,
    pub text_is_valid: bool,
    selection: Selection,
    pub view: View
}
impl Default for InteractiveTextBox{
    fn default() -> Self{
        Self{
            text: Rope::from(""),
            text_is_valid: false,
            selection: Selection::new(0, 1),
            view: View::new(0, 0, 0, 1)
        }
    }
}
impl InteractiveTextBox{
    pub fn selection(&self) -> &Selection{
        &self.selection
    }
    pub fn selection_mut(&mut self) -> &mut Selection{
        &mut self.selection
    }
    pub fn view(&self) -> &View{
        &self.view
    }
    pub fn view_mut(&mut self) -> &mut View{
        &mut self.view
    }
    pub fn text(&self) -> &Rope{
        &self.text
    }
    pub fn cursor_position(&self) -> u16{
        self.selection.cursor(CursorSemantics::Block) as u16
    }
    pub fn clear(&mut self){
        *self = Self::default();
    }
    pub fn insert_char(&mut self, char: char){
        if self.selection.is_extended(CursorSemantics::Block){
            self.delete();
        }
        let text = self.text.clone();
        let mut new_text = text.clone();
        new_text.insert_char(self.selection.cursor(CursorSemantics::Block), char);
        self.text = new_text;
        self.selection = self.selection.move_right(&self.text.clone(), CursorSemantics::Block);
    }
    pub fn delete(&mut self){
        let text = self.text.clone();
        let mut new_text = self.text.clone();

        match self.selection.cursor(CursorSemantics::Block).cmp(&self.selection.anchor()){
            Ordering::Less => {
                new_text.remove(self.selection.head()..self.selection.anchor());
                self.selection = self.selection.put_cursor(self.selection.cursor(CursorSemantics::Block), &text, Movement::Move, CursorSemantics::Block, true);
            }
            Ordering::Greater => {
                if self.selection.cursor(CursorSemantics::Block) == text.len_chars(){
                    new_text.remove(self.selection.anchor()..self.selection.cursor(CursorSemantics::Block));
                }
                else{
                    new_text.remove(self.selection.anchor()..self.selection.head());
                }
                self.selection = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true);
            }
            Ordering::Equal => {
                if self.selection.cursor(CursorSemantics::Block) == text.len_chars(){}    //do nothing
                else{
                    new_text.remove(self.selection.anchor()..self.selection.head());
                    self.selection = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true);
                }
            }
        }

        self.text = new_text;
    }
    #[allow(clippy::collapsible_else_if)]
    pub fn backspace(&mut self){
        let semantics = CursorSemantics::Block;
        if self.selection.is_extended(semantics){
            self.delete();
        }else{
            if self.selection.cursor(semantics) > 0{
                self.selection = self.selection.move_left(&self.text, semantics);
                self.delete();
            }
        }
    }
}

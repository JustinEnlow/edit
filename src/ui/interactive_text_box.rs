use edit_core::{
    selection::{CursorSemantics, Movement, Selection}, 
    view::View, 
};
use ropey::Rope;
use std::cmp::Ordering;



pub struct InteractiveTextBox{
    pub text: Rope,
    pub text_is_valid: bool,
    pub selection: Selection,
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
    pub fn cursor_position(&self) -> u16{
        self.selection.cursor(&self.text, CursorSemantics::Block) as u16
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
        new_text.insert_char(self.selection.cursor(&text, CursorSemantics::Block), char);
        self.text = new_text;
        //self.selection = self.selection.move_right(&self.text.clone(), CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.move_right(&self.text.clone(), CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn delete(&mut self){
        let text = self.text.clone();
        let mut new_text = self.text.clone();

        match self.selection.cursor(&text, CursorSemantics::Block).cmp(&self.selection.anchor()){
            Ordering::Less => {
                new_text.remove(self.selection.head()..self.selection.anchor());
                //self.selection = self.selection.put_cursor(self.selection.cursor(CursorSemantics::Block), &text, Movement::Move, CursorSemantics::Block, true);
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
                //self.selection = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true);
                if let Ok(new_selection) = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true){
                    self.selection = new_selection;
                }
            }
            Ordering::Equal => {
                if self.selection.cursor(&text, CursorSemantics::Block) == text.len_chars(){}    //do nothing
                else{
                    new_text.remove(self.selection.anchor()..self.selection.head());
                    //self.selection = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true);
                    if let Ok(new_selection) = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true){
                        self.selection = new_selection;
                    }
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
            if self.selection.cursor(&self.text, semantics) > 0{
                //self.selection = self.selection.move_left(&self.text, semantics);
                if let Ok(new_selection) = self.selection.move_left(&self.text, semantics){
                    self.selection = new_selection;
                }
                self.delete();
            }
        }
    }



    pub fn extend_selection_end(&mut self){
        //self.selection = self.selection.extend_line_text_end(&self.text, CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.extend_line_text_end(&self.text, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_home(&mut self){
        //self.selection = self.selection.extend_home(&self.text, CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.extend_home(&self.text, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_left(&mut self){
        //self.selection = self.selection.extend_left(&self.text, CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.extend_left(&self.text, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn extend_selection_right(&mut self){
        //self.selection = self.selection.extend_right(&self.text, CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.extend_right(&self.text, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_left(&mut self){
        //self.selection = self.selection.move_left(&self.text, CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.move_left(&self.text, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_line_end(&mut self){
        //self.selection = self.selection.move_line_text_end(&self.text, CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.move_line_text_end(&self.text, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_line_start(&mut self){
        //self.selection = self.selection.move_home(&self.text, CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.move_home(&self.text, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
    pub fn move_cursor_right(&mut self){
        //self.selection = self.selection.move_right(&self.text, CursorSemantics::Block);
        if let Ok(new_selection) = self.selection.move_right(&self.text, CursorSemantics::Block){
            self.selection = new_selection;
        }
    }
}

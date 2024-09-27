use ropey::Rope;
use crate::selection::{Selections, Selection2d};
use crate::Position;



/// The dimensions of the area a client has for displaying a document
/// origin is top left
#[derive(Debug, Default, Clone)]
pub struct View{
    /// from left to right
    horizontal_start: usize,
    /// from top to bottom
    vertical_start: usize,
    width: usize,
    height: usize,
}
impl View{
    pub fn new(horizontal_start: usize, vertical_start: usize, width: usize, height: usize) -> Self{
        Self{horizontal_start, vertical_start, width, height}
    }
    pub fn set_size(&mut self, width: usize, height: usize){
        self.width = width;
        self.height = height;
    }
    pub fn horizontal_start(&self) -> usize{
        self.horizontal_start
    }
    pub fn set_horizontal_start(&mut self, horizontal_start: usize){
        self.horizontal_start = horizontal_start;
    }
    pub fn vertical_start(&self) -> usize{
        self.vertical_start
    }
    pub fn set_vertical_start(&mut self, vertical_start: usize){
        self.vertical_start = vertical_start;
    }
    pub fn width(&self) -> usize{
        self.width
    }
    pub fn set_width(&mut self, width: usize){
        self.width = width;
    }
    pub fn height(&self) -> usize{
        self.height
    }
    pub fn set_height(&mut self, height: usize){
        self.height = height;
    }

    pub fn scroll_down(&mut self, amount: usize, text: &Rope){
        if self.vertical_start() + self.height() + amount <= text.len_lines(){
            self.vertical_start = self.vertical_start.saturating_add(amount);
        }
    }
    pub fn scroll_left(&mut self, amount: usize){
        self.horizontal_start = self.horizontal_start.saturating_sub(amount);
    }
    pub fn scroll_right(&mut self, amount: usize, text: &Rope){
        let mut longest = 0;
        for line in text.lines(){
            let line_width = crate::text_util::line_width_excluding_newline(line);

            if line_width > longest{
                longest = line_width;
            }
        }

        if self.horizontal_start() + self.width() + amount <= longest{
            self.horizontal_start = self.horizontal_start.saturating_add(amount);
        }
    }
    pub fn scroll_up(&mut self, amount: usize){
        self.vertical_start = self.vertical_start.saturating_sub(amount);
    }

    pub fn scroll_following_cursor(&mut self, selections: &Selections, text: &Rope) -> bool{
        // following last cursor pushed to cursors vec
        let cursor = selections.last().clone().selection_to_selection2d(text);

        let mut should_update_client_view = false;

        if cursor.head().y() < self.vertical_start(){
            self.vertical_start = cursor.head().y();
            should_update_client_view = true;
        }
        else if cursor.head().y() >= self.vertical_start().saturating_add(self.height()){
            self.vertical_start = cursor.head().y().saturating_sub(self.height).saturating_add(1);
            should_update_client_view = true;
        }
    
        if cursor.head().x() < self.horizontal_start(){
            self.horizontal_start = cursor.head().x();
            should_update_client_view = true;
        }
        else if cursor.head().x() >= self.horizontal_start().saturating_add(self.width()){
            self.horizontal_start = cursor.head().x().saturating_sub(self.width).saturating_add(1);
            should_update_client_view = true;
        }

        should_update_client_view
    }

    pub fn text(&self, text: &Rope) -> String{
        let mut client_view_text = String::new();
        for (y, line) in text.lines().enumerate(){
            let mut bounded_line = String::new();
            if y >= self.vertical_start()
            && y <= (self.height().saturating_sub(1) + self.vertical_start()){
                for (x, char) in line.chars().enumerate(){
                    if x >= self.horizontal_start()
                    && x <= (self.width().saturating_sub(1) + self.horizontal_start())
                    && char != '\n'{
                        bounded_line.push(char);
                    }
                }
                client_view_text.push_str(format!("{}\n", bounded_line).as_str());
            }
        }

        client_view_text
    }
    pub fn line_numbers(&self, text: &Rope) -> String{
        let mut client_view_line_numbers = String::new();
        for (y, _) in text.lines().enumerate(){
            if y >= self.vertical_start()
            && y <= (self.height().saturating_sub(1) + self.vertical_start()){
                client_view_line_numbers.push_str(&format!("{}\n", y.saturating_add(1)));
            }
        }

        client_view_line_numbers
    }
    pub fn cursor_positions(&self, text: &Rope, selections: &Selections) -> Vec<Position>{
        let mut positions = Vec::new();
        for cursor in selections.iter(){
            if let Some(client_cursor) = Self::cursor_position(
                cursor.selection_to_selection2d(text),
                self.clone()
            ){
                positions.push(client_cursor);
            }
        }
        positions
    }
    // translates a document cursor position to a client view cursor position. if outside client view, returns None
    fn cursor_position(doc_cursor: Selection2d, client_view: View) -> Option<Position>{
        if doc_cursor.head().x() >= client_view.horizontal_start()
        && doc_cursor.head().x() < client_view.horizontal_start().saturating_add(client_view.width())
        && doc_cursor.head().y() >= client_view.vertical_start()
        && doc_cursor.head().y() < client_view.vertical_start().saturating_add(client_view.height()){
            Some(Position::new(
                doc_cursor.head().x().saturating_sub(client_view.horizontal_start()),
                doc_cursor.head().y().saturating_sub(client_view.vertical_start())
            ))
        }else{None}
    }
}


//TODO: implement tests for [View] behavior
//scroll down
//scroll left
//scroll right
//scroll up
//scroll following cursor
//etc.

//set client view size (does this need testing?)
//get client view text
    //#[test]
    //fn get_client_view_text_works(){
    //    let mut doc = Document::default();
    //    doc.text = Rope::from("idk\nsomething\nelse\n");
    //    doc.view_mut().set_size(2, 2);
    //    println!("{:?}", doc.get_client_view_text());
    //    assert!(doc.get_client_view_text() == String::from("id\nso\n"));
    //}
//get client view line numbers
//get client cursor positions

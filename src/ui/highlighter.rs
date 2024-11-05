use edit_core::{
    selection::Selection2d, 
    Position
};
use ratatui::layout::Rect;
use ratatui::style::{Style, Color};



// render order matters. for example, always render cursors after selections, so that the cursor shows on top of the selection.
#[derive(Default, Debug, Clone)]
pub struct Highlighter{
    pub rect: Rect,
    //cursor_line: Option<u16>,   //bg color
    //cursor_column: Option<u16>, //bg color
    // debug highlights //bg color
    // lsp highlights   //fg color
    pub selections: Option<Vec<Selection2d>>,   //bg color
    pub cursors: Option<Position>,//Option<Vec<Position>>, //bg color + fg color?
    // others idk
}
impl Highlighter{
    // TODO: can this be done by caller?
    pub fn set_client_cursor_position(&mut self, positions: Vec<Position>){
        if !positions.is_empty(){
            self.cursors = Some(*positions.last().unwrap());
        }else{
            self.cursors = None;
        }
    }
}
impl ratatui::widgets::Widget for Highlighter{
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer){
        if let Some(selections) = self.selections{
            for selection in selections.iter(){
                if selection.head().x() - selection.anchor().x() == 0{continue;}
                for col in selection.anchor().x()../*=*/selection.head().x(){
                    let x_pos = area.left() + (col as u16);
                    let y_pos = selection.head().y() as u16;
        
                    if let Some(cell) = buf.cell_mut((x_pos, y_pos)){
                        cell.set_style(Style::default()
                            .bg(Color::Blue)
                            .fg(Color::Black)
                        );
                    }
                }
            }
        }

        if let Some(cursor) = self.cursors{
            if let Some(cell) = buf.cell_mut((area.left() + (cursor.x() as u16), area.top() + (cursor.y() as u16))){
                cell.set_style(Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                );
            }
        }
    }
}

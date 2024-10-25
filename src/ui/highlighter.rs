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
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if let Some(selections) = self.selections{
            for selection in selections{
                for i in selection.anchor().x()..=selection.head().x(){
                    buf.get_mut(area.left() + (i as u16), area.top() + (selection.anchor().y() as u16))
                        .set_style(Style::default().bg(Color::Blue));
                }
            }
        }
    }
}

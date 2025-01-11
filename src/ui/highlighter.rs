use edit_core::{
    selection::Selection2d, 
    Position
};
use ratatui::layout::Rect;
use ratatui::style::Style;
use crate::config::{SELECTION_BACKGROUND_COLOR, SELECTION_FOREGROUND_COLOR, CURSOR_BACKGROUND_COLOR, CURSOR_FOREGROUND_COLOR};


/*TODO: separate Highlighter struct members into their own widget components, so they can be rendered separately in ui.rs       //or maybe each ui component should have its own highlighting sub component...
    pub struct DocumentSelections{  //maybe syntax highlighting/error highlighting belong here too, idk...
        pub rect: Rect,
        pub selections: Option<Vec<Selection2d>>,
        pub cursors: Option<Position>, or Option<Vec<Position>>
    }
    impl DocumentSelections{
        fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer){}
    }

    pub struct UtilBarSelections{}
    pub struct SyntaxHighlighting{}

*/


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
    //pub fn set_client_cursor_position(&mut self, positions: Vec<Position>){
    //    if !positions.is_empty(){
    //        self.cursors = Some(*positions.last().unwrap());
    //    }else{
    //        self.cursors = None;
    //    }
    //}
    pub fn set_primary_cursor_position(&mut self, position: Option<Position>){
        self.cursors = position;
    }
}
impl ratatui::widgets::Widget for Highlighter{
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer){
        if let Some(selections) = self.selections{  //selection not rendering properly on last empty line following previous newline, when cursor rendering below is not drawn there. maybe this is correct, because there is technically no content there...
            for selection in selections.iter(){
                if selection.head().x() - selection.anchor().x() == 0{continue;}    //should this use start and end instead?
                for col in selection.anchor().x()../*=*/selection.head().x(){
                    let x_pos = area.left() + (col as u16);
                    let y_pos = selection.head().y() as u16;
        
                    if let Some(cell) = buf.cell_mut((x_pos, y_pos)){
                        cell.set_style(Style::default()
                            .bg(SELECTION_BACKGROUND_COLOR)
                            .fg(SELECTION_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }

        if let Some(cursor) = self.cursors{
            if let Some(cell) = buf.cell_mut((area.left() + (cursor.x() as u16), area.top() + (cursor.y() as u16))){
                cell.set_style(Style::default()
                    .bg(CURSOR_BACKGROUND_COLOR)
                    .fg(CURSOR_FOREGROUND_COLOR)
                );
            }
        }
    }
}

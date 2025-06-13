use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::Style;
use ratatui::layout::{Alignment, Direction, Layout, Constraint};
use crate::config::{LINE_NUMBER_BACKGROUNG_COLOR, LINE_NUMBER_FOREGROUNG_COLOR, DOCUMENT_BACKGROUND_COLOR, DOCUMENT_FOREGROUND_COLOR};
use crate::position::Position;
use crate::selection2d::Selection2d;
use crate::config::{SELECTION_BACKGROUND_COLOR, SELECTION_FOREGROUND_COLOR, PRIMARY_CURSOR_BACKGROUND_COLOR, PRIMARY_CURSOR_FOREGROUND_COLOR, CURSOR_BACKGROUND_COLOR, CURSOR_FOREGROUND_COLOR};



#[derive(Default)]
pub struct LineNumberWidget{
    pub rect: Rect,
    pub line_numbers_in_view: String,
}
impl LineNumberWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.line_numbers_in_view.clone())
            .style(
                Style::default()
                    .bg(LINE_NUMBER_BACKGROUNG_COLOR)
                    .fg(LINE_NUMBER_FOREGROUNG_COLOR)
            )
            .alignment(Alignment::Right)
    }
}

#[derive(Default, Clone)]
pub struct DocumentWidget{
    pub rect: Rect,
    pub doc_length: usize,  //used in DocumentViewport
    pub text_in_view: String,
}
impl DocumentWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.text_in_view.clone())
            .style(
                Style::default()
                    .bg(DOCUMENT_BACKGROUND_COLOR)
                    .fg(DOCUMENT_FOREGROUND_COLOR)
            )
    }
}

// render order matters. for example, always render cursors after selections, so that the cursor shows on top of the selection.
#[derive(Default, Clone)]
pub struct Highlighter{
    //cursor_line: Option<u16>,   //bg color
    //cursor_column: Option<u16>, //bg color
    // debug highlights //bg color
    // lsp highlights   //fg color
    pub selections: Vec<Selection2d>,   //bg color
    pub primary_cursor: Option<Position>, //bg color + fg color?
    pub cursors: Option<Vec<Position>>, //should this really be option? i feel like we could accomplish the same with an empty vec...
    // others idk
}
impl Highlighter{
    pub fn set_client_cursor_positions(&mut self, positions: Vec<Position>){
        if positions.is_empty(){
            self.cursors = None;
        }

        self.cursors = Some(positions);
    }
    pub fn set_primary_cursor_position(&mut self, position: Option<Position>){
        self.primary_cursor = position;
    }
}
impl ratatui::widgets::Widget for Highlighter{
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer){
        //if let Some(selections) = self.selections{  //selection not rendering properly on last empty line following previous newline, when cursor rendering below is not drawn there. maybe this is correct, because there is technically no content there...
        if !self.selections.is_empty(){
            for selection in &self.selections{  //self.selections.iter(){   //change suggested by clippy lint
                if selection.head().x - selection.anchor().x == 0{continue;}    //should this use start and end instead?
                for col in selection.anchor().x../*=*/selection.head().x{
                    let x_pos = area.left() + (col as u16);
                    let y_pos = selection.head().y as u16;
        
                    if let Some(cell) = buf.cell_mut((x_pos, y_pos)){
                        cell.set_style(Style::default()
                            .bg(SELECTION_BACKGROUND_COLOR)
                            .fg(SELECTION_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }

        //render cursors for all selections
        if let Some(cursors) = self.cursors{
            for cursor in cursors{
                if let Some(cell) = buf.cell_mut((area.left() + (cursor.x as u16), area.top() + (cursor.y as u16))){
                    cell.set_style(Style::default()
                        .bg(CURSOR_BACKGROUND_COLOR)
                        .fg(CURSOR_FOREGROUND_COLOR)
                    );
                }
            }
        }

        // render primary cursor
        if let Some(cursor) = self.primary_cursor{
            if let Some(cell) = buf.cell_mut((area.left() + (cursor.x as u16), area.top() + (cursor.y as u16))){
                cell.set_style(Style::default()
                    .bg(PRIMARY_CURSOR_BACKGROUND_COLOR)
                    .fg(PRIMARY_CURSOR_FOREGROUND_COLOR)
                );
            }
        }
    }
}

/// Container type for widgets in the document viewport.
pub struct DocumentViewport{
    pub display_line_numbers: bool,
    pub document_widget: DocumentWidget,
    pub line_number_widget: LineNumberWidget,
    pub highlighter: Highlighter,
}
impl Default for DocumentViewport{
    fn default() -> Self{
        Self{
            display_line_numbers: true,
            document_widget: DocumentWidget::default(),
            line_number_widget: LineNumberWidget::default(),
            highlighter: Highlighter::default(),
        }
    }
}
impl DocumentViewport{
    pub fn toggle_line_numbers(&mut self){
        self.display_line_numbers = !self.display_line_numbers;
    }
    pub fn layout(&self, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of document + line num rect
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // line number left padding
                    //Constraint::Length(if self.display_line_numbers{1}else{0}),
                    // line number rect width
                    Constraint::Length(
                        if self.display_line_numbers{
                            count_digits(self.document_widget.doc_length)
                        }else{0}
                    ),
                    // line number right padding
                    Constraint::Length(if self.display_line_numbers{1}else{0}),
                    // document rect width
                    Constraint::Min(5)
                ]
            )
            .split(rect)
    }
}

fn count_digits(mut n: usize) -> u16{
    if n == 0{
        return 1;
    }

    let mut count = 0;
    while n > 0{
        count += 1;
        n /= 10;
    }

    count
}

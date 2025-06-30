use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::Style;
use ratatui::layout::{Alignment, Direction, Layout, Constraint};
use crate::config::{LINE_NUMBER_BACKGROUNG_COLOR, LINE_NUMBER_FOREGROUNG_COLOR, DOCUMENT_BACKGROUND_COLOR, DOCUMENT_FOREGROUND_COLOR};
use crate::position::Position;
use crate::selection2d::Selection2d;
use crate::config::{SELECTION_BACKGROUND_COLOR, SELECTION_FOREGROUND_COLOR, PRIMARY_CURSOR_BACKGROUND_COLOR, PRIMARY_CURSOR_FOREGROUND_COLOR, CURSOR_BACKGROUND_COLOR, CURSOR_FOREGROUND_COLOR};



/// This is used to fill space between other widgets
#[derive(Default)]
pub struct Padding{
    pub rect: Rect,
}
impl Padding{
    pub fn widget(&self) ->Paragraph<'static>{
        Paragraph::new(String::new())
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .bg(LINE_NUMBER_BACKGROUNG_COLOR)
                    .fg(LINE_NUMBER_BACKGROUNG_COLOR)
            )
    }
}

#[derive(Default)]
pub struct LineNumberWidget{
    pub rect: Rect,
    //pub line_numbers_in_view: String,
    pub text: String,
    pub show: bool,
}
impl LineNumberWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        //Paragraph::new(self.line_numbers_in_view.clone())
        Paragraph::new(self.text.clone())
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
    pub doc_length: usize,  //used in DocumentViewport  //TODO: can this be set elsewhere?...maybe pass in to DocumentViewport::layout()?...
    pub text: String,
}
impl DocumentWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.text.clone())
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
    // debug highlights //bg color
    // lsp highlights   //fg color
    pub selections: Vec<Selection2d>,   //bg color
    pub primary_cursor: Option<Position>, //bg color + fg color?
    pub cursors: Vec<Position>, 
    // others idk
}
impl ratatui::widgets::Widget for Highlighter{
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer){
        if crate::config::SHOW_CURSOR_COLUMN{
            for y in area.top()..area.height{
                if let Some(primary_cursor_position) = self.primary_cursor.clone(){
                    if let Some(cell) = buf.cell_mut((area.left() + primary_cursor_position.x as u16, y)){
                        cell.set_style(
                            Style::default()
                                .bg(crate::config::CURSOR_COLUMN_BACKGROUND_COLOR)
                                .fg(crate::config::CURSOR_COLUMN_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }
        if crate::config::SHOW_CURSOR_LINE{
            for x in area.left()..(area.width + area.left()){
                if let Some(primary_cursor_position) = self.primary_cursor.clone(){
                    if let Some(cell) = buf.cell_mut((x, area.top() + primary_cursor_position.y as u16)){
                        cell.set_style(
                            Style::default()
                                .bg(crate::config::CURSOR_LINE_BACKGROUND_COLOR)
                                .fg(crate::config::CURSOR_LINE_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }

        //if let Some(selections) = self.selections{  //selection not rendering properly on last empty line following previous newline, when cursor rendering below is not drawn there. maybe this is correct, because there is technically no content there...
        if !self.selections.is_empty(){
            for selection in &self.selections{  //self.selections.iter(){   //change suggested by clippy lint
                if selection.head().x - selection.anchor().x == 0{continue;}    //should this use start and end instead?
                for col in selection.anchor().x../*=*/selection.head().x{
                    let x_pos = area.left() + (col as u16);
                    //let y_pos = selection.head().y as u16;
                    let y_pos = area.top() + (selection.head().y as u16);
        
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
        if !self.cursors.is_empty(){
            for cursor in self.cursors{
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

        //debug //this can help ensure we are using the correct Rect
        //if let Some(cell) = buf.cell_mut((area.left(), area.top())){
        //    cell.set_style(
        //        Style::default()
        //            .bg(ratatui::style::Color::Yellow)
        //    );
        //}
    }
}

/// Container type for widgets in the document viewport.
pub struct DocumentViewport{
    //pub display_line_numbers: bool,
    pub line_number_widget: LineNumberWidget,
    pub padding: Padding,
    pub document_widget: DocumentWidget,
    pub highlighter: Highlighter,
}
impl Default for DocumentViewport{
    fn default() -> Self{
        Self{
            //display_line_numbers: true,
            line_number_widget: LineNumberWidget::default(),
            padding: Padding::default(),
            document_widget: DocumentWidget::default(),
            highlighter: Highlighter::default(),
        }
    }
}
impl DocumentViewport{
    //pub fn toggle_line_numbers(&mut self){
    //    self.display_line_numbers = !self.display_line_numbers;
    //}
    pub fn layout(&self, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of document + line num rect
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    //[0]
                    // line number rect width
                    Constraint::Length(
                        //if self.display_line_numbers{
                        if self.line_number_widget.show{
                            count_digits(self.document_widget.doc_length)
                        }else{0}
                    ),

                    //[1]
                    // line number right padding
                    Constraint::Length(
                        //if self.display_line_numbers{
                        if self.line_number_widget.show{
                            1
                        }else{0}
                    ),

                    //[2]
                    // document rect width
                    Constraint::Min(5)
                ]
            )
            .split(rect)
    }
}

pub fn count_digits(mut n: usize) -> u16{
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

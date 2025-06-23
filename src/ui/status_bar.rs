use crate::position::Position;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Stylize};
use ratatui::layout::{Alignment, Direction, Layout, Constraint};
use crate::config::{STATUS_BAR_BACKGROUND_COLOR, STATUS_BAR_FOREGROUND_COLOR};



const MODIFIED_INDICATOR: &str = "[Modified]";



#[derive(Default)]
pub struct ModeWidget{
    pub rect: Rect,
    pub text: String,
}
impl ModeWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.text.clone())
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(STATUS_BAR_FOREGROUND_COLOR)
                    .bold()
            )
    }
}
#[derive(Default)]
pub struct SelectionsWidget{
    pub rect: Rect,
    pub primary_selection_index: usize,
    pub num_selections: usize
}
impl SelectionsWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        let selections = format!("selections: {}/{}", self.primary_selection_index + 1, self.num_selections);
        Paragraph::new(selections)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(STATUS_BAR_FOREGROUND_COLOR)
                    .bold()
            )
    }
}

#[derive(Default)]
pub struct DocumentCursorPositionWidget{
    pub rect: Rect,
    pub document_cursor_position: Position,
}
impl DocumentCursorPositionWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        let position = format!("cursor: {}:{}", self.document_cursor_position.y + 1, self.document_cursor_position.x + 1);
        Paragraph::new(position)
            .alignment(Alignment::Right)
            .style(
                Style::default()
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(STATUS_BAR_FOREGROUND_COLOR)
                    .bold()
            )
    }
}



#[derive(Default)]
pub struct FileNameWidget{
    pub rect: Rect,
    pub file_name: Option<String>
}
impl FileNameWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        let file_name = match &self.file_name{
            Some(file_name) => file_name.to_string(),
            //None => "None".to_string()
            None => "[Scratch]".to_string()
        };
        Paragraph::new(file_name)
            .alignment(Alignment::Left)
            .style(
                Style::default()
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(STATUS_BAR_FOREGROUND_COLOR)
                    .bold()
            )
    }
}

#[derive(Default)]
pub struct ModifiedIndicatorWidget{
    pub rect: Rect,
    pub document_modified_status: bool
}
impl ModifiedIndicatorWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        //if self.document_modified_status{
            Paragraph::new(MODIFIED_INDICATOR)
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .bg(STATUS_BAR_BACKGROUND_COLOR)
                        .fg(STATUS_BAR_FOREGROUND_COLOR)
                        .bold()
                )
        //}else{
        //    //Paragraph::new("".repeat(MODIFIED_INDICATOR.len()))
        //    Paragraph::new(String::new())
        //    .style(
        //        Style::default()
        //            .bg(STATUS_BAR_BACKGROUND_COLOR)
        //            .fg(STATUS_BAR_FOREGROUND_COLOR)
        //            .bold()
        //    )
        //}
    }
}

/// Container type for widgets on the status bar.
pub struct StatusBar{
    pub display: bool,
    pub modified_indicator_widget: ModifiedIndicatorWidget,
    pub file_name_widget: FileNameWidget,
    pub mode_widget: ModeWidget,
    pub selections_widget: SelectionsWidget,
    pub document_cursor_position_widget: DocumentCursorPositionWidget,
}
impl Default for StatusBar{
    fn default() -> Self{
        Self{
            display: true,
            modified_indicator_widget: ModifiedIndicatorWidget::default(),
            file_name_widget: FileNameWidget::default(),
            mode_widget: ModeWidget::default(),
            selections_widget: SelectionsWidget::default(),
            document_cursor_position_widget: DocumentCursorPositionWidget::default()
        }
    }
}
impl StatusBar{
    pub fn toggle_status_bar(&mut self){
        self.display = !self.display;
    }
    pub fn layout(&self, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of status bar rect (modified_indicator/file_name/cursor_position)
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // file_name width
                    Constraint::Max(
                        if let Some(file_name) = &self.file_name_widget.file_name{
                            file_name.len() as u16
                        }else{0}
                    ),
                    // modified indicator width
                    Constraint::Max(
                        if self.modified_indicator_widget.document_modified_status{
                            MODIFIED_INDICATOR.len() as u16 + 2
                        }else{0}
                    ),
                    // mode widget
                    Constraint::Max(self.mode_widget.text.len() as u16 + 2),
                    // selections widget
                    Constraint::Min(0),
                    // cursor position indicator width
                    Constraint::Max(format!("cursor: {}:{}", self.document_cursor_position_widget.document_cursor_position.y + 1, self.document_cursor_position_widget.document_cursor_position.x + 1).len() as u16)
                ]
            )
            .split(rect)
    }
}

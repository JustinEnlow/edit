use edit_core::Position;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color, Stylize};
use ratatui::layout::{Alignment, Direction, Layout, Constraint};



const MODIFIED_INDICATOR: &str = "[Modified]";



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
                    .bg(Color::DarkGray)
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
        let position = format!("cursor: {}:{}", self.document_cursor_position.y() + 1, self.document_cursor_position.x() + 1);
        Paragraph::new(position)
            .alignment(Alignment::Right)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
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
            None => "None".to_string()
        };
        Paragraph::new(file_name)
            .alignment(Alignment::Left)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
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
        Paragraph::new(MODIFIED_INDICATOR)
            .alignment(Alignment::Left)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
                    .bold()
            )
    }
}

/// Container type for widgets on the status bar.
pub struct StatusBar{
    pub display: bool,
    pub modified_indicator_widget: ModifiedIndicatorWidget,
    pub file_name_widget: FileNameWidget,
    pub selections_widget: SelectionsWidget,
    pub document_cursor_position_widget: DocumentCursorPositionWidget,
}
impl Default for StatusBar{
    fn default() -> Self{
        Self{
            display: true,
            modified_indicator_widget: ModifiedIndicatorWidget::default(),
            file_name_widget: FileNameWidget::default(),
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
                    // modified indicator width
                    Constraint::Max(
                        if self.modified_indicator_widget.document_modified_status{
                            MODIFIED_INDICATOR.len() as u16
                        }else{0}
                    ),
                    // file_name width
                    Constraint::Max(
                        if let Some(file_name) = &self.file_name_widget.file_name{
                            file_name.len() as u16
                        }else{0}
                    ),
                        //TODO: add padding around selections widget
                    // selections widget
                    Constraint::Min(0),
                        //TODO: add padding around selections widget
                    // cursor position indicator width
                    //Constraint::Min(0)
                    Constraint::Max(format!("cursor: {}:{}", self.document_cursor_position_widget.document_cursor_position.y() + 1, self.document_cursor_position_widget.document_cursor_position.x() + 1).len() as u16)
                ]
            )
            .split(rect)
    }
}

use crate::position::Position;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Stylize};
use ratatui::layout::{Alignment, Direction, Layout, Constraint};
use crate::config::{
    STATUS_BAR_BACKGROUND_COLOR, 
    //STATUS_BAR_FOREGROUND_COLOR, 
    READ_ONLY_WIDGET_FOREGROUND_COLOR, 
    FILE_NAME_WIDGET_FOREGROUND_COLOR, 
    MODIFIED_WIDGET_FOREGROUND_COLOR,
    MODE_WIDGET_FOREGROUND_COLOR,
    SELECTIONS_WIDGET_FOREGROUND_COLOR,
    CURSOR_POSITION_WIDGET_FOREGROUNG_COLOR,
};



//TODO: all widget should have rect and text properties, with the text being set(possibly depending on app data) in application.rs
//and maybe a show property, that if false would set widget width to 0 for layout
//this maybe should not apply to highlighters...



const MODIFIED_INDICATOR: &str = "[Modified]";



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
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(STATUS_BAR_BACKGROUND_COLOR)
                    .bold()
            )
    }
}

#[derive(Default)]
pub struct ReadOnlyWidget{
    pub rect: Rect,
    pub text: String,
    pub show: bool,
}
impl ReadOnlyWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.text.clone())
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(READ_ONLY_WIDGET_FOREGROUND_COLOR)
                    .bold()
            )
    }
}

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
                    .fg(MODE_WIDGET_FOREGROUND_COLOR)
                    .bold()
            )
    }
}
#[derive(Default)]
pub struct SelectionsWidget{
    pub rect: Rect,
    //pub primary_selection_index: usize, //TODO: remove in favor of text
    //pub num_selections: usize           //TODO: remove in favor of text
    pub text: String,
}
impl SelectionsWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        //let selections = format!("selections: {}/{}", self.primary_selection_index + 1, self.num_selections);
        //Paragraph::new(selections)
        Paragraph::new(self.text.clone())
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(SELECTIONS_WIDGET_FOREGROUND_COLOR)
                    .bold()
            )
    }
}

#[derive(Default)]
pub struct CursorPositionWidget{
    pub rect: Rect,
    //pub document_cursor_position: Position, //TODO: remove in favor of text
    pub text: String,
}
impl CursorPositionWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        //let position = format!("cursor: {}:{}", self.document_cursor_position.y + 1, self.document_cursor_position.x + 1);
        //Paragraph::new(position)
        Paragraph::new(self.text.clone())
            .alignment(Alignment::Right)
            .style(
                Style::default()
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(CURSOR_POSITION_WIDGET_FOREGROUNG_COLOR)
                    .bold()
            )
    }
}



#[derive(Default)]
pub struct FileNameWidget{
    pub rect: Rect,
    //pub file_name: Option<String>   //TODO: remove in favor of text and show
    pub text: String,
    pub show: bool
}
impl FileNameWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        //let file_name = match &self.file_name{
        //    Some(file_name) => file_name.to_string(),
        //    //None => "None".to_string()
        //    None => "[Scratch]".to_string()
        //};
        //Paragraph::new(file_name)
        Paragraph::new(self.text.clone())
            .alignment(Alignment::Left)
            .style(
                Style::default()
                    .bg(STATUS_BAR_BACKGROUND_COLOR)
                    .fg(FILE_NAME_WIDGET_FOREGROUND_COLOR)
                    .bold()
            )
    }
}

#[derive(Default)]
pub struct ModifiedWidget{
    pub rect: Rect,
    //pub modified_status: bool,  //TODO: remove in favor of text and show
    pub text: String,
    pub show: bool,
}
impl ModifiedWidget{
    pub fn widget(&self) -> Paragraph<'static>{
        //if self.document_modified_status{
            //Paragraph::new(MODIFIED_INDICATOR)
            Paragraph::new(self.text.clone())
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .bg(STATUS_BAR_BACKGROUND_COLOR)
                        .fg(MODIFIED_WIDGET_FOREGROUND_COLOR)
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
    pub read_only_widget: ReadOnlyWidget,
    pub padding_1: Padding,
    pub file_name_widget: FileNameWidget,
    pub padding_2: Padding,
    pub modified_widget: ModifiedWidget,
    pub selections_widget: SelectionsWidget,
    pub cursor_position_widget: CursorPositionWidget,
    pub padding_3: Padding,
    pub mode_widget: ModeWidget,
}
impl Default for StatusBar{
    fn default() -> Self{
        Self{
            display: true,
            read_only_widget: ReadOnlyWidget::default(),
            padding_1: Padding::default(),
            file_name_widget: FileNameWidget::default(),
            padding_2: Padding::default(),
            modified_widget: ModifiedWidget::default(),
            selections_widget: SelectionsWidget::default(),
            cursor_position_widget: CursorPositionWidget::default(),
            padding_3: Padding::default(),
            mode_widget: ModeWidget::default(),
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
                    //[0]
                    // read_only widget
                    Constraint::Max(
                        if self.read_only_widget.show{
                            self.read_only_widget.text.len() as u16
                        }else{0}
                    ),

                    //[1]
                    // padding_1
                    Constraint::Max(
                        if self.read_only_widget.show{  //make padding dependent on previous widget
                            1
                        }else{0}
                    ),
                    
                    //[2]
                    // file_name widget
                    Constraint::Max(
                        //if let Some(file_name) = &self.file_name_widget.file_name{
                        //    file_name.len() as u16
                        //}else{0}
                        if self.file_name_widget.show{
                            self.file_name_widget.text.len() as u16
                        }else{0}
                    ),

                    //[3]
                    // padding_2
                    Constraint::Max(
                        if self.modified_widget.show{
                            1
                        }else{0}
                    ),
                    
                    //[4]
                    // modified widget
                    Constraint::Max(
                        //if self.modified_widget.modified_status{
                        //    MODIFIED_INDICATOR.len() as u16
                        //}else{0}
                        if self.modified_widget.show{
                            self.modified_widget.text.len() as u16
                        }else{0}
                    ),
                    
                    //[5]
                    // selections widget
                    Constraint::Min(0),     //or set selections widget to Max, and surround with 2 padding widgets set to Min(0)?...idk if that will work the same?...
                    
                    //[6]
                    // cursor position indicator width
                    Constraint::Max(
                        //format!("cursor: {}:{}", self.cursor_position_widget.document_cursor_position.y + 1, self.cursor_position_widget.document_cursor_position.x + 1).len() as u16
                        self.cursor_position_widget.text.len() as u16
                    ),

                    //[7]
                    // padding_3
                    Constraint::Max(1),

                    //[8]
                    // mode widget
                    Constraint::Max(self.mode_widget.text.len() as u16),
                ]
            )
            .split(rect)
    }
}

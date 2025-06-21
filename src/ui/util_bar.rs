use crate::ui::interactive_text_box::InteractiveTextBox;
use crate::application::Mode;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Stylize};
use ratatui::layout::{Direction, Layout, Constraint};
use crate::config::{UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR, ERROR_BACKGROUND_COLOR, ERROR_FOREGROUND_COLOR, WARNING_BACKGROUND_COLOR, WARNING_FOREGROUND_COLOR, NOTIFY_BACKGROUND_COLOR, NOTIFY_FOREGROUND_COLOR, INFO_BACKGROUND_COLOR, INFO_FOREGROUND_COLOR};
use crate::selections::Selections;
use crate::config::{SELECTION_BACKGROUND_COLOR, SELECTION_FOREGROUND_COLOR, PRIMARY_CURSOR_BACKGROUND_COLOR, PRIMARY_CURSOR_FOREGROUND_COLOR};



const GOTO_PROMPT: &str = " Go to: ";
const FIND_PROMPT: &str = " Find: ";
const SPLIT_PROMPT: &str = " Split: ";
const COMMAND_PROMPT: &str = " Command: ";



#[derive(Default)]
pub struct UtilityWidget{
    pub rect: Rect,
    pub text_box: InteractiveTextBox,
    pub preserved_selections: Option<Selections>,
}
impl UtilityWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Goto | Mode::Find | Mode::Split => {
                let buffer = &self.text_box.buffer;
                if self.text_box.text_is_valid{
                    Paragraph::new(self.text_box.view.text(buffer))
                        .style(
                            Style::default()
                            .bg(UTIL_BAR_BACKGROUND_COLOR)
                            .fg(UTIL_BAR_FOREGROUND_COLOR)
                        )
                }else{
                    Paragraph::new(self.text_box.view.text(buffer))
                        .style(
                            Style::default()
                                .fg(UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR)
                        )
                }
            }
            Mode::Command => {
                let buffer = &self.text_box.buffer;
                Paragraph::new(self.text_box.view.text(buffer))
            }
            Mode::Error(string) => {
                Paragraph::new(string)
                    .alignment(ratatui::prelude::Alignment::Center)
                    .style(
                        Style::default()
                            .bg(ERROR_BACKGROUND_COLOR)
                            .fg(ERROR_FOREGROUND_COLOR)
                            .bold()
                    )
            },
            Mode::Warning(string) => {
                Paragraph::new(string)
                    .alignment(ratatui::prelude::Alignment::Center)
                    .style(
                        Style::default()
                            .bg(WARNING_BACKGROUND_COLOR)
                            .fg(WARNING_FOREGROUND_COLOR)
                            .bold()
                    )
            }
            Mode::Notify(string) => {
                Paragraph::new(string)
                    .alignment(ratatui::prelude::Alignment::Center)
                    .style(
                        Style::default()
                            .bg(NOTIFY_BACKGROUND_COLOR)
                            .fg(NOTIFY_FOREGROUND_COLOR)
                            .bold()
                    )
            },
            Mode::Info(string) => {
                Paragraph::new(string)
                    .alignment(ratatui::prelude::Alignment::Center)
                    .style(
                        Style::default()
                            .bg(INFO_BACKGROUND_COLOR)
                            .fg(INFO_FOREGROUND_COLOR)
                            .bold()
                    )
            }
            Mode::Insert => {
                Paragraph::new(String::new())
            }
            Mode::View => Paragraph::new(String::new()),
            Mode::Object => Paragraph::new(String::new()),
            Mode::AddSurround => Paragraph::new(String::new())
        }
    }
}

#[derive(Default)]
pub struct UtilityPromptWidget{
    pub rect: Rect
}
impl UtilityPromptWidget{
    pub fn widget(&self, mode: &Mode) -> Paragraph<'static>{
        match mode{
            Mode::Goto => Paragraph::new(GOTO_PROMPT),
            Mode::Find => Paragraph::new(FIND_PROMPT),
            Mode::Split => Paragraph::new(SPLIT_PROMPT),
            Mode::Command => Paragraph::new(COMMAND_PROMPT),
            Mode::Insert |
            Mode::View |
            Mode::Error(_) |
            Mode::Warning(_) |
            Mode::Notify(_) |
            Mode::Info(_) |
            Mode::Object |
            Mode::AddSurround => Paragraph::new("")
        }
    }
}

#[derive(Default, Clone)]
pub struct Highlighter{
    pub selection: Option<crate::selection2d::Selection2d>, //util bar text should be guaranteed to be one line...
    pub cursor: Option<crate::position::Position>,
}
impl ratatui::widgets::Widget for Highlighter{
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer){
        //render selection
        if let Some(selection) = self.selection{
            if selection.head().x - selection.anchor().x > 0{   //if selection extended
                for col in selection.anchor().x..selection.head().x{
                    let x_pos = area.left() + (col as u16);
                    //let y_pos = area.top();
                    let y_pos = area.top() + (selection.head().y as u16);
                    assert_eq!(0, y_pos, "util bar text should be guaranteed to be one line");
        
                    if let Some(cell) = buf.cell_mut((x_pos, y_pos)){
                        cell.set_style(Style::default()
                            .bg(SELECTION_BACKGROUND_COLOR)
                            .fg(SELECTION_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }

        // render cursor
        if let Some(cursor) = self.cursor{
            assert_eq!(0, cursor.y, "util bar text should be guaranteed to be one line");
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

/// Container type for widgets on the util bar.
#[derive(Default)]
pub struct UtilBar{
    pub prompt: UtilityPromptWidget,
    pub utility_widget: UtilityWidget,
    pub highlighter: Highlighter,
}
impl UtilBar{
    pub fn update_width(&mut self, mode: &Mode){
        match mode{ //TODO: can these be set from relevant fns in application.rs? display_line_numbers, display_status_bar, resize, any mode change, etc
            Mode::Command 
            | Mode::Goto 
            | Mode::Find 
            | Mode::Split => {
                let width = self.utility_widget.rect.width as usize;
                self.utility_widget.text_box.view.set_size(width, 1);
            }
            Mode::Object |
            Mode::Insert |
            Mode::View |
            Mode::Error(_) |
            Mode::Warning(_) |
            Mode::Notify(_) |
            Mode::Info(_) |
            Mode::AddSurround => {
                self.utility_widget.text_box.view.set_size(0, 1);
            }
        }
    }
    pub fn layout(&self, mode: &Mode, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of util rect (goto/find/command/save as)
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // util bar prompt width
                    Constraint::Length(
                        match mode{
                            Mode::Goto => GOTO_PROMPT.len() as u16,
                            Mode::Find => FIND_PROMPT.len() as u16,
                            Mode::Split => SPLIT_PROMPT.len() as u16,
                            Mode::Command => COMMAND_PROMPT.len() as u16,
                            Mode::Error(_)
                            | Mode::Warning(_)
                            | Mode::Notify(_)
                            | Mode::Info(_)
                            | Mode::Insert
                            | Mode::Object
                            | Mode::View 
                            | Mode::AddSurround => 0
                        }
                    ),
                    // util bar rect width
                    Constraint::Length(
                        match mode{
                            Mode::Insert
                            | Mode::Object
                            | Mode::View
                            | Mode::Error(_) 
                            | Mode::Warning(_)
                            | Mode::Notify(_)
                            | Mode::Info(_)
                            | Mode::AddSurround => rect.width,
                            Mode::Goto => rect.width - GOTO_PROMPT.len() as u16,
                            Mode::Command => rect.width - COMMAND_PROMPT.len() as u16,
                            Mode::Find => rect.width - FIND_PROMPT.len() as u16,
                            Mode::Split => rect.width - SPLIT_PROMPT.len() as u16,
                        }
                    ),
                    // used to fill in space when other two are 0 length
                    Constraint::Length(0)
                ]
            )
            .split(rect)
    }
}

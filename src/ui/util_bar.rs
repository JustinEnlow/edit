use crate::ui::interactive_text_box::InteractiveTextBox;
use crate::application::{Mode, WarningKind};
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Stylize};
use ratatui::layout::{Direction, Layout, Constraint};
use crate::config::{UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR, WARNING_BACKGROUND_COLOR, WARNING_FOREGROUND_COLOR, COPIED_INDICATOR_BACKGROUND_COLOR, COPIED_INDICATOR_FOREGROUND_COLOR};
use crate::selections::Selections;
use crate::selection::Selection;
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
            Mode::Warning(kind) => {
                Paragraph::new(
                match kind{
                        WarningKind::FileIsModified => {
                            "WARNING! File has unsaved changes. Press close again to ignore and close.".to_string()
                        }
                        WarningKind::FileSaveFailed => {
                            "WARNING! File could not be saved.".to_string()
                        }
                        WarningKind::CommandParseFailed => {
                            "WARNING! Failed to parse command. Command may be undefined.".to_string()
                        }
                        WarningKind::SingleSelection => {
                            "WARNING! Requested action cannot be performed on single selection.".to_string()
                        }
                        WarningKind::MultipleSelections => {
                            "WARNING! Requested action cannot be performed on multiple selections.".to_string()
                        }
                        WarningKind::InvalidInput => {
                            "WARNING! Invalid input.".to_string()
                        }
                        WarningKind::SameState => {
                            "WARNING! Requested action results in the same state.".to_string()
                        }
                        WarningKind::UnhandledError(e) => {
                            e
                        }
                        WarningKind::UnhandledKeypress => {
                            "WARNING! Unbound key pressed.".to_string()
                        }
                        WarningKind::UnhandledEvent => {
                            "WARNING! Unhandled event occurred.".to_string()
                        }
                    }
                )
                    .alignment(ratatui::prelude::Alignment::Center)
                    .style(
                        Style::default()
                            .bg(WARNING_BACKGROUND_COLOR)
                            .fg(WARNING_FOREGROUND_COLOR)
                            .bold()
                    )
            },
            Mode::Notify => {
                Paragraph::new("Text copied to clipboard.")
                        .alignment(ratatui::prelude::Alignment::Center)
                        .style(
                            Style::default()
                                .bg(COPIED_INDICATOR_BACKGROUND_COLOR)
                                .fg(COPIED_INDICATOR_FOREGROUND_COLOR)
                                .bold()
                        )
            },
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
            Mode::Notify |
            Mode::Object |
            Mode::Warning(_) |
            Mode::AddSurround => Paragraph::new("")
        }
    }
}

#[derive(Default, Clone)]
pub struct Highlighter{
    pub selection: Option<Selection>, //util bar text should be guaranteed to be one line...    //Option used here only to satisfy Default derive...
    pub cursor: u16,
}
impl ratatui::widgets::Widget for Highlighter{
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer){    //TODO: need to fix selection/cursor rendering when text is larger than util_bar rect
        //render selection
        if let Some(selection) = self.selection{
            if selection.range.end - selection.range.start > 0{
                for col in selection.range.start..selection.range.end{
                    let x_pos = area.left() + (col as u16);
                    let y_pos = area.top();
        
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
        if let Some(cell) = buf.cell_mut((area.left() + self.cursor, area.top())){
            cell.set_style(Style::default()
                .bg(PRIMARY_CURSOR_BACKGROUND_COLOR)
                .fg(PRIMARY_CURSOR_FOREGROUND_COLOR)
            );
        }
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
            Mode::Notify |
            Mode::Warning(_) |
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
                            Mode::Warning(_)
                            | Mode::Notify
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
                            | Mode::Notify
                            | Mode::Warning(_) 
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

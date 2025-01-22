use crate::ui::interactive_text_box::InteractiveTextBox;
use crate::application::{Mode, WarningKind};
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Stylize};
use ratatui::layout::{Direction, Layout, Constraint};
use crate::config::{UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR, WARNING_BACKGROUND_COLOR, WARNING_FOREGROUND_COLOR, COPIED_INDICATOR_BACKGROUND_COLOR, COPIED_INDICATOR_FOREGROUND_COLOR};
use edit_core::selections::Selections;



const GOTO_PROMPT: &str = " Go to: ";
const FIND_PROMPT: &str = " Find: ";
const COMMAND_PROMPT: &str = " Command: ";



#[derive(Default)]
pub struct UtilityWidget{
    pub rect: Rect,
    pub text_box: InteractiveTextBox,
    pub display_copied_indicator: bool,
    pub clear_copied_indicator: bool,   // clear_copied_indicator exists because copied_indicator widget rendering needs to persist for an entire loop cycle(until next keypress)
    pub selections_before_search: Option<Selections>,
}
impl UtilityWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Goto | Mode::Find => {
                let text = self.text_box.text.clone();
                if self.text_box.text_is_valid{
                    Paragraph::new(self.text_box.view.text(&text))
                        .style(
                            Style::default()
                            .bg(UTIL_BAR_BACKGROUND_COLOR)
                            .fg(UTIL_BAR_FOREGROUND_COLOR)
                        )
                }else{
                    Paragraph::new(self.text_box.view.text(&text))
                        .style(
                            Style::default()
                                .fg(UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR)
                        )
                }
            }
            Mode::Command => {
                let text = self.text_box.text.clone();
                Paragraph::new(self.text_box.view.text(&text))
            }
            Mode::Warning(kind) => Paragraph::new(
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
                }
            )
                .alignment(ratatui::prelude::Alignment::Center)
                .style(
                    Style::default()
                        .bg(WARNING_BACKGROUND_COLOR)
                        .fg(WARNING_FOREGROUND_COLOR)
                        .bold()
                )
            ,
            Mode::Insert => {
                if self.display_copied_indicator{
                    Paragraph::new("Text copied to clipboard.")
                        .alignment(ratatui::prelude::Alignment::Center)
                        .style(
                            Style::default()
                                .bg(COPIED_INDICATOR_BACKGROUND_COLOR)
                                .fg(COPIED_INDICATOR_FOREGROUND_COLOR)
                                .bold()
                        )
                }else{
                    Paragraph::new("".to_string())
                }
            }
            _ => Paragraph::new("".to_string())
        }
    }
}

#[derive(Default)]
pub struct UtilityPromptWidget{
    pub rect: Rect
}
impl UtilityPromptWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Goto => Paragraph::new(GOTO_PROMPT),
            Mode::Find => Paragraph::new(FIND_PROMPT),
            Mode::Command => Paragraph::new(COMMAND_PROMPT),
            _ => Paragraph::new("")
        }
    }
}

/// Container type for widgets on the util bar.
#[derive(Default)]
pub struct UtilBar{
    pub prompt: UtilityPromptWidget,
    pub utility_widget: UtilityWidget,
}
impl UtilBar{
    pub fn update_width(&mut self, mode: Mode){
        match mode{ //TODO: can these be set from relevant fns in application.rs? display_line_numbers, display_status_bar, resize, any mode change, etc
            Mode::Command 
            | Mode::Goto 
            | Mode::Find => {
                let width = self.utility_widget.rect.width as usize;
                self.utility_widget.text_box.view.set_size(width, 1);
            }
            _ => {
                self.utility_widget.text_box.view.set_size(0, 1);
            }
        }
    }
    pub fn layout(&self, mode: Mode, rect: Rect) -> std::rc::Rc<[Rect]>{
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
                            Mode::Command => COMMAND_PROMPT.len() as u16,
                            Mode::Warning(_)
                            | Mode::Insert
                            | Mode::Space => 0
                        }
                    ),
                    // util bar rect width
                    Constraint::Length(
                        match mode{
                            Mode::Insert
                            | Mode::Space
                            | Mode::Warning(_) => rect.width,
                            Mode::Goto => rect.width - GOTO_PROMPT.len() as u16,
                            Mode::Command => rect.width - COMMAND_PROMPT.len() as u16,
                            Mode::Find => rect.width - FIND_PROMPT.len() as u16,
                        }
                    ),
                    // used to fill in space when other two are 0 length
                    Constraint::Length(0)
                ]
            )
            .split(rect)
    }
}

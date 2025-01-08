use crate::ui::interactive_text_box::InteractiveTextBox;
use crate::application::{Mode, WarningKind};
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Stylize};
use ratatui::layout::{Direction, Layout, Constraint};
use crate::config::{UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR, WARNING_BACKGROUND_COLOR, WARNING_FOREGROUND_COLOR, COPIED_INDICATOR_BACKGROUND_COLOR, COPIED_INDICATOR_FOREGROUND_COLOR};



const GOTO_PROMPT: &str = " Go to: ";
const FIND_PROMPT: &str = " Find: ";
const REPLACE_PROMPT: &str = " Replace: ";
const COMMAND_PROMPT: &str = " Command: ";



#[derive(Default)]
pub struct UtilityWidget{
    pub rect: Rect,
    pub text_box: InteractiveTextBox,
    pub display_copied_indicator: bool,
    pub clear_copied_indicator: bool,   // clear_copied_indicator exists because copied_indicator widget rendering needs to persist for an entire loop cycle(until next keypress)
}
impl UtilityWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Goto | Mode::FindReplace => {
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
                                //.fg(Color::Red)
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
                        "WARNING! File has unsaved changes. Press close again to ignore and close."
                    }
                    WarningKind::FileSaveFailed => {
                        "WARNING! File could not be saved."
                    }
                    WarningKind::CommandParseFailed => {
                        "WARNING! Failed to parse command. Command may be undefined."
                    }
                    WarningKind::SingleSelection => {
                        "WARNING! Requested action cannot be performed on single selection."
                    }
                    WarningKind::MultipleSelections => {
                        "WARNING! Requested action cannot be performed on multiple selections."
                    }
                    WarningKind::InvalidInput => {
                        "WARNING! Invalid input."
                    }
                    WarningKind::SameState => {
                        "WARNING! Requested action results in the same state."
                    }
                }
            )
                .alignment(ratatui::prelude::Alignment::Center)
                .style(
                    Style::default()
                        //.bg(Color::Red)
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
                                //.bg(Color::Green)
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
pub struct UtilityAlternateWidget{
    pub rect: Rect,
    pub text_box: InteractiveTextBox,
}
impl UtilityAlternateWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        let text = self.text_box.text.clone();
        match mode{
            Mode::FindReplace => {
                Paragraph::new(self.text_box.view.text(&text))
            }
            _ => Paragraph::new(self.text_box.view.text(&text))
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
            Mode::FindReplace => Paragraph::new(FIND_PROMPT),
            Mode::Command => Paragraph::new(COMMAND_PROMPT),
            _ => Paragraph::new("")
        }
    }
}

#[derive(Default)]
pub struct UtilityAlternatePromptWidget{
    pub rect: Rect
}
impl UtilityAlternatePromptWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::FindReplace => {
                Paragraph::new(REPLACE_PROMPT)
            },
            _ => Paragraph::new("")
        }
    }
}

/// Container type for widgets on the util bar.
#[derive(Default)]
pub struct UtilBar{
    pub alternate_focused: bool,
    pub prompt: UtilityPromptWidget,
    pub alternate_prompt: UtilityAlternatePromptWidget,
    pub utility_widget: UtilityWidget,
    pub alternate_utility_widget: UtilityAlternateWidget,
}
impl UtilBar{
    pub fn update_width(&mut self, mode: Mode){
        match mode{ //TODO: can these be set from relevant fns in application.rs? display_line_numbers, display_status_bar, resize, any mode change, etc
            Mode::Command 
            | Mode::Goto 
            | Mode::FindReplace => {
                let width = self.utility_widget.rect.width as usize;
                self.utility_widget.text_box.view.set_size(width, 1);
                let width = self.alternate_utility_widget.rect.width as usize;
                self.alternate_utility_widget.text_box.view.set_size(width, 1);
            }
            _ => {
                self.utility_widget.text_box.view.set_size(0, 1);
                self.alternate_utility_widget.text_box.view.set_size(0, 1);
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
                            Mode::FindReplace => FIND_PROMPT.len() as u16,
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
                            Mode::FindReplace => (rect.width / 2) - FIND_PROMPT.len() as u16,
                        }
                    ),
                    // util bar alternate prompt width
                    Constraint::Length(
                        match mode{
                            Mode::FindReplace => REPLACE_PROMPT.len() as u16,
                            _ => 0
                        }
                    ),
                    // util bar alternate rect width
                    Constraint::Length(
                        match mode{
                            Mode::FindReplace => (rect. width / 2).saturating_sub(REPLACE_PROMPT.len() as u16),
                            _ => 0
                        }
                    ),
                    // used to fill in space when other two are 0 length
                    Constraint::Length(0)
                ]
            )
            .split(rect)
    }
}

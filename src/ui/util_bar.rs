use crate::ui::interactive_text_box::InteractiveTextBox;
use crate::application::{Mode, UtilityKind, WarningKind};
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color, Stylize};
use ratatui::layout::{Direction, Layout, Constraint};



const GOTO_PROMPT: &str = " Go to: ";
const FIND_PROMPT: &str = " Find: ";
const REPLACE_PROMPT: &str = " Replace: ";
const COMMAND_PROMPT: &str = " Command: ";



#[derive(Default)]
pub struct UtilityWidget{
    pub rect: Rect,
    pub text_box: InteractiveTextBox,
}
impl UtilityWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Utility(UtilityKind::Goto) | Mode::Utility(UtilityKind::FindReplace) => {
                let text = self.text_box.text.clone();
                if self.text_box.text_is_valid{
                    Paragraph::new(self.text_box.view.text(&text))
                }else{
                    Paragraph::new(self.text_box.view.text(&text))
                        .style(Style::default().fg(Color::Red))
                }
            }
            Mode::Utility(UtilityKind::Command) => {
                let text = self.text_box.text.clone();
                Paragraph::new(self.text_box.view.text(&text))
            }
            Mode::Utility(UtilityKind::Warning(kind)) => Paragraph::new(
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
                }
            )
                .alignment(ratatui::prelude::Alignment::Center)
                .style(Style::default().bg(Color::Red).bold())
            ,
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
            Mode::Utility(UtilityKind::FindReplace) => {
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
            Mode::Utility(UtilityKind::Goto) => Paragraph::new(GOTO_PROMPT),
            Mode::Utility(UtilityKind::FindReplace) => Paragraph::new(FIND_PROMPT),
            Mode::Utility(UtilityKind::Command) => Paragraph::new(COMMAND_PROMPT),
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
            Mode::Utility(UtilityKind::FindReplace) => {
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
            Mode::Utility(UtilityKind::Command) 
            | Mode::Utility(UtilityKind::Goto) 
            | Mode::Utility(UtilityKind::FindReplace) => {
                let width = self.utility_widget.rect.width as usize;
                self.utility_widget.text_box.view_mut().set_size(width, 1);
                let width = self.alternate_utility_widget.rect.width as usize;
                self.alternate_utility_widget.text_box.view_mut().set_size(width, 1);
            }
            _ => {
                self.utility_widget.text_box.view_mut().set_size(0, 1);
                self.alternate_utility_widget.text_box.view_mut().set_size(0, 1);
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
                            Mode::Utility(UtilityKind::Goto) => GOTO_PROMPT.len() as u16,
                            Mode::Utility(UtilityKind::FindReplace) => FIND_PROMPT.len() as u16,
                            Mode::Utility(UtilityKind::Command) => COMMAND_PROMPT.len() as u16,
                            Mode::Utility(UtilityKind::Warning(_))
                            | Mode::Insert
                            | Mode::Space => 0
                        }
                    ),
                    // util bar rect width
                    Constraint::Length(
                        match mode{
                            Mode::Insert
                            | Mode::Space
                            | Mode::Utility(UtilityKind::Warning(_)) => rect.width,
                            Mode::Utility(UtilityKind::Goto) => rect.width - GOTO_PROMPT.len() as u16,
                            Mode::Utility(UtilityKind::Command) => rect.width - COMMAND_PROMPT.len() as u16,                            
                            Mode::Utility(UtilityKind::FindReplace) => (rect.width / 2) - FIND_PROMPT.len() as u16,
                        }
                    ),
                    // util bar alternate prompt width
                    Constraint::Length(
                        match mode{
                            Mode::Utility(UtilityKind::FindReplace) => REPLACE_PROMPT.len() as u16,
                            _ => 0
                        }
                    ),
                    // util bar alternate rect width
                    Constraint::Length(
                        match mode{
                            Mode::Utility(UtilityKind::FindReplace) => (rect. width / 2).saturating_sub(REPLACE_PROMPT.len() as u16),
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

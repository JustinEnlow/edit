use crate::ui::interactive_text_box::InteractiveTextBox;
use crate::selection2d::Selection2d;
use crate::position::Position;
use ratatui::layout::Rect;

pub const GOTO_PROMPT: &str = " Go to: ";
pub const FIND_PROMPT: &str = " Find: ";
pub const SPLIT_PROMPT: &str = " Split: ";
pub const COMMAND_PROMPT: &str = " Command: ";

#[derive(Default)] pub struct UtilityWidget{
    pub rect: Rect,
    pub text_box: InteractiveTextBox,
}

#[derive(Default)] pub struct UtilityPromptWidget{pub rect: Rect}

#[derive(Default, Clone)] pub struct Highlighter{
    pub selection: Option<Selection2d>, //util bar text should be guaranteed to be one line...
    pub cursor: Option<Position>,
}

/// Container type for widgets on the util bar.
#[derive(Default)] pub struct UtilBar{
    pub prompt: UtilityPromptWidget,
    pub utility_widget: UtilityWidget,
    pub highlighter: Highlighter,
}

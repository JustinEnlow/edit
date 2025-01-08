use crossterm::cursor;
use edit_core::selection::CursorSemantics;
use ratatui::style::Color;

// users preferred cursor style. Options: DefaultUserShape, BlinkingBLock(inform crossterm of capital L in 'Block'), SteadyBlock, BlinkingUnderScore, SteadyUnderScore, BlinkingBar, SteadyBar
pub const CURSOR_STYLE: cursor::SetCursorStyle = cursor::SetCursorStyle::SteadyBlock;

// should only really be using Block semantics in a terminal...
pub const CURSOR_SEMANTICS: CursorSemantics = match CURSOR_STYLE{
    cursor::SetCursorStyle::BlinkingBar | cursor::SetCursorStyle::SteadyBar => CursorSemantics::Bar,
    _ => CursorSemantics::Block
};

pub const VIEW_SCROLL_AMOUNT: usize = 1;    //should this have separate vertical and horizontal definitions?

// should TAB_WIDTH be defined here instead of in edit_core?

// what other config should be here?
    //themeing/coloring consts
    pub const LINE_NUMBER_BACKGROUNG_COLOR: Color = Color::Black;
    pub const LINE_NUMBER_FOREGROUNG_COLOR: Color = Color::Rgb(100, 100, 100);
    pub const DOCUMENT_BACKGROUND_COLOR: Color = Color::Black;
    pub const DOCUMENT_FOREGROUND_COLOR: Color = Color::White;
    pub const STATUS_BAR_BACKGROUND_COLOR: Color = Color::DarkGray; //should this be broken down into widget specific background colors?
    pub const STATUS_BAR_FOREGROUND_COLOR: Color = Color::White;    //should this be broken down into widget specific foreground colors?
    pub const UTIL_BAR_BACKGROUND_COLOR: Color = Color::Black;
    pub const UTIL_BAR_FOREGROUND_COLOR: Color = Color::White;
    pub const UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR: Color = Color::Red;
    pub const WARNING_BACKGROUND_COLOR: Color = Color::Red;
    pub const WARNING_FOREGROUND_COLOR: Color = Color::White;
    pub const COPIED_INDICATOR_BACKGROUND_COLOR: Color = Color::Green;
    pub const COPIED_INDICATOR_FOREGROUND_COLOR: Color = Color::Black;

    pub const SELECTION_BACKGROUND_COLOR: Color = Color::Blue;
    pub const SELECTION_FOREGROUND_COLOR: Color = Color::Black;
    pub const CURSOR_BACKGROUND_COLOR: Color = Color::White;
    pub const CURSOR_FOREGROUND_COLOR: Color = Color::Black;

// By default, this editor shows a warning when a requested action would result in the same state.
// This is to make every action have a visible response.
// To disable, change to false...
pub const SHOW_SAME_STATE_WARNINGS: bool = true;

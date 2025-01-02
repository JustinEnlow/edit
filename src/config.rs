use crossterm::cursor;
use edit_core::selection::CursorSemantics;

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

// By default, this editor shows a warning when a requested action would result in the same state.
// This is to make every action have a visible response.
// To disable, change to false...
pub const SHOW_SAME_STATE_WARNINGS: bool = true;

use crate::selection::CursorSemantics;
use ratatui::style::Color;

// users preferred cursor style. Options: DefaultUserShape, BlinkingBLock(inform crossterm of capital L in 'Block'), SteadyBlock, BlinkingUnderScore, SteadyUnderScore, BlinkingBar, SteadyBar
//pub const CURSOR_STYLE: cursor::SetCursorStyle = cursor::SetCursorStyle::SteadyBlock;

//pub const CURSOR_SEMANTICS: CursorSemantics = match CURSOR_STYLE{
//    cursor::SetCursorStyle::BlinkingBar | cursor::SetCursorStyle::SteadyBar => CursorSemantics::Bar,
//    _ => CursorSemantics::Block
//};
pub const CURSOR_SEMANTICS: CursorSemantics = CursorSemantics::Block;

/// Determines whether the full file path or just the file name should be displayed when showing the document's name.
pub const USE_FULL_FILE_PATH: bool = false;

/// Indicates whether to use hard tabs (e.g., `\t`) or spaces for indentation.
///     - If `USE_HARD_TAB` is `true`, a literal tab character (`\t`) is inserted.
///     - If `USE_HARD_TAB` is `false`, spaces are inserted, with the number of spaces determined by the `TAB_WIDTH` setting.
pub const USE_HARD_TAB: bool = false;   //maybe do enum TabStyle{Hard, Soft, Smart}
/// Specifies the display width of a tab character. 
/// This value could be adjusted based on user preferences or configuration, though there are currently no per-language settings.
pub const TAB_WIDTH: usize = 4; //should this be language dependant? on-the-fly configurable?   //TODO: consider what to do with files where the tab width already in use is different than this setting

pub const VIEW_SCROLL_AMOUNT: usize = 1;    //should this have separate vertical and horizontal definitions?

// what other config should be here?
    //themeing/coloring consts
    pub const LINE_NUMBER_BACKGROUNG_COLOR: Color = Color::Rgb(0, 0, 0);
    pub const LINE_NUMBER_FOREGROUNG_COLOR: Color = Color::Rgb(100, 100, 100);
    pub const DOCUMENT_BACKGROUND_COLOR: Color = Color::Rgb(0, 0, 0);
    pub const DOCUMENT_FOREGROUND_COLOR: Color = Color::White;
    pub const STATUS_BAR_BACKGROUND_COLOR: Color = Color::DarkGray; //should this be broken down into widget specific background colors?
    pub const STATUS_BAR_FOREGROUND_COLOR: Color = Color::White;    //should this be broken down into widget specific foreground colors?
    pub const UTIL_BAR_BACKGROUND_COLOR: Color = Color::Black;
    pub const UTIL_BAR_FOREGROUND_COLOR: Color = Color::White;
    pub const UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR: Color = Color::Red;
    pub const ERROR_BACKGROUND_COLOR: Color = Color::Red;
    pub const ERROR_FOREGROUND_COLOR: Color = Color::White;
    pub const WARNING_BACKGROUND_COLOR: Color = Color::Yellow;
    pub const WARNING_FOREGROUND_COLOR: Color = Color::Black;
    pub const NOTIFY_BACKGROUND_COLOR: Color = Color::Green;
    pub const NOTIFY_FOREGROUND_COLOR: Color = Color::Black;
    pub const INFO_BACKGROUND_COLOR: Color = Color::DarkGray;
    pub const INFO_FOREGROUND_COLOR: Color = Color::Gray;

    pub const SELECTION_BACKGROUND_COLOR: Color = Color::Blue;
    pub const SELECTION_FOREGROUND_COLOR: Color = Color::Black;
    pub const PRIMARY_CURSOR_BACKGROUND_COLOR: Color = Color::White;//Rgb(0, 255, 0);
    pub const PRIMARY_CURSOR_FOREGROUND_COLOR: Color = Color::Black;
    pub const CURSOR_BACKGROUND_COLOR: Color = Color::Rgb(150, 150, 150);
    pub const CURSOR_FOREGROUND_COLOR: Color = Color::Black;

    pub const CURSOR_COLUMN_BACKGROUND_COLOR: Color = Color::Rgb(45, 45, 45);
    pub const CURSOR_COLUMN_FOREGROUND_COLOR: Color = Color::White;
    pub const CURSOR_LINE_BACKGROUND_COLOR: Color = Color::Rgb(45, 45, 45);
    pub const CURSOR_LINE_FOREGROUND_COLOR: Color = Color::White;

pub const SHOW_CURSOR_COLUMN: bool = false;
pub const SHOW_CURSOR_LINE: bool = false;

// user can change these text strings to customize their error/warning/notify/info messages
// errors/warnings/notifications/information

    //this could allow the user to decide which mode to display messages in...
    //TODO: need to match on this(in application.rs?), and display the message in the appropriate mode
    pub enum DisplayMode{Error, Warning, Notify, Info}

    pub const FILE_MODIFIED: &'static str = "File has unsaved changes";
    //maybe this should always be error mode, since we match against this to handle quitting...
    //pub const FILE_MODIFIED_DISPLAY_MODE: DisplayMode = DisplayMode::Error;

    pub const FILE_SAVE_FAILED: &'static str = "File could not be saved";
    pub const FILE_SAVE_FAILED_DISPLAY_MODE: DisplayMode = DisplayMode::Error;

    pub const COMMAND_PARSE_FAILED: &'static str = "Failed to parse command. Command may be undefined";
    pub const COMMAND_PARSE_FAILED_DISPLAY_MODE: DisplayMode = DisplayMode::Error;

    pub const SINGLE_SELECTION: &'static str = "Requested action cannot be performed on single selection";
    pub const SINGLE_SELECTION_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;
    
    pub const MULTIPLE_SELECTIONS: &'static str = "Requested action cannot be performed on multiple selections";
    pub const MULTIPLE_SELECTIONS_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;
    
    pub const INVALID_INPUT: &'static str = "Invalid input";
    pub const INVALID_INPUT_DISPLAY_MODE: DisplayMode = DisplayMode::Error;
    
    pub const SAME_STATE: &'static str = "Requested action results in the same state";
    pub const SAME_STATE_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;
    
    pub const UNHANDLED_KEYPRESS: &'static str = "Unbound key pressed";
    pub const UNHANDLED_KEYPRESS_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;
    
    pub const UNHANDLED_EVENT: &'static str = "Unhandled event occurred";
    pub const UNHANDLED_EVENT_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;

    pub const READ_ONLY_BUFFER: &'static str = "Buffer is read only";
    pub const READ_ONLY_BUFFER_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;

    pub const COPIED_TEXT: &'static str = "Text copied to clipboard.";
    pub const COPIED_TEXT_DISPLAY_MODE: DisplayMode = DisplayMode::Notify;

    // By default, this editor shows a warning when a requested action would result in the same state.
    // This is to make every action have a visible response.
    // To disable, change to false...
    pub const SHOW_SAME_STATE_WARNINGS: bool = true;
//

//
pub const SHOW_CONTEXTUAL_KEYBINDS: bool = true;    //may break these up into per mode toggles
// whether popup menus should display the source(edit_core or name of external utility that provides command functionality) for each command
pub const SHOW_COMMAND_SOURCES_IN_POPUP_MENUS: bool = false;
// whether key binds in popup menus should be represented as a symbol or text
pub const SHOW_SYMBOLIC_MENU_KEYS: bool = true;
//
pub const SHOW_POPUP_MENU_COLUMN_HEADERS: bool = true;

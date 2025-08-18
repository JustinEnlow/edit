use crate::selection::CursorSemantics;
use ratatui::style::Color;

//this should contain config options that could be changed at runtime
#[derive(Clone)] pub struct Config{
    //pub options: HashMap<String, OptionType>, //OptionType{Bool(bool), U8(u8), String(String), etc}    //container for config options     //add/remove/set-option
    //pub user_commands: Vec<Command>,  //container for user defined commands. built ins stored elsewhere       //add/remove-command
    //pub user_hooks: Vec<Hook>,        //add/remove-hook
    pub semantics: CursorSemantics,
    pub use_full_file_path: bool,
    pub use_hard_tab: bool,
    pub tab_width: usize,
    pub view_scroll_amount: usize,
    pub show_cursor_column: bool,
    pub show_cursor_line: bool,
    pub keybinds: indexmap::IndexMap<(crate::mode::Mode, crossterm::event::KeyEvent), crate::action::Action>,   //maybe instead of value being an Action, it should be a command string...  //add/remove-keybind
    //maybe message display modes?...
    //maybe others...
}
//display_line_numbers_on_startup can be passed to Application::new() separately, since it doesn't need to be stored
//display_status_bar_on_startup can be passed to Application::new() separately, since it doesn't need to be stored

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

pub const DISPLAY_LINE_NUMBERS_ON_STARTUP: bool = true;
pub const DISPLAY_STATUS_BAR_ON_STARTUP: bool = true;

// what other config should be here?
//themeing/coloring consts
    pub const LINE_NUMBER_BACKGROUND_COLOR: Color = Color::Rgb(0, 0, 0);
    pub const LINE_NUMBER_FOREGROUND_COLOR: Color = Color::Rgb(100, 100, 100);
    
    pub const DOCUMENT_BACKGROUND_COLOR: Color = Color::Rgb(0, 0, 0);
    pub const DOCUMENT_FOREGROUND_COLOR: Color = Color::White;
    
    pub const STATUS_BAR_BACKGROUND_COLOR: Color = Color::DarkGray; //should this be broken down into widget specific background colors?
    //pub const STATUS_BAR_FOREGROUND_COLOR: Color = Color::White;    //should this be broken down into widget specific foreground colors?
    //TODO: foreground colors for all other status bar widgets
    pub const READ_ONLY_WIDGET_FOREGROUND_COLOR: Color = Color::White;//Rgb(0, 0, 100);
    pub const FILE_NAME_WIDGET_FOREGROUND_COLOR: Color = Color::White;//Rgb(10, 10, 10);
    pub const MODIFIED_WIDGET_FOREGROUND_COLOR: Color = Color::White;//Cyan;
    pub const SELECTIONS_WIDGET_FOREGROUND_COLOR: Color = Color::White;//Rgb(100, 255, 100);
    pub const CURSOR_POSITION_WIDGET_FOREGROUND_COLOR: Color = Color::White;
    pub const MODE_WIDGET_FOREGROUND_COLOR: Color = Color::White;//Rgb(100, 0, 0);
    
    pub const UTIL_BAR_BACKGROUND_COLOR: Color = Color::Black;
    pub const UTIL_BAR_FOREGROUND_COLOR: Color = Color::White;
    pub const UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR: Color = Color::Red;
    pub const ERROR_BACKGROUND_COLOR: Color = Color::Red;
    pub const ERROR_FOREGROUND_COLOR: Color = Color::White;
    pub const WARNING_BACKGROUND_COLOR: Color = Color::Rgb(180, 180, 0);//Color::Rgb(255, 255, 0);
    pub const WARNING_FOREGROUND_COLOR: Color = Color::Black;
    pub const NOTIFY_BACKGROUND_COLOR: Color = Color::Green;
    pub const NOTIFY_FOREGROUND_COLOR: Color = Color::Black;
    pub const INFO_BACKGROUND_COLOR: Color = Color::Black;
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
pub const SHOW_CURSOR_LINE: bool = true;//false;

// user can change these text strings to customize contextual util bar messages
// errors/warnings/notifications/information

    //allows the user to decide which mode to display messages in...
    //Ignore variant can be used to intentionally not display certain messages
    pub enum DisplayMode{Error, Warning, Notify, Info, Ignore}  //should Info really be a part of this?...

    pub const FILE_MODIFIED: &str = "Buffer has unsaved changes";
    //maybe this should always be error mode, since we match against this to handle quitting...
    //pub const FILE_MODIFIED_DISPLAY_MODE: DisplayMode = DisplayMode::Error;

    pub const FILE_SAVE_FAILED: &str = "Buffer could not be saved to file";
    pub const FILE_SAVE_FAILED_DISPLAY_MODE: DisplayMode = DisplayMode::Error;

    pub const COMMAND_PARSE_FAILED: &str = "Failed to parse command. Command may be undefined or malformed";
    pub const COMMAND_PARSE_FAILED_DISPLAY_MODE: DisplayMode = DisplayMode::Error;

    pub const SINGLE_SELECTION: &str = "Requested action cannot be performed on single selection";
    pub const SINGLE_SELECTION_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;
    
    pub const MULTIPLE_SELECTIONS: &str = "Requested action cannot be performed on multiple selections";
    pub const MULTIPLE_SELECTIONS_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;
    
    pub const INVALID_INPUT: &str = "Invalid input";
    pub const INVALID_INPUT_DISPLAY_MODE: DisplayMode = DisplayMode::Error;
    
    pub const SAME_STATE: &str = "Requested action results in the same state";
    pub const SAME_STATE_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;
    
    pub const UNHANDLED_KEYPRESS: &str = "Unbound key pressed";
    pub const UNHANDLED_KEYPRESS_DISPLAY_MODE: DisplayMode = DisplayMode::Error;
    
    pub const UNHANDLED_EVENT: &str = "Unhandled event occurred";
    pub const UNHANDLED_EVENT_DISPLAY_MODE: DisplayMode = DisplayMode::Ignore;//Warning;//Error;

    pub const READ_ONLY_BUFFER: &str = "Buffer is read only";
    pub const READ_ONLY_BUFFER_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;

    pub const COPIED_TEXT: &str = "Text copied to clipboard";
    pub const COPIED_TEXT_DISPLAY_MODE: DisplayMode = DisplayMode::Notify;

    pub const SELECTION_ACTION_OUT_OF_VIEW: &str = "A selection action occurred out of view";
    pub const SELECTION_ACTION_DISPLAY_MODE: DisplayMode = DisplayMode::Info;

    pub const EDIT_ACTION_OUT_OF_VIEW: &str = "An edit action occurred out of view";
    pub const EDIT_ACTION_DISPLAY_MODE: DisplayMode = DisplayMode::Info;

    pub const SPANS_MULTIPLE_LINES: &str = "Requested action cannot be performed on a selection that spans multiple lines";
    pub const SPANS_MULTIPLE_LINES_DISPLAY_MODE: DisplayMode = DisplayMode::Warning;
//

// whether to display a popup menu showing mode specific keybinds   //TODO: need to add status bar Mode indicator, for when this is set to false, so user can see what mode they are in
pub const SHOW_CONTEXTUAL_KEYBINDS: bool = true;    //may break these up into per mode toggles
// whether popup menus should display the source(edit_core or name of external utility that provides command functionality) for each command
pub const SHOW_COMMAND_SOURCES_IN_POPUP_MENUS: bool = true;
    // whether key binds in popup menus should be represented as a symbol or text
    //pub const SHOW_SYMBOLIC_MENU_KEYS: bool = false;  //maybe let user define a string per keycode/modifier in config instead...
//
pub const SHOW_POPUP_MENU_COLUMN_HEADERS: bool = true;

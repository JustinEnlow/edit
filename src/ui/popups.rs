use ratatui::layout::Rect;
use unicode_segmentation::UnicodeSegmentation;
use crate::config;


//TODO: always make sure to add new widgets to update_layouts fn in ui.rs, so that they have screen space assigned to them

//maybe these belong in config.rs?...
const ESCAPE_GRAPHEME: &str = "␛";
const ENTER_GRAPHEME: &str = "⏎";
const UP_GRAPHEME: &str = "↑";
const DOWN_GRAPHEME: &str = "↓";
const LEFT_GRAPHEME: &str = "←";
const RIGHT_GRAPHEME: &str = "→";


const GOTO_MODE_MENU_ITEMS: [MenuItem; 4] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ENTER_GRAPHEME}else{"enter"},   command: "go to specified line number",         source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{UP_GRAPHEME}else{"up"},         command: "move up specified number of times",   source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{DOWN_GRAPHEME}else{"down"},     command: "move down specified number of times", source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",                           source: "(edit)"},
];
const COMMAND_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ENTER_GRAPHEME}else{"enter"},   command: "submit command", source: "(edit)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",      source: "(edit)"},
];
const FIND_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ENTER_GRAPHEME}else{"enter"},   command: "accept new selections", source: "(edit)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",             source: "(edit)"},
];
const SPLIT_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ENTER_GRAPHEME}else{"enter"},   command: "accept new selections", source: "(edit)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",             source: "(edit)"},
];
const ERROR_MODE_MENU_ITEMS: [MenuItem; 1] = [     //may have to have a second for WarningKing::FileIsModified, which has an extra "q quit ignoring changes (edit)" menu item
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode", source: "(edit)"},
];
const MODIFIED_ERROR_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: "ctrl+q",                                                          command: "quit ignoring changes", source: "(edit)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",             source: "(edit)"},
];
const WARNING_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",                                  source: "(edit)"},
    MenuItem{key: "",                                                                command: "all other keys fall through to Insert mode", source: ""},
];
const NOTIFY_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",                                  source: "(edit)"},
    MenuItem{key: "",                                                                command: "all other keys fall through to Insert mode", source: ""},
];
const INFO_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",                                  source: "(edit)"},
    MenuItem{key: "",                                                                command: "all other keys fall through to Insert mode", source: ""},
];
const VIEW_MODE_MENU_ITEMS: [MenuItem; 9] = [
    MenuItem{key: "v",                                                               command: "center vertically around primary cursor",                    source: "(core)"},
    MenuItem{key: "h",                                                               command: "center horizontally around primary cursor(not implemented)", source: "(core)"},
    MenuItem{key: "t",                                                               command: "align with primary cursor at top(not implemented)",          source: "(core)"},
    MenuItem{key: "b",                                                               command: "align with primary cursor at bottom(not implemented)",       source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{UP_GRAPHEME}else{"up"},         command: "scroll up",                                                  source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{DOWN_GRAPHEME}else{"down"},     command: "scroll down",                                                source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{LEFT_GRAPHEME}else{"left"},     command: "scroll left",                                                source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{RIGHT_GRAPHEME}else{"right"},   command: "scroll right",                                               source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",                                                  source: "(edit)"},
];
const OBJECT_MODE_MENU_ITEMS: [MenuItem; 7] = [
    MenuItem{key: "w",                                                               command: "word(not implemented)",                       source: "(core)"},
    MenuItem{key: "s",                                                               command: "sentence(not implemented)",                   source: "(core)"},
    MenuItem{key: "p",                                                               command: "paragraph(not implemented)",                  source: "(core)"},
    MenuItem{key: "b",                                                               command: "surrounding bracket pair",                    source: "(core)"},
    MenuItem{key: "e",                                                               command: "exclusive surrounding pair(not implemented)", source: "(core)"},
    MenuItem{key: "i",                                                               command: "inclusive surrounding pair(not implemented)", source: "(core)"},
    //TODO?: surrounding whitespace?
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",                                    source: "(edit)"},
];
const ADD_SURROUND_MODE_MENU_ITEMS: [MenuItem; 5] = [
    MenuItem{key: "[",                                                               command: "add surrounding square brackets", source: "(core)"},
    MenuItem{key: "{",                                                               command: "add surrounding curly brackets",  source: "(core)"},
    MenuItem{key: "(",                                                               command: "add surrounding paren",           source: "(core)"},
    MenuItem{key: "<",                                                               command: "add surrounding angle brackets",  source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{ESCAPE_GRAPHEME}else{"escape"}, command: "exit mode",                       source: "(edit)"},
];



// TODO: suggestions widget

/// Structure to hold relevant menu item information
// display format: " <keybind>  command string  (source)\n"
#[derive(Clone)]
struct MenuItem<'a>{
    pub key: &'a str,           //(the key the user should press to select this option)(should be assigned in config?, and kept up to date in this module)
    pub command: &'a str,       //(a short explanation of the command)(should be assigned in config? or external utility?)
    pub source: &'a str,        //(edit_core or external utility name)(SHOW_SOURCE toggle in config...)
}
/// Structure to ease the building of popups that display a menu of available commands
//TODO: impl separate popup struct for popups with different behavior(like suggestions, etc.)
pub struct PopupMenu{
    pub rect: Rect,
    pub widest_element_len: u16,    //+2 for border //the number of chars in the widest option in the space menu
    pub num_elements: u16,  //+2 for border //the number of options in the space menu
    pub text: String,
    pub title: String,
    //bg_color: Color   //if we want to keep mode specific styling for popup
    //fg_color: Color   //if we want to keep mode specific styling for popup
}
impl PopupMenu{
    pub fn new(content: &str, context_menu_title: &str) -> Self{
        //get len of longest line and number of lines from content
        let lines = content.lines();
        let mut num_lines = 0;
        let mut longest_line_len = 0;
        for line in lines{
            //if line.len() as u16 > longest_line_len{    //TODO: this seems to be counting chars, we need it to count graphemes, or more accurately, the number of terminal cells used for display. (wide graphemes may take up multiple terminal cells...)
            if UnicodeSegmentation::graphemes(line, true).count() as u16 > longest_line_len{    //this still prob isn't right for wide graphemes...
                longest_line_len = line.len() as u16;
            }
            num_lines = num_lines + 1;
        }
        
        Self{
            rect: Rect::default(),
            widest_element_len: longest_line_len + 2,   //TODO: make a note why we need to add this number
            num_elements: num_lines + 2,                //TODO: make a note why we need to add this number
            text: String::from(content),
            title: String::from(context_menu_title),
        }
    }
    fn new_from_mode_menu(mode_menu: &[MenuItem], context_menu_title: &str) -> Self{
        let key = "KEY";
        let key_grapheme_count = UnicodeSegmentation::graphemes(key, true).count();
        let command = "COMMAND";
        let command_grapheme_count = UnicodeSegmentation::graphemes(command, true).count();
        let source = "SOURCE";
        let source_grapheme_count = UnicodeSegmentation::graphemes(source, true).count();

        let mut longest_key = key_grapheme_count;
        let mut longest_command = command_grapheme_count;
        let mut longest_source = source_grapheme_count;
        for menu_item in mode_menu{
            if menu_item.key.len() > longest_key{longest_key = menu_item.key.len();}
            if menu_item.command.len() > longest_command{longest_command = menu_item.command.len();}
            if menu_item.source.len() > longest_source{longest_source = menu_item.source.len();}
        }
        
        let mut content = String::new();
        let leading_padding = " ";
        let key_command_inner_padding = "  ";
        let command_source_inner_padding = if config::SHOW_COMMAND_SOURCES_IN_POPUP_MENUS{" "}else{""};
        let trailing_padding = " ";

        if config::SHOW_POPUP_MENU_COLUMN_HEADERS{
            let key_padding = " ".repeat(longest_key.saturating_sub(key_grapheme_count));
            let command_padding = if config::SHOW_COMMAND_SOURCES_IN_POPUP_MENUS{" ".repeat(longest_command.saturating_sub(command_grapheme_count))}
            else{String::new()};
            let source = if config::SHOW_COMMAND_SOURCES_IN_POPUP_MENUS{source}
            else{""};
            content.push_str(&format!(
                "{leading_padding}{key}{key_padding}{key_command_inner_padding}{command}{command_padding}{command_source_inner_padding}{source}{trailing_padding}\n"
            ));
        }

        for menu_item in mode_menu{
            let key = menu_item.key;
            let key_padding = " ".repeat(longest_key.saturating_sub(UnicodeSegmentation::graphemes(menu_item.key, true).count()));
            let command = menu_item.command;
            let command_padding = if config::SHOW_COMMAND_SOURCES_IN_POPUP_MENUS{" ".repeat(longest_command.saturating_sub(UnicodeSegmentation::graphemes(menu_item.command, true).count()))}
            else{String::new()};
            let source = if config::SHOW_COMMAND_SOURCES_IN_POPUP_MENUS{menu_item.source.to_string()}
            else{String::new()};
            content.push_str(&format!(
                "{leading_padding}{key}{key_padding}{key_command_inner_padding}{command}{command_padding}{command_source_inner_padding}{source}{trailing_padding}\n"
            ));
        }
        
        PopupMenu::new(&content, context_menu_title)
    }
}

/// Container type for popup style widgets.
pub struct Popups{
    pub goto: PopupMenu,
    pub command: PopupMenu,
    pub find: PopupMenu,
    pub split: PopupMenu,
    pub error: PopupMenu,
    pub modified_error: PopupMenu,  //TODO?: maybe remove this, and use normal error mode display instead?...
    pub warning: PopupMenu,
    pub notify: PopupMenu,
    pub info: PopupMenu,
    pub view: PopupMenu,
    pub object: PopupMenu,
    pub add_surround: PopupMenu,
}
impl Popups{
    pub fn new() -> Self{
        Self{
            goto: PopupMenu::new_from_mode_menu(&GOTO_MODE_MENU_ITEMS, "Goto"),
            command: PopupMenu::new_from_mode_menu(&COMMAND_MODE_MENU_ITEMS, "Command"),
            find: PopupMenu::new_from_mode_menu(&FIND_MODE_MENU_ITEMS, "Find"),
            split: PopupMenu::new_from_mode_menu(&SPLIT_MODE_MENU_ITEMS, "Split"),
            error: PopupMenu::new_from_mode_menu(&ERROR_MODE_MENU_ITEMS, "Error"),
            modified_error: PopupMenu::new_from_mode_menu(&MODIFIED_ERROR_MODE_MENU_ITEMS, "Error(Modified)"),
            warning: PopupMenu::new_from_mode_menu(&WARNING_MODE_MENU_ITEMS, "Warning"),
            notify: PopupMenu::new_from_mode_menu(&NOTIFY_MODE_MENU_ITEMS, "Notify"),
            info: PopupMenu::new_from_mode_menu(&INFO_MODE_MENU_ITEMS, "Info"),
            view: PopupMenu::new_from_mode_menu(&VIEW_MODE_MENU_ITEMS, "View"),
            object: PopupMenu::new_from_mode_menu(&OBJECT_MODE_MENU_ITEMS, "Object"),
            add_surround: PopupMenu::new_from_mode_menu(&ADD_SURROUND_MODE_MENU_ITEMS, "Surround")
        }
    }
}

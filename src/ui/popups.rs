use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color};
use unicode_segmentation::UnicodeSegmentation;
use crate::config;


//TODO: always make sure to add new widgets to update_layouts fn in ui.rs, so that they have screen space assigned to them


const GOTO_MODE_MENU_ITEMS: [MenuItem; 4] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"⏎"}else{"enter"},    command: "go to specified line number",         source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"↑"}else{"up"},       command: "move up specified number of times",   source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"↓"}else{"down"},     command: "move down specified number of times", source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",                           source: "(edit)"},
];
const COMMAND_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"⏎"}else{"enter"},    command: "submit command", source: "(edit)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",      source: "(edit)"},
];
const FIND_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"⏎"}else{"enter"},    command: "accept new selections", source: "(edit)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",             source: "(edit)"},
];
const SPLIT_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"⏎"}else{"enter"},    command: "accept new selections", source: "(edit)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",             source: "(edit)"},
];
const WARNING_MODE_MENU_ITEMS: [MenuItem; 1] = [     //may have to have a second for WarningKing::FileIsModified, which has an extra "q quit ignoring changes (edit)" menu item
MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode", source: "(edit)"},
];
const MODIFIED_WARNING_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: "ctrl+q",                                                command: "quit ignoring changes", source: "(edit)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",             source: "(edit)"},
];
const NOTIFY_MODE_MENU_ITEMS: [MenuItem; 2] = [
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",                                  source: "(edit)"},
    MenuItem{key: "",                                                      command: "all other keys fall through to Insert mode", source: ""},
];
const VIEW_MODE_MENU_ITEMS: [MenuItem; 9] = [
    MenuItem{key: "v",                                                     command: "center vertically around primary cursor",                    source: "(core)"},
    MenuItem{key: "h",                                                     command: "center horizontally around primary cursor(not implemented)", source: "(core)"},
    MenuItem{key: "t",                                                     command: "align with primary cursor at top(not implemented)",          source: "(core)"},
    MenuItem{key: "b",                                                     command: "align with primary cursor at bottom(not implemented)",       source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"↑"}else{"up"},       command: "scroll up",                                                  source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"↓"}else{"down"},     command: "scroll down",                                                source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"←"}else{"left"},     command: "scroll left",                                                source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"→"}else{"right"},    command: "scroll right",                                               source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",                                                  source: "(edit)"},
];
const OBJECT_MODE_MENU_ITEMS: [MenuItem; 7] = [
    MenuItem{key: "w",                                                     command: "word(not implemented)",                       source: "(core)"},
    MenuItem{key: "s",                                                     command: "sentence(not implemented)",                   source: "(core)"},
    MenuItem{key: "p",                                                     command: "paragraph(not implemented)",                  source: "(core)"},
    MenuItem{key: "b",                                                     command: "surrounding bracket pair",                    source: "(core)"},
    MenuItem{key: "e",                                                     command: "exclusive surrounding pair(not implemented)", source: "(core)"},
    MenuItem{key: "i",                                                     command: "inclusive surrounding pair(not implemented)", source: "(core)"},
    //TODO?: surrounding whitespace?
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",                  source: "(edit)"},
];
const ADD_SURROUND_MODE_MENU_ITEMS: [MenuItem; 5] = [
    MenuItem{key: "[",                                                     command: "add surrounding square brackets", source: "(core)"},
    MenuItem{key: "{",                                                     command: "add surrounding curly brackets",  source: "(core)"},
    MenuItem{key: "(",                                                     command: "add surrounding paren",           source: "(core)"},
    MenuItem{key: "<",                                                     command: "add surrounding angle brackets",  source: "(core)"},
    MenuItem{key: if config::SHOW_SYMBOLIC_MENU_KEYS{"esc"}else{"escape"}, command: "exit mode",                       source: "(edit)"},
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
    content: String,
    context_menu_title: String,
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
            content: String::from(content),
            context_menu_title: String::from(context_menu_title),
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
            else{"".to_string()};
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
            else{"".to_string()};
            let source = if config::SHOW_COMMAND_SOURCES_IN_POPUP_MENUS{menu_item.source.to_string()}
            else{"".to_string()};
            content.push_str(&format!(
                "{leading_padding}{key}{key_padding}{key_command_inner_padding}{command}{command_padding}{command_source_inner_padding}{source}{trailing_padding}\n"
            ));
        }
        
        PopupMenu::new(&content, context_menu_title)
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.content.clone())
            .block(ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::all())
                .title(self.context_menu_title.clone()))
            .style(Style::new().fg(Color::Rgb(255, 255, 0)))
    }
}

/// Container type for popup style widgets.
pub struct Popups{
    pub goto_mode_widget: PopupMenu,
    pub command_mode_widget: PopupMenu,
    pub find_mode_widget: PopupMenu,
    pub split_mode_widget: PopupMenu,
    pub warning_mode_widget: PopupMenu,
    pub modified_warning_mode_widget: PopupMenu,
    pub notify_mode_widget: PopupMenu,
    pub view_mode_widget: PopupMenu,
    pub object_mode_widget: PopupMenu,
    pub add_surround_mode_widget: PopupMenu,
}
impl Popups{
    pub fn new() -> Self{
        Self{
            goto_mode_widget: PopupMenu::new_from_mode_menu(&GOTO_MODE_MENU_ITEMS, "Goto"),
            command_mode_widget: PopupMenu::new_from_mode_menu(&COMMAND_MODE_MENU_ITEMS, "Command"),
            find_mode_widget: PopupMenu::new_from_mode_menu(&FIND_MODE_MENU_ITEMS, "Find"),
            split_mode_widget: PopupMenu::new_from_mode_menu(&SPLIT_MODE_MENU_ITEMS, "Split"),
            warning_mode_widget: PopupMenu::new_from_mode_menu(&WARNING_MODE_MENU_ITEMS, "Warning"),
            modified_warning_mode_widget: PopupMenu::new_from_mode_menu(&MODIFIED_WARNING_MODE_MENU_ITEMS, "Warning"),
            notify_mode_widget: PopupMenu::new_from_mode_menu(&NOTIFY_MODE_MENU_ITEMS, "Notify"),
            view_mode_widget: PopupMenu::new_from_mode_menu(&VIEW_MODE_MENU_ITEMS, "View"),
            object_mode_widget: PopupMenu::new_from_mode_menu(&OBJECT_MODE_MENU_ITEMS, "Object"),
            add_surround_mode_widget: PopupMenu::new_from_mode_menu(&ADD_SURROUND_MODE_MENU_ITEMS, "Surround")
        }
    }
}

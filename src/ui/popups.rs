use ratatui::layout::Rect;
use unicode_segmentation::UnicodeSegmentation;
use crate::config;


//TODO: always make sure to add new widgets to update_layouts fn in ui.rs, so that they have screen space assigned to them

//maybe these belong in config.rs?...
//const ESCAPE_GRAPHEME: &str = "␛";
//const ENTER_GRAPHEME: &str = "⏎";
//const UP_GRAPHEME: &str = "↑";
//const DOWN_GRAPHEME: &str = "↓";
//const LEFT_GRAPHEME: &str = "←";
//const RIGHT_GRAPHEME: &str = "→";



// TODO: suggestions widget

/// Structure to hold relevant menu item information
// display format: " <keybind>  command string  (source)\n"
#[derive(Clone)]
struct MenuItem{
    pub key: String,           //(the key the user should press to select this option)(should be assigned in config?, and kept up to date in this module)
    pub command: String,       //(a short explanation of the command)(should be assigned in config? or external utility?)
    pub source: String,        //(edit_core or external utility name)(SHOW_SOURCE toggle in config...)
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
    //TODO: could accept flag indicating whether to add "all other keybinds fall through to Insert Mode" to end of menu
    fn new_from_mode_menu(mode_menu: &[MenuItem], context_menu_title: &str) -> Self{
        let key = "KEY";
        let key_grapheme_count = key.graphemes(true).count();
        let command = "COMMAND";
        let command_grapheme_count = command.graphemes(true).count();
        let source = "SOURCE";
        let source_grapheme_count = source.graphemes(true).count();

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
        let command_source_inner_padding = if config::SHOW_COMMAND_SOURCES_IN_POPUP_MENUS{"  "}else{""};
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
            let key = menu_item.key.clone();
            let key_padding = " ".repeat(longest_key.saturating_sub(menu_item.key.graphemes(true).count()));
            let command = menu_item.command.clone();
            let command_padding = if config::SHOW_COMMAND_SOURCES_IN_POPUP_MENUS{" ".repeat(longest_command.saturating_sub(menu_item.command.graphemes(true).count()))}
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
    //TODO: popup text should probably be set in ModePush, so that keybinds added at run time can be included...
    //currently, popup text is only being set at start up
    //or maybe reload whenev user adds a new keybind/command, because reloading every mode_push may be unnecessary
    //TODO: menus are not ordered when using HashMap. try BTreeMap(although, i think this may be more related to sorting), 
    //or the indexmap crate to retain insert order
    pub fn new(keybinds: &/*std::collections::HashMap*/indexmap::IndexMap<(crate::mode::Mode, KeyEvent), crate::action::Action>) -> Self{
        //the hashmap seems to have no set order. every time the editor runs, the order of menu items changes.
        //is there some way to force it to stay the same?...
        use crate::mode::Mode;
        let mut goto_mode_menu_items = Vec::new();
        let mut command_mode_menu_items = Vec::new();
        let mut find_mode_menu_items = Vec::new();
        let mut split_mode_menu_items = Vec::new();
        let mut error_mode_menu_items = Vec::new();
        let mut modified_error_mode_menu_items = Vec::new();
        let mut warning_mode_menu_items = Vec::new();
        let mut notify_mode_menu_items = Vec::new();
        let mut info_mode_menu_items = Vec::new();
        let mut view_mode_menu_items = Vec::new();
        let mut object_mode_menu_items = Vec::new();
        let mut add_surround_mode_menu_items = Vec::new();
        for ((mode, key_event), action) in keybinds{
            fn menu_item(key_event: &KeyEvent, action: &crate::action::Action) -> MenuItem{
                MenuItem{
                    key: format!("{}{}", modifiers(key_event), key(key_event)),
                    command: action.command_name(), 
                    source: action.command_source()
                }
            }
            match mode{
                Mode::Goto => goto_mode_menu_items.push(menu_item(key_event, action)),
                Mode::Command => command_mode_menu_items.push(menu_item(key_event, action)),
                Mode::Find => find_mode_menu_items.push(menu_item(key_event, action)),
                Mode::Split => split_mode_menu_items.push(menu_item(key_event, action)),
                //Mode::Error if !matches!(action, crate::action::Action::EditorAction(crate::action::EditorAction::Quit)) => error_mode_menu_items.push(menu_item(key_event, action)),
                //Mode::Error => modified_error_mode_menu_items.push(menu_item(key_event, action)),
                Mode::Error => {
                    if matches!(action, crate::action::Action::EditorAction(crate::action::EditorAction::Quit)){
                        modified_error_mode_menu_items.push(menu_item(key_event, action));
                    }else{
                        error_mode_menu_items.push(menu_item(key_event, action));
                        modified_error_mode_menu_items.push(menu_item(key_event, action));  //because modified error mode still needs to show all non "quit" keybinds...
                    }
                }
                Mode::Warning => warning_mode_menu_items.push(menu_item(key_event, action)),
                Mode::Notify => notify_mode_menu_items.push(menu_item(key_event, action)),
                Mode::Info => info_mode_menu_items.push(menu_item(key_event, action)),
                Mode::View => view_mode_menu_items.push(menu_item(key_event, action)),
                Mode::Object => object_mode_menu_items.push(menu_item(key_event, action)),
                Mode::AddSurround => add_surround_mode_menu_items.push(menu_item(key_event, action)),
                Mode::Insert => {}
            }
        }

        Self{
            goto: PopupMenu::new_from_mode_menu(&goto_mode_menu_items, "Goto"),
            command: PopupMenu::new_from_mode_menu(&command_mode_menu_items, "Command"),
            find: PopupMenu::new_from_mode_menu(&find_mode_menu_items, "Find"),
            split: PopupMenu::new_from_mode_menu(&split_mode_menu_items, "Split"),
            error: PopupMenu::new_from_mode_menu(&error_mode_menu_items, "Error"),
            modified_error: PopupMenu::new_from_mode_menu(&modified_error_mode_menu_items, "Error(Modified)"),
            warning: PopupMenu::new_from_mode_menu(&warning_mode_menu_items, "Warning"),
            notify: PopupMenu::new_from_mode_menu(&notify_mode_menu_items, "Notify"),
            info: PopupMenu::new_from_mode_menu(&info_mode_menu_items, "Info"),
            view: PopupMenu::new_from_mode_menu(&view_mode_menu_items, "View"),
            object: PopupMenu::new_from_mode_menu(&object_mode_menu_items, "Object"),
            add_surround: PopupMenu::new_from_mode_menu(&add_surround_mode_menu_items, "Surround")
        }
    }
}

use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
fn modifiers(key_event: &KeyEvent) -> String{
    let mut modifiers = String::new();
    if key_event.modifiers.contains(KeyModifiers::META)   {modifiers.push_str(&format!("{}{}", "meta",  "+"));}
    if key_event.modifiers.contains(KeyModifiers::SUPER)  {modifiers.push_str(&format!("{}{}", "super", "+"));}
    if key_event.modifiers.contains(KeyModifiers::HYPER)  {modifiers.push_str(&format!("{}{}", "hyper", "+"));}
    if key_event.modifiers.contains(KeyModifiers::CONTROL){modifiers.push_str(&format!("{}{}", "ctrl",  "+"));}
    if key_event.modifiers.contains(KeyModifiers::ALT)    {modifiers.push_str(&format!("{}{}", "alt",   "+"));}
    if key_event.modifiers.contains(KeyModifiers::SHIFT)  {modifiers.push_str(&format!("{}{}", "shift", "+"));}
    //if key_event.modifiers.contains(KeyModifiers::NONE){/* do nothing*/}
    modifiers
}
fn key(key_event: &KeyEvent) -> String{
    let mut key = String::new();
    match key_event.code{
        KeyCode::BackTab => key.push_str("backtab"),
        KeyCode::Backspace => key.push_str("backspace"),
        KeyCode::CapsLock => key.push_str("capslock"),
        KeyCode::Char(c) => key.push(c),
        KeyCode::Delete => key.push_str("delete"),
        KeyCode::Down => key.push_str("down"),
        KeyCode::End => key.push_str("end"),
        KeyCode::Enter => key.push_str("enter"),
        KeyCode::Esc => key.push_str("escape"),
        KeyCode::F(num) => key.push_str(&format!("f{}", num)),
        KeyCode::Home => key.push_str("home"),
        KeyCode::Insert => key.push_str("insert"),
        KeyCode::KeypadBegin => key.push_str("keypad_begin"),
        KeyCode::Left => key.push_str("left"),
        KeyCode::Media(idk) => key.push_str(&format!("media_{:?}", idk)),
        KeyCode::Menu => key.push_str("menu"),
        KeyCode::Modifier(idfk) => key.push_str(&format!("modifier_{:?}", idfk)),
        KeyCode::Null => key.push_str("null"),
        KeyCode::NumLock => key.push_str("numlock"),
        KeyCode::PageDown => key.push_str("page_down"),
        KeyCode::PageUp => key.push_str("page_up"),
        KeyCode::Pause => key.push_str("pause"),
        KeyCode::PrintScreen => key.push_str("print_screen"),
        KeyCode::Right => key.push_str("right"),
        KeyCode::ScrollLock => key.push_str("scroll_lock"),
        KeyCode::Tab => key.push_str("tab"),
        KeyCode::Up => key.push_str("up"),
    }
    key
}

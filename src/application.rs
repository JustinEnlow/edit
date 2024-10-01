use edit::selection::{CursorSemantics, Movement};
use ratatui::{backend::CrosstermBackend, Terminal};
use edit::document::Document;
use crate::ui::UserInterface;
use std::error::Error;
use std::path::PathBuf;
use crossterm::{
    cursor,
    terminal,
    execute,
    ExecutableCommand
};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};



// users preferred cursor style
    // Options:
        // DefaultUserShape
        // BlinkingBLock    //inform crossterm of capital L in 'Block'
        // SteadyBlock
        // BlinkingUnderScore
        // SteadyUnderScore
        // BlinkingBar
        // SteadyBar
const CURSOR_STYLE: cursor::SetCursorStyle = cursor::SetCursorStyle::SteadyBlock;
const VIEW_SCROLL_AMOUNT: usize = 1;



#[derive(Clone, Copy)]
pub enum Mode{
    Insert,
    Warning(WarningKind),
    Command,
    FindReplace,    //FindReplace mode can prob be replaced with a selection based find/replace
    Goto,
    //Utility(UtilityKind),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WarningKind{
    FileIsModified,
    FileSaveFailed,
}



pub struct Application{
    should_quit: bool,
    mode: Mode,
    host_terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    supports_keyboard_enhancement: bool,
    document: Document,
    ui: UserInterface,
}
impl Application{
    pub fn new() -> Result<Self, Box<dyn Error>>{
        let (terminal, supports_keyboard_enhancement) = setup_terminal()?;
        let terminal_size = terminal.size()?;

        Ok(Self{
            should_quit: false,
            mode: Mode::Insert,
            host_terminal: terminal,
            supports_keyboard_enhancement,
            document: Document::new(),
            ui: UserInterface::new(terminal_size),
        })
    }

    pub fn run(&mut self, file_path: String) -> Result<(), Box<dyn Error>>{
        let path = PathBuf::from(file_path).canonicalize()?;
        
        self.document = Document::open(path)?;
        self.ui.set_file_name(self.document.file_name());
        self.ui.set_document_length(self.document.len());
        
        self.ui.update_layouts(self.mode);
        
        // update client view size
        self.document.view_mut().set_size(self.ui.document_rect().width as usize, self.ui.document_rect().height as usize);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        self.document.view_mut().scroll_following_cursor(&selections, &text);

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());

        loop{
            self.ui.update_layouts(self.mode);
            self.ui.render(&mut self.host_terminal, self.mode)?;
            self.handle_event()?;
            if self.should_quit{
                return Ok(());
            }
        }
    }

    pub fn handle_event(&mut self) -> Result<(), Box<dyn Error>>{
        match event::read()?{
            event::Event::Key(key_event) => {
                match (key_event, self.mode){
                    // Insert Mode
                    //(KeyEvent{modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT, code, ..}, Mode::Insert) => {Action::}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Right,         ..}, Mode::Insert) => {self.move_cursor_word_end()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Left,          ..}, Mode::Insert) => {self.move_cursor_word_start();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Home,          ..}, Mode::Insert) => {self.move_cursor_document_start();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::End,           ..}, Mode::Insert) => {self.move_cursor_document_end();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('q'),     ..}, Mode::Insert) => {self.quit();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('s'),     ..}, Mode::Insert) => {self.save();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('g'),     ..}, Mode::Insert) => {self.set_mode_goto();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('f'),     ..}, Mode::Insert) => {self.set_mode_find_replace();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('l'),     ..}, Mode::Insert) => {self.display_line_numbers();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('k'),     ..}, Mode::Insert) => {self.display_status_bar();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('o'),     ..}, Mode::Insert) => {self.set_mode_command();}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('t'),     ..}, Mode::Insert) => {self.open_new_terminal_window();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Right,         ..}, Mode::Insert) => {self.extend_selection_right();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Left,         ..}, Mode::Insert) => {self.extend_selection_left();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Up,         ..}, Mode::Insert) => {self.extend_selection_up();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Down,         ..}, Mode::Insert) => {self.extend_selection_down();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Home,         ..}, Mode::Insert) => {self.extend_selection_home();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::End,         ..}, Mode::Insert) => {self.extend_selection_end();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Char(c),       ..}, Mode::Insert) => {self.insert_char(c);}
                    (KeyEvent{modifiers: KeyModifiers::ALT,     code: KeyCode::Down,          ..}, Mode::Insert) => {self.scroll_view_down(VIEW_SCROLL_AMOUNT);}
                    (KeyEvent{modifiers: KeyModifiers::ALT,     code: KeyCode::Left,          ..}, Mode::Insert) => {self.scroll_view_left(VIEW_SCROLL_AMOUNT);}
                    (KeyEvent{modifiers: KeyModifiers::ALT,     code: KeyCode::Right,         ..}, Mode::Insert) => {self.scroll_view_right(VIEW_SCROLL_AMOUNT);}
                    (KeyEvent{modifiers: KeyModifiers::ALT,     code: KeyCode::Up,            ..}, Mode::Insert) => {self.scroll_view_up(VIEW_SCROLL_AMOUNT);}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Tab,           ..}, Mode::Insert) => {self.insert_tab();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Enter,         ..}, Mode::Insert) => {self.insert_newline();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Delete,        ..}, Mode::Insert) => {self.delete();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Backspace,     ..}, Mode::Insert) => {self.backspace();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Up,            ..}, Mode::Insert) => {self.move_cursor_up();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Down,          ..}, Mode::Insert) => {self.move_cursor_down();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Left,          ..}, Mode::Insert) => {self.move_cursor_left();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Right,         ..}, Mode::Insert) => {self.move_cursor_right();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::PageUp,        ..}, Mode::Insert) => {self.move_cursor_page_up();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::PageDown,      ..}, Mode::Insert) => {self.move_cursor_page_down();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Home,          ..}, Mode::Insert) => {self.move_cursor_line_start();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::End,           ..}, Mode::Insert) => {self.move_cursor_line_end();}
                    //(KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Esc,           ..}, Mode::Insert) => {ClientAction::CollapseSelectionCursor}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Char(c), ..}, Mode::Insert) => {self.insert_char(c);}
    
                    // Warning Mode
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('q'), ..}, Mode::Warning(_)) => {self.quit_ignoring_changes();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Esc,       ..}, Mode::Warning(_)) => {self.warning_mode_exit();}
    
                    // Goto Mode
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Right,         ..}, Mode::Goto) => {self.goto_mode_extend_selection_right();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Left,          ..}, Mode::Goto) => {self.goto_mode_extend_selection_left();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Home,          ..}, Mode::Goto) => {self.goto_mode_extend_selection_home();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::End,           ..}, Mode::Goto) => {self.goto_mode_extend_selection_end();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Esc,           ..}, Mode::Goto) => {self.goto_mode_exit();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Enter,         ..}, Mode::Goto) => {self.goto_mode_accept();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Backspace,     ..}, Mode::Goto) => {self.goto_mode_backspace();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Delete,        ..}, Mode::Goto) => {self.goto_mode_delete();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Right,         ..}, Mode::Goto) => {self.goto_mode_move_cursor_right();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Left,          ..}, Mode::Goto) => {self.goto_mode_move_cursor_left();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Home,          ..}, Mode::Goto) => {self.goto_mode_move_cursor_line_start();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::End,           ..}, Mode::Goto) => {self.goto_mode_move_cursor_line_end();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Char(c), ..}, Mode::Goto) => {self.goto_mode_insert_char(c);}
                
                    // FindReplace Mode
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Right,         ..}, Mode::FindReplace) => {self.find_replace_mode_extend_selection_right();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Left,          ..}, Mode::FindReplace) => {self.find_replace_mode_extend_selection_left();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Home,          ..}, Mode::FindReplace) => {self.find_replace_mode_extend_selection_home();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::End,           ..}, Mode::FindReplace) => {self.find_replace_mode_extend_selection_end();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Char(c), ..}, Mode::FindReplace) => {self.find_replace_mode_insert_char(c);}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Esc,           ..}, Mode::FindReplace) => {self.find_replace_mode_exit();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Tab,           ..}, Mode::FindReplace) => {self.find_replace_mode_switch_util_bar_focus();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Up,            ..}, Mode::FindReplace) => {self.find_replace_mode_previous_instance();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Down,          ..}, Mode::FindReplace) => {self.find_replace_mode_next_instance();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Backspace,     ..}, Mode::FindReplace) => {self.find_replace_mode_backspace();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Delete,        ..}, Mode::FindReplace) => {self.find_replace_mode_delete();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Right,         ..}, Mode::FindReplace) => {self.find_replace_mode_move_cursor_right();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Left,          ..}, Mode::FindReplace) => {self.find_replace_mode_move_cursor_left();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Home,          ..}, Mode::FindReplace) => {self.find_replace_mode_move_cursor_line_start();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::End,           ..}, Mode::FindReplace) => {self.find_replace_mode_move_cursor_line_end();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Enter,         ..}, Mode::FindReplace) => {self.find_replace_mode_accept();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Char(c), ..}, Mode::FindReplace) => {self.find_replace_mode_insert_char(c);}
                
                    // Command Mode
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Right,         ..}, Mode::Command) => {self.command_mode_extend_selection_right();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Left,          ..}, Mode::Command) => {self.command_mode_extend_selection_left();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Home,          ..}, Mode::Command) => {self.command_mode_extend_selection_home();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::End,           ..}, Mode::Command) => {self.command_mode_extend_selection_end();}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Char(c), ..}, Mode::Command) => {self.command_mode_insert_char(c);}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Esc,           ..}, Mode::Command) => {self.command_mode_exit();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Char(c), ..}, Mode::Command) => {self.command_mode_insert_char(c);}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Enter,         ..}, Mode::Command) => {self.command_mode_accept();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Backspace,     ..}, Mode::Command) => {self.command_mode_backspace();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Delete,        ..}, Mode::Command) => {self.command_mode_delete();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Right,         ..}, Mode::Command) => {self.command_mode_move_cursor_right();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Left,          ..}, Mode::Command) => {self.command_mode_move_cursor_left();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Home,          ..}, Mode::Command) => {self.command_mode_move_cursor_line_start();}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::End,           ..}, Mode::Command) => {self.command_mode_move_cursor_line_end();}
    
                    // unhandled keybinds
                    _ => {self.no_op();}
                }
            },
            event::Event::Resize(x, y) => self.resize(x, y),
            _ => self.no_op(),
        }

        Ok(())
    }

    fn add_selection_above(&mut self){}
    fn add_selection_below(&mut self){}
    fn backspace(&mut self){
        self.document.backspace();

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        self.document.view_mut().scroll_following_cursor(&selections, &text);

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());

        self.ui.set_document_length(self.document.len());
    }
    fn collapse_selections(&mut self){
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            selection.collapse(&text, CursorSemantics::Bar);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn command_mode_accept(&mut self){
        //if parse_command(editor, ui.util_bar().text()).is_ok(){
            self.ui.util_bar_mut().clear();
            self.ui.util_bar_mut().set_offset(0);
            self.mode = Mode::Insert;
        //}
        //ui.scroll(editor);

        //TODO: send action request to server
    }
    fn command_mode_backspace(&mut self){
        self.ui.util_bar_mut().backspace();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_delete(&mut self){
        self.ui.util_bar_mut().delete();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_exit(&mut self){
        self.ui.util_bar_mut().clear();
        self.ui.util_bar_mut().set_offset(0);
        self.mode = Mode::Insert;
    }
    fn command_mode_extend_selection_end(&mut self){
        self.ui.util_bar_mut().extend_selection_end();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_extend_selection_home(&mut self){
        self.ui.util_bar_mut().extend_selection_home();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_extend_selection_left(&mut self){
        self.ui.util_bar_mut().extend_selection_left();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_extend_selection_right(&mut self){
        self.ui.util_bar_mut().extend_selection_right();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_insert_char(&mut self, c: char){
        self.ui.util_bar_mut().insert_char(c);
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_move_cursor_left(&mut self){
        self.ui.util_bar_mut().move_cursor_left();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_move_cursor_line_end(&mut self){
        self.ui.util_bar_mut().move_cursor_end();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_move_cursor_line_start(&mut self){
        self.ui.util_bar_mut().move_cursor_home();
        self.ui.util_bar_mut().scroll();
    }
    fn command_mode_move_cursor_right(&mut self){
        self.ui.util_bar_mut().move_cursor_right();
        self.ui.util_bar_mut().scroll();
    }
    fn delete(&mut self){
        self.document.delete();

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        self.document.view_mut().scroll_following_cursor(&selections, &text);

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());

        self.ui.set_document_length(self.document.len());
    }
    fn display_line_numbers(&mut self){
        self.ui.set_display_line_numbers(!self.ui.display_line_numbers());
                
        self.ui.update_layouts(self.mode);

        self.document.view_mut().set_size(
            self.ui.document_rect().width as usize, 
            self.ui.document_rect().height as usize
        );
                
        let text = self.document.text().clone();
        let selections = self.document.selections().clone();

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn display_status_bar(&mut self){
        self.ui.set_display_status_bar(!self.ui.display_status_bar());
                
        self.ui.update_layouts(self.mode);

        self.document.view_mut().set_size(
            self.ui.document_rect().width as usize, 
            self.ui.document_rect().height as usize
        );

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn extend_selection_down(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().extend_selections_down(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.extend_down(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn extend_selection_end(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().extend_selections_end(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.extend_line_text_end(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn extend_selection_home(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().extend_selections_home(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.extend_home(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn extend_selection_left(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().extend_selections_left(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.extend_left(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn extend_selection_page_up(&mut self){
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        for selection in self.document.selections_mut().iter_mut(){
            selection.extend_page_up(&text, &view);
        }

        //update ui
    }
    fn extend_selection_page_down(&mut self){
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        for selection in self.document.selections_mut().iter_mut(){
            selection.extend_page_down(&text, &view);
        }

        //update ui
    }
    fn extend_selection_right(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().extend_selections_right(&text);
        for selection in self.document.selections_mut().iter_mut(){
            //selection.move_right(&text, Movement::Extend);
            selection.extend_right(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn extend_selection_up(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().extend_selections_up(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.extend_up(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn find_replace_mode_accept(&mut self){}
    fn find_replace_mode_backspace(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().backspace();
        }else{
            self.ui.util_bar_mut().backspace();
        }

        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();

        //run text validity check
        //if let Some(doc) = editor.document(){
        //    if !doc.lines_as_single_string().contains(&ui.util_bar().text()){
        //        ui.util_bar_mut().set_text_is_valid(false);
        //    }else{
        //        ui.util_bar_mut().set_text_is_valid(true);
        //    }
        //}
    }
    fn find_replace_mode_delete(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().delete();
        }else{
            self.ui.util_bar_mut().delete();
        }

        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();

        //run text validity check
        //if let Some(doc) = editor.document(){
        //    if !doc.lines_as_single_string().contains(&ui.util_bar().text()){
        //        ui.util_bar_mut().set_text_is_valid(false);
        //    }else{
        //        ui.util_bar_mut().set_text_is_valid(true);
        //    }
        //}
    }
    fn find_replace_mode_exit(&mut self){
        self.ui.util_bar_mut().clear();
        self.ui.util_bar_alternate_mut().clear();
        self.ui.util_bar_mut().set_offset(0);
        self.ui.util_bar_alternate_mut().set_offset(0);
        self.ui.set_util_bar_alternate_focused(false);
        self.mode = Mode::Insert;
    }
    fn find_replace_mode_extend_selection_end(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().extend_selection_end();
        }else{
            self.ui.util_bar_mut().extend_selection_end();
        }
        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
    }
    fn find_replace_mode_extend_selection_home(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().extend_selection_home();
        }else{
            self.ui.util_bar_mut().extend_selection_home();
        }
        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
    }
    fn find_replace_mode_extend_selection_left(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().extend_selection_left();
        }else{
            self.ui.util_bar_mut().extend_selection_left();
        }
        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
    }
    fn find_replace_mode_extend_selection_right(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().extend_selection_right();
        }else{
            self.ui.util_bar_mut().extend_selection_right();
        }
        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
    }
    fn find_replace_mode_insert_char(&mut self, c: char){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().insert_char(c);
        }else{
            self.ui.util_bar_mut().insert_char(c);
        }

        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();

        //run text validity check
        //if let Some(doc) = editor.document(){
        //    if !doc.lines_as_single_string().contains(&ui.util_bar().text()){
        //        ui.util_bar_mut().set_text_is_valid(false);
        //    }else{
        //        ui.util_bar_mut().set_text_is_valid(true);
        //    }
        //}
    }
    fn find_replace_mode_move_cursor_left(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().move_cursor_left();
        }else{
            self.ui.util_bar_mut().move_cursor_left();
        }

        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
    }
    fn find_replace_mode_move_cursor_line_end(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().move_cursor_end();
        }else{
            self.ui.util_bar_mut().move_cursor_end();
        }

        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
    }
    fn find_replace_mode_move_cursor_line_start(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().move_cursor_home();
        }else{
            self.ui.util_bar_mut().move_cursor_home();
        }

        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
    }
    fn find_replace_mode_move_cursor_right(&mut self){
        if self.ui.util_bar_alternate_focused(){
            self.ui.util_bar_alternate_mut().move_cursor_right();
        }else{
            self.ui.util_bar_mut().move_cursor_right();
        }

        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
    }
    fn find_replace_mode_next_instance(&mut self){}
    fn find_replace_mode_previous_instance(&mut self){}
    fn find_replace_mode_switch_util_bar_focus(&mut self){
        self.ui.set_util_bar_alternate_focused(!self.ui.util_bar_alternate_focused());
    }
    fn goto_mode_accept(&mut self){
        if let Ok(line_number) = self.ui.util_bar().text().parse::<usize>(){
            if line_number.saturating_sub(1) < self.ui.document_length(){
                let text =  self.document.text().clone();
                
                //self.document.selections_mut().go_to(&text, line_number.saturating_sub(1));
                self.document.selections_mut().clear_non_primary_selections();
                self.document.selections_mut().first_mut().set_from_line_number(
                    line_number.saturating_sub(1), 
                    &text, 
                    Movement::Move
                );
                
                let selections = self.document.selections().clone();
                self.document.view_mut().scroll_following_cursor(&selections, &text);
                self.ui.set_text_in_view(self.document.view().text(&text));
                self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
                self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
                self.ui.set_document_cursor_position(selections.cursor_positions(&text));
                self.ui.set_document_modified(self.document.is_modified());
                
                self.ui.util_bar_mut().clear();
                self.ui.util_bar_mut().set_offset(0);
                self.mode = Mode::Insert;
            }
        }
    }
    fn goto_mode_backspace(&mut self){
        self.ui.util_bar_mut().backspace();
        self.ui.util_bar_mut().scroll();
    
        // run text validity check
        let mut is_numeric = true;
        for grapheme in self.ui.util_bar().text().chars(){ // .graphemes(true)?
            if !grapheme.is_ascii_digit(){
                is_numeric = false;
            }
        }
        let exceeds_doc_length = match self.ui.util_bar().text().parse::<usize>(){
            Ok(line_number) => {
                line_number > self.ui.document_length()
            }
            Err(_) => false
        };
        if !is_numeric || exceeds_doc_length{
            self.ui.util_bar_mut().set_text_is_valid(false);
        }else{
            self.ui.util_bar_mut().set_text_is_valid(true);
        }
    }
    fn goto_mode_delete(&mut self){
        self.ui.util_bar_mut().delete();
        self.ui.util_bar_mut().scroll();
    
        // run text validity check
        let mut is_numeric = true;
        for grapheme in self.ui.util_bar().text().chars(){ // .graphemes(true)?
            if !grapheme.is_ascii_digit(){
                is_numeric = false;
            }
        }
        let exceeds_doc_length = match self.ui.util_bar().text().parse::<usize>(){
            Ok(line_number) => {
                line_number > self.ui.document_length()
            }
            Err(_) => false
        };
        if !is_numeric || exceeds_doc_length{
            self.ui.util_bar_mut().set_text_is_valid(false);
        }else{
            self.ui.util_bar_mut().set_text_is_valid(true);
        }
    }
    fn goto_mode_exit(&mut self){
        self.ui.util_bar_mut().clear();
        self.ui.util_bar_mut().set_offset(0);
        self.mode = Mode::Insert;
    }
    fn goto_mode_extend_selection_end(&mut self){
        self.ui.util_bar_mut().extend_selection_end();
        self.ui.util_bar_mut().scroll();
    }
    fn goto_mode_extend_selection_home(&mut self){
        self.ui.util_bar_mut().extend_selection_home();
        self.ui.util_bar_mut().scroll();
    }
    fn goto_mode_extend_selection_left(&mut self){
        self.ui.util_bar_mut().extend_selection_left();
        self.ui.util_bar_mut().scroll();
    }
    fn goto_mode_extend_selection_right(&mut self){
        self.ui.util_bar_mut().extend_selection_right();
        self.ui.util_bar_mut().scroll();
    }
    fn goto_mode_insert_char(&mut self, c: char){
        self.ui.util_bar_mut().insert_char(c);
        self.ui.util_bar_mut().scroll();
    
        // run text validity check
        let mut is_numeric = true;
        for grapheme in self.ui.util_bar().text().chars(){ // .graphemes(true)?
            if !grapheme.is_ascii_digit(){
                is_numeric = false;
            }
        }
        let exceeds_doc_length = match self.ui.util_bar().text().parse::<usize>(){
            Ok(line_number) => {
                line_number > self.ui.document_length()
            }
            Err(_) => false
        };
        if !is_numeric || exceeds_doc_length{
            self.ui.util_bar_mut().set_text_is_valid(false);
        }else{
            self.ui.util_bar_mut().set_text_is_valid(true);
        }
    }
    fn goto_mode_move_cursor_left(&mut self){
        self.ui.util_bar_mut().move_cursor_left();
        self.ui.util_bar_mut().scroll();
    }
    fn goto_mode_move_cursor_line_end(&mut self){
        self.ui.util_bar_mut().move_cursor_end();
        self.ui.util_bar_mut().scroll();
    }
    fn goto_mode_move_cursor_line_start(&mut self){
        self.ui.util_bar_mut().move_cursor_home();
        self.ui.util_bar_mut().scroll();
    }
    fn goto_mode_move_cursor_right(&mut self){
        self.ui.util_bar_mut().move_cursor_right();
        self.ui.util_bar_mut().scroll();
    }
    fn insert_char(&mut self, c: char){
        self.document.insert_char(c);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        self.document.view_mut().scroll_following_cursor(&selections, &text);

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn insert_newline(&mut self){
        self.document.enter();

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        self.document.view_mut().scroll_following_cursor(&selections, &text);

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());

        self.ui.set_document_length(self.document.len());
    }
    fn insert_tab(&mut self){
        self.document.tab();

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        self.document.view_mut().scroll_following_cursor(&selections, &text);

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn move_cursor_document_end(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().move_cursors_document_end(&text);
        self.document.selections_mut().clear_non_primary_selections();
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_doc_end(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_document_start(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().move_cursors_document_start(&text);
        self.document.selections_mut().clear_non_primary_selections();
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_doc_start(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_down(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().move_cursors_down(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_down(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_left(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().move_cursors_left(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_left(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_line_end(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().move_cursors_end(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_line_text_end(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_line_start(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().move_cursors_home(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_home(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_page_down(&mut self){
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        //self.document.selections_mut().move_cursors_page_down(&text, &view);
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_page_down(&text, &view);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_page_up(&mut self){
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        //self.document.selections_mut().move_cursors_page_up(&text, &view);
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_page_up(&text, &view);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_right(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().move_cursors_right(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_right(&text);
            //if selection.is_extended(edit::selection::CURSOR_SEMANTICS){
            //    selection.collapse(&text, edit::selection::CURSOR_SEMANTICS);
            //}
            //*selection = edit::selection::Selection::move_horizontally(selection.clone(), 1, &text, edit::selection::Movement::Move, edit::selection::Direction::Forward)
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_up(&mut self){
        let text = self.document.text().clone();

        //self.document.selections_mut().move_cursors_up(&text);
        for selection in self.document.selections_mut().iter_mut(){
            selection.move_up(&text);
        }

        let selections = self.document.selections().clone();
        if self.document.view_mut().scroll_following_cursor(&selections, &text){
            self.ui.set_text_in_view(self.document.view().text(&text));
            self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
            self.ui.set_document_modified(self.document.is_modified());
        }else{
            self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
            self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        }
    }
    fn move_cursor_word_end(&mut self){}
    fn move_cursor_word_start(&mut self){}
    fn no_op(&mut self){}
    fn open_new_terminal_window(&mut self){
        //open new terminal window at current working directory
        std::process::Command::new("alacritty")
        .spawn()
        .expect("failed to spawn new terminal at current directory");
    }
    fn quit(&mut self){
        if self.ui.document_modified(){
            self.mode = Mode::Warning(WarningKind::FileIsModified);
        }else{
            self.should_quit = true;
        }
    }
    fn quit_ignoring_changes(&mut self){
        self.should_quit = true;
    }
    fn resize(&mut self, x: u16, y: u16){
        self.ui.set_terminal_size(x, y);
        self.ui.update_layouts(self.mode);
        self.ui.util_bar_mut().scroll();
        self.ui.util_bar_alternate_mut().scroll();
                
        self.document.view_mut().set_size(self.ui.document_rect().width as usize, self.ui.document_rect().height as usize);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        self.document.view_mut().scroll_following_cursor(&selections, &text);

        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn save(&mut self){
        match self.document.save(){
            Ok(_) => {
                let text = self.document.text();
                let selections = self.document.selections();

                self.ui.set_text_in_view(self.document.view().text(text));
                self.ui.set_line_numbers_in_view(self.document.view().line_numbers(text));
                self.ui.set_client_cursor_position(self.document.view().cursor_positions(text, selections));
                self.ui.set_document_cursor_position(selections.cursor_positions(text));
                self.ui.set_document_modified(self.document.is_modified());
            }
            Err(_) => {
                self.mode = Mode::Warning(WarningKind::FileSaveFailed);
            }
        }
    }
    fn scroll_view_down(&mut self, amount: usize){
        let text = self.document.text().clone();

        self.document.view_mut().scroll_down(amount, &text);

        let selections = self.document.selections().clone();
        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn scroll_view_left(&mut self, amount: usize){
        let text = self.document.text().clone();

        self.document.view_mut().scroll_left(amount);

        let selections = self.document.selections().clone();
        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn scroll_view_right(&mut self, amount: usize){
        let text = self.document.text().clone();

        self.document.view_mut().scroll_right(amount, &text);

        let selections = self.document.selections().clone();
        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn scroll_view_up(&mut self, amount: usize){
        let text = self.document.text().clone();

        self.document.view_mut().scroll_up(amount);

        let selections = self.document.selections().clone();
        self.ui.set_text_in_view(self.document.view().text(&text));
        self.ui.set_line_numbers_in_view(self.document.view().line_numbers(&text));
        self.ui.set_client_cursor_position(self.document.view().cursor_positions(&text, &selections));
        self.ui.set_document_cursor_position(selections.cursor_positions(&text));
        self.ui.set_document_modified(self.document.is_modified());
    }
    fn select_all(&mut self){
        let text = self.document.text().clone();

        self.document.selections_mut().clear_non_primary_selections();
        self.document.selections_mut().first_mut().select_all(&text);
    }
    fn set_mode_command(&mut self){
        self.mode = Mode::Command;
    }
    fn set_mode_find_replace(&mut self){
        self.mode = Mode::FindReplace;
    }
    fn set_mode_goto(&mut self){
        self.mode = Mode::Goto;
    }
    fn warning_mode_exit(&mut self){
        self.mode = Mode::Insert;
    }

    pub fn restore_terminal(&mut self) -> Result<(), Box<dyn Error>>{
        restore_terminal(&mut self.host_terminal, self.supports_keyboard_enhancement)
    }
}

fn setup_terminal() -> Result<(Terminal<CrosstermBackend<std::io::Stdout>>, bool), Box<dyn Error>>{
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    stdout.execute(CURSOR_STYLE)?;
    
    let supports_keyboard_enhancement = terminal::supports_keyboard_enhancement().unwrap_or(false);

    // only allow terminals with enhanced kb protocol support?
    //if !supports_keyboard_enhancement{
    //    panic!("this terminal does not support enhanced keyboard protocols")
    //}
    //
    
    if supports_keyboard_enhancement {
        use event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
        execute!(
            stdout, 
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                //| KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                //| KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                //| KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
    }

    let terminal = Terminal::new(
        CrosstermBackend::new(stdout)
    )?;

    Ok((terminal, supports_keyboard_enhancement))
}

pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, 
    supports_keyboard_enhancement: bool
) -> Result<(), Box<dyn Error>>{
    if supports_keyboard_enhancement{
        terminal.backend_mut().execute(event::PopKeyboardEnhancementFlags)?;
    }
    terminal::disable_raw_mode()?;
    terminal.backend_mut().execute(crossterm::terminal::LeaveAlternateScreen)?;
    terminal.backend_mut().execute(crossterm::cursor::SetCursorStyle::DefaultUserShape)?;
    terminal.show_cursor()?;
    
    Ok(())
}

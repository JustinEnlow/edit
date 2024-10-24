use std::error::Error;
use std::path::PathBuf;
use crossterm::cursor;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::ui::UserInterface;
use edit_core::selection::{CursorSemantics, Movement, Selections};
use edit_core::document::Document;



// users preferred cursor style
    // Options:
        // DefaultUserShape
        // BlinkingBLock    //inform crossterm of capital L in 'Block'
        // SteadyBlock
        // BlinkingUnderScore
        // SteadyUnderScore
        // BlinkingBar
        // SteadyBar
pub const CURSOR_STYLE: cursor::SetCursorStyle = cursor::SetCursorStyle::SteadyBlock;
const CURSOR_SEMANTICS: CursorSemantics = match CURSOR_STYLE{
    cursor::SetCursorStyle::BlinkingBar | cursor::SetCursorStyle::SteadyBar => CursorSemantics::Bar,
    _ => CursorSemantics::Block
};
const VIEW_SCROLL_AMOUNT: usize = 1;



#[derive(Clone, Copy, PartialEq)]
pub enum Mode{
    Insert, //Insert(Normal, Completion, Selection, etc)
    Space,
    Utility(UtilityKind),
}

#[derive(Clone, Copy, PartialEq)]
pub enum UtilityKind{
    Warning(WarningKind),
    Command,
    FindReplace,    //FindReplace mode can prob be replaced with Find mode, which upon acceptance would select all instances of the entered text, that can then be edited normally
    Goto,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WarningKind{
    FileIsModified,
    FileSaveFailed,
    CommandParseFailed,
    SingleSelection,
    MultipleSelections,
    InvalidInput,
}



pub struct Application{
    should_quit: bool,
    mode: Mode,
    document: Document,
    ui: UserInterface,
}
impl Application{
    pub fn new(terminal: &Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<Self, Box<dyn Error>>{
        Ok(Self{
            should_quit: false,
            mode: Mode::Insert,
            document: Document::new(CURSOR_SEMANTICS),
            ui: UserInterface::new(terminal.size()?),
        })
    }

    pub fn run(&mut self, file_path: String, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn Error>>{
        let path = PathBuf::from(file_path).canonicalize()?;
        
        self.document = Document::open(&path, CURSOR_SEMANTICS)?;
        //self.ui.file_name_widget_mut().set(self.document.file_name());
        self.ui.status_bar_mut().file_name_widget_mut().set(self.document.file_name());
        //self.ui.document_widget_mut().set_length(self.document.len());
        self.ui.document_viewport_mut().document_widget_mut().set_length(self.document.len());
        
        self.ui.update_layouts(self.mode);
        
        // init doc view size
        self.document.view_mut().set_size(
            //self.ui.document_widget().rect().width as usize, 
            self.ui.document_viewport().document_widget().rect().width as usize,
            //self.ui.document_widget().rect().height as usize
            self.ui.document_viewport().document_widget().rect().height as usize
        );

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui(&text, &selections);

        loop{
            //terminal.hide_cursor()?;    //testing this to resolve cursor displaying in random places while moving quickly
            self.ui.update_layouts(self.mode);
            self.ui.render(terminal, self.mode)?;
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
                    //(KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Esc, ..}, Mode::Insert(_)) => {Action::SomeAction}   //catch all for insert sub modes
                    (KeyEvent{modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT, code: KeyCode::PageDown, ..}, Mode::Insert) => {self.extend_selection_page_down()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT, code: KeyCode::PageUp, ..}, Mode::Insert) => {self.extend_selection_page_up()}
                    //(KeyEvent{modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT, code: KeyCode::Char('z'), ..}, Mode::Insert) => {self.redo()}   //idk why this won't work...
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('r'),     ..}, Mode::Insert) => {self.redo()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('z'),     ..}, Mode::Insert) => {self.undo()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Right,         ..}, Mode::Insert) => {self.move_cursor_word_end()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Left,          ..}, Mode::Insert) => {self.move_cursor_word_start()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Home,          ..}, Mode::Insert) => {self.move_cursor_document_start()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::End,           ..}, Mode::Insert) => {self.move_cursor_document_end()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Esc,           ..}, Mode::Insert) => {self.clear_non_primary_selections()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('p'),     ..}, Mode::Insert) => {self.increment_primary_selection()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char(' '),     ..}, Mode::Insert) => {self.set_mode_space()}  //show context menu
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('q'),     ..}, Mode::Insert) => {self.quit()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('s'),     ..}, Mode::Insert) => {self.save()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('g'),     ..}, Mode::Insert) => {self.set_mode_goto()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('f'),     ..}, Mode::Insert) => {self.set_mode_find_replace()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('l'),     ..}, Mode::Insert) => {self.display_line_numbers()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('k'),     ..}, Mode::Insert) => {self.display_status_bar()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('o'),     ..}, Mode::Insert) => {self.set_mode_command()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('t'),     ..}, Mode::Insert) => {self.open_new_terminal_window()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('a'),     ..}, Mode::Insert) => {self.select_all()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('x'),     ..}, Mode::Insert) => {self.cut()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('c'),     ..}, Mode::Insert) => {self.copy()}
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('v'),     ..}, Mode::Insert) => {self.paste()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Right,         ..}, Mode::Insert) => {self.extend_selection_right()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Left,          ..}, Mode::Insert) => {self.extend_selection_left()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Up,            ..}, Mode::Insert) => {self.extend_selection_up()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Down,          ..}, Mode::Insert) => {self.extend_selection_down()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Home,          ..}, Mode::Insert) => {self.extend_selection_home()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::End,           ..}, Mode::Insert) => {self.extend_selection_end()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT,   code: KeyCode::Char(c),       ..}, Mode::Insert) => {self.insert_char(c)}
                    (KeyEvent{modifiers: KeyModifiers::ALT,     code: KeyCode::Down,          ..}, Mode::Insert) => {self.scroll_view_down(VIEW_SCROLL_AMOUNT)}
                    (KeyEvent{modifiers: KeyModifiers::ALT,     code: KeyCode::Left,          ..}, Mode::Insert) => {self.scroll_view_left(VIEW_SCROLL_AMOUNT)}
                    (KeyEvent{modifiers: KeyModifiers::ALT,     code: KeyCode::Right,         ..}, Mode::Insert) => {self.scroll_view_right(VIEW_SCROLL_AMOUNT)}
                    (KeyEvent{modifiers: KeyModifiers::ALT,     code: KeyCode::Up,            ..}, Mode::Insert) => {self.scroll_view_up(VIEW_SCROLL_AMOUNT)}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Tab,           ..}, Mode::Insert) => {self.insert_tab()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Enter,         ..}, Mode::Insert) => {self.insert_newline()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Delete,        ..}, Mode::Insert) => {self.delete()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Backspace,     ..}, Mode::Insert) => {self.backspace()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Up,            ..}, Mode::Insert) => {self.move_cursor_up()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Down,          ..}, Mode::Insert) => {self.move_cursor_down()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Left,          ..}, Mode::Insert) => {self.move_cursor_left()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Right,         ..}, Mode::Insert) => {self.move_cursor_right()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::PageUp,        ..}, Mode::Insert) => {self.move_cursor_page_up()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::PageDown,      ..}, Mode::Insert) => {self.move_cursor_page_down()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Home,          ..}, Mode::Insert) => {self.move_cursor_line_start()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::End,           ..}, Mode::Insert) => {self.move_cursor_line_end()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Esc,           ..}, Mode::Insert) => {self.collapse_selections()}   //depending on context: close suggestions, close context menu, collapse selections, clear non-primary selections
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Char(c), ..}, Mode::Insert) => {self.insert_char(c)}

                    // Space Mode
                    (KeyEvent{modifiers: KeyModifiers::NONE, code: KeyCode::Esc, ..}, Mode::Space) => {self.space_mode_exit()}
                    (KeyEvent{modifiers: KeyModifiers::NONE, code: KeyCode::Char('c'), ..}, Mode::Space) => {self.center_view_vertically_around_cursor()}
                    (KeyEvent{modifiers: KeyModifiers::NONE, code: KeyCode::Char('p'), ..}, Mode::Space) => {self.increment_primary_selection()}

                    // Warning Mode
                    (KeyEvent{modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('q'), ..}, Mode::Utility(UtilityKind::Warning(_))) => {self.quit_ignoring_changes()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,    code: KeyCode::Esc,       ..}, Mode::Utility(UtilityKind::Warning(_))) => {self.warning_mode_exit()}
    
                    // Goto Mode
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Right,         ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_extend_selection_right()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Left,          ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_extend_selection_left()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Home,          ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_extend_selection_home()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::End,           ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_extend_selection_end()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Esc,           ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_exit()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Enter,         ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_accept()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Backspace,     ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_backspace()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Delete,        ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_delete()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Right,         ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_move_cursor_right()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Left,          ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_move_cursor_left()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Home,          ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_move_cursor_line_start()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::End,           ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_move_cursor_line_end()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Char(c), ..}, Mode::Utility(UtilityKind::Goto)) => {self.goto_mode_insert_char(c)}
                
                    // FindReplace Mode
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Right,         ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_extend_selection_right()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Left,          ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_extend_selection_left()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Home,          ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_extend_selection_home()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::End,           ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_extend_selection_end()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Char(c), ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_insert_char(c)}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Esc,           ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_exit()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Tab,           ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_switch_util_bar_focus()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Up,            ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_previous_instance()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Down,          ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_next_instance()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Backspace,     ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_backspace()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Delete,        ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_delete()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Right,         ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_move_cursor_right()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Left,          ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_move_cursor_left()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Home,          ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_move_cursor_line_start()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::End,           ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_move_cursor_line_end()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Enter,         ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_accept()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Char(c), ..}, Mode::Utility(UtilityKind::FindReplace)) => {self.find_replace_mode_insert_char(c)}
                
                    // Command Mode
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Right,         ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_extend_selection_right()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Left,          ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_extend_selection_left()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Home,          ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_extend_selection_home()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::End,           ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_extend_selection_end()}
                    (KeyEvent{modifiers: KeyModifiers::SHIFT, code: KeyCode::Char(c), ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_insert_char(c)}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Esc,           ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_exit()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Char(c), ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_insert_char(c)}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Enter,         ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_accept()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Backspace,     ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_backspace()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Delete,        ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_delete()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Right,         ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_move_cursor_right()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Left,          ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_move_cursor_left()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::Home,          ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_move_cursor_line_start()}
                    (KeyEvent{modifiers: KeyModifiers::NONE,  code: KeyCode::End,           ..}, Mode::Utility(UtilityKind::Command)) => {self.command_mode_move_cursor_line_end()}
    
                    // unhandled keybinds
                    _ => {self.no_op()}
                }
            },
            event::Event::Resize(x, y) => self.resize(x, y),
            _ => self.no_op(),
        }

        Ok(())
    }

    fn update_ui(&mut self, text: &ropey::Rope, selections: &Selections){
        self.ui.document_viewport_mut().document_widget_mut().set_text_in_view(self.document.view().text(text));
        self.ui.document_viewport_mut().line_number_widget_mut().set(self.document.view().line_numbers(text));
        self.ui.document_viewport_mut().document_widget_mut().set_client_cursor_position(self.document.view().cursor_positions(text, selections, CURSOR_SEMANTICS));
        self.ui.status_bar_mut().document_cursor_position_widget_mut().set(selections.cursor_positions(text, CURSOR_SEMANTICS));
        self.ui.status_bar_mut().modified_indicator_widget_mut().set(self.document.is_modified());
    }

    fn update_cursor_positions(&mut self, text: &ropey::Rope, selections: &Selections){
        //self.ui.document_widget_mut().set_client_cursor_position(self.document.view().cursor_positions(text, selections, CURSOR_SEMANTICS));
        self.ui.document_viewport_mut().document_widget_mut().set_client_cursor_position(self.document.view().cursor_positions(text, selections, CURSOR_SEMANTICS));
        //self.ui.document_cursor_position_widget_mut().set(selections.cursor_positions(text, CURSOR_SEMANTICS));
        self.ui.status_bar_mut().document_cursor_position_widget_mut().set(selections.cursor_positions(text, CURSOR_SEMANTICS));
    }

    //fn add_selection_above(&mut self){}
    //fn add_selection_below(&mut self){}
    fn backspace(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        self.document.backspace(CURSOR_SEMANTICS);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui(&text, &selections);

        if len != self.document.len(){  //if length has changed after backspace
            //self.ui.document_widget_mut().set_length(self.document.len());
            self.ui.document_viewport_mut().document_widget_mut().set_length(self.document.len());
        }
    }
    fn center_view_vertically_around_cursor(&mut self){
        assert!(self.mode == Mode::Space);
        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        let selection = selections.primary();
        *self.document.view_mut() = self.document.view().center_vertically_around_cursor(selection, &text, CURSOR_SEMANTICS);
        self.update_ui(&text, &selections);
        //exit space mode
        self.mode = Mode::Insert;
    }
    fn clear_non_primary_selections(&mut self){
        assert!(self.mode == Mode::Insert);
        if self.document.selections().count() > 1{
            self.document.selections_mut().clear_non_primary_selections();

            let text = self.document.text().clone();
            let selections = self.document.selections().clone();
            if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
                *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
                self.update_ui(&text, &selections);
            }else{
                self.update_cursor_positions(&text, &selections);
            }
        }else{
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::SingleSelection));
        }
    }
    fn collapse_selections(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.collapse(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn command_mode_accept(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        if self.parse_command(&self.ui.util_bar().utility_widget().text_box().text().to_string()).is_ok(){
            self.command_mode_exit();
        }else{
            self.command_mode_exit();
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::CommandParseFailed));
        }
        //ui.scroll(editor);
    }
    fn command_mode_backspace(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().backspace();
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_delete(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().delete();
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_exit(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().clear();
        self.mode = Mode::Insert;
    }
    fn command_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_line_text_end(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_home(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_left(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_right(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().insert_char(c);
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_left(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_line_text_end(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_home(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn command_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_right(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn copy(&mut self){
        assert!(self.mode == Mode::Insert);
        if self.document.selections().count() == 1{
            self.document.copy();
        }else{
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::MultipleSelections));
        }
    }
    fn cut(&mut self){
        assert!(self.mode == Mode::Insert);
        if self.document.selections().count() == 1{
            let len = self.document.len();
            self.document.cut(CURSOR_SEMANTICS);
            //TODO: should document widget length be set here instead of later?

            let text = self.document.text().clone();
            let selections = self.document.selections().clone();
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

            self.update_ui(&text, &selections);

            if len != self.document.len(){  //if length has changed after cut
                self.ui.document_viewport_mut().document_widget_mut().set_length(self.document.len());
            }
        }else{
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::MultipleSelections));
        }
    }
    fn delete(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        self.document.delete(CURSOR_SEMANTICS);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui(&text, &selections);

        if len != self.document.len(){  //if length has changed after delete
            self.ui.document_viewport_mut().document_widget_mut().set_length(self.document.len());
        }
    }
    fn display_line_numbers(&mut self){
        assert!(self.mode == Mode::Insert);
        self.ui.document_viewport_mut().toggle_line_numbers();
                
        self.ui.update_layouts(self.mode);

        self.document.view_mut().set_size(
            self.ui.document_viewport().document_widget().rect().width as usize,
            self.ui.document_viewport().document_widget().rect().height as usize
        );
                
        let text = self.document.text().clone();
        let selections = self.document.selections().clone();

        self.update_ui(&text, &selections);
    }
    fn display_status_bar(&mut self){
        assert!(self.mode == Mode::Insert);
        self.ui.status_bar_mut().toggle_status_bar();
                
        self.ui.update_layouts(self.mode);

        self.document.view_mut().set_size(
            self.ui.document_viewport().document_widget().rect().width as usize,
            self.ui.document_viewport().document_widget().rect().height as usize
        );

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();

        self.update_ui(&text, &selections);
    }
    fn extend_selection_down(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.extend_down(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn extend_selection_end(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.extend_line_text_end(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn extend_selection_home(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.extend_home(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn extend_selection_left(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.extend_left(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn extend_selection_page_up(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.extend_page_up(&text, &view, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn extend_selection_page_down(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.extend_page_down(&text, &view, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn extend_selection_right(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.extend_right(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn extend_selection_up(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.extend_up(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn find_replace_mode_accept(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
    }
    fn find_replace_mode_backspace(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().backspace();
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            self.ui.util_bar_mut().utility_widget_mut().text_box_mut().backspace();
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }

        self.find_replace_mode_text_validity_check();
    }
    fn find_replace_mode_delete(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().delete();
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            self.ui.util_bar_mut().utility_widget_mut().text_box_mut().delete();
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }

        self.find_replace_mode_text_validity_check();
    }
    fn find_replace_mode_exit(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().clear();
        self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().clear();
        self.ui.util_bar_mut().set_alternate_focused(false);
        self.mode = Mode::Insert;
    }
    fn find_replace_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().alternate_utility_widget().text_box().selection().extend_line_text_end(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_line_text_end(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
    }
    fn find_replace_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().alternate_utility_widget().text_box().selection().extend_home(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_home(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
    }
    fn find_replace_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().alternate_utility_widget().text_box().selection().extend_left(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_left(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
    }
    fn find_replace_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().alternate_utility_widget().text_box().selection().extend_right(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_right(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
    }
    fn find_replace_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().insert_char(c);
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            self.ui.util_bar_mut().utility_widget_mut().text_box_mut().insert_char(c);
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
        
        self.find_replace_mode_text_validity_check();
    }
    fn find_replace_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().alternate_utility_widget().text_box().selection().move_left(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_left(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
    }
    fn find_replace_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().alternate_utility_widget().text_box().selection().move_line_text_end(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_line_text_end(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
    }
    fn find_replace_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().alternate_utility_widget().text_box().selection().move_home(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_home(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
    }
    fn find_replace_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar().alternate_focused(){
            let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().alternate_utility_widget().text_box().selection().move_right(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }else{
            let text = self.ui.util_bar().utility_widget().text_box().text().clone();
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_right(&text, CURSOR_SEMANTICS);
            let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
            *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        }
    }
    fn find_replace_mode_next_instance(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
    }
    fn find_replace_mode_previous_instance(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
    }
    fn find_replace_mode_switch_util_bar_focus(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        let toggle = !self.ui.util_bar().alternate_focused();
        self.ui.util_bar_mut().set_alternate_focused(toggle);
    }
    fn find_replace_mode_text_validity_check(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        //run text validity check
        if !self.document.text().clone().to_string().contains(&self.ui.util_bar().utility_widget().text_box().text().to_string()){
            self.ui.util_bar_mut().utility_widget_mut().text_box_mut().set_text_is_valid(false);
        }else{
            self.ui.util_bar_mut().utility_widget_mut().text_box_mut().set_text_is_valid(true);
        }
    }
    fn goto_mode_accept(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        if let Ok(line_number) = self.ui.util_bar().utility_widget().text_box().text().to_string().parse::<usize>(){
            let line_number = line_number.saturating_sub(1);

            //if line_number < self.ui.document_length(){
            if line_number < self.document.len(){   //&& line_number > 0
                let text =  self.document.text().clone();
                
                if self.document.selections().count() > 1{
                    self.document.selections_mut().clear_non_primary_selections();
                }
                *self.document.selections_mut().primary_mut() = self.document.selections().primary().set_from_line_number(line_number, &text, Movement::Move, CURSOR_SEMANTICS);
                
                let selections = self.document.selections().clone();
                *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
                self.update_ui(&text, &selections);
                
                self.goto_mode_exit();
            }else{
                self.goto_mode_exit();
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));
            }
        }else{
            self.ui.util_bar_mut().utility_widget_mut().text_box_mut().clear();
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));
        }
    }
    fn goto_mode_backspace(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().backspace();
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    
        self.goto_mode_text_validity_check();
    }
    fn goto_mode_delete(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().delete();
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    
        self.goto_mode_text_validity_check();
    }
    fn goto_mode_exit(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().clear();
        self.mode = Mode::Insert;
    }
    fn goto_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_line_text_end(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn goto_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_home(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn goto_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_left(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn goto_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().extend_right(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn goto_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar_mut().utility_widget_mut().text_box_mut().insert_char(c);
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    
        self.goto_mode_text_validity_check();
    }
    fn goto_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_left(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn goto_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_line_text_end(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn goto_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_home(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn goto_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().selection_mut() = self.ui.util_bar().utility_widget().text_box().selection().move_right(&text, CURSOR_SEMANTICS);
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn goto_mode_text_validity_check(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        // run text validity check
        let mut is_numeric = true;
        for grapheme in self.ui.util_bar().utility_widget().text_box().text().chars(){ // .graphemes(true)?
            if !grapheme.is_ascii_digit(){
                is_numeric = false;
            }
        }
        let exceeds_doc_length = match self.ui.util_bar().utility_widget().text_box().text().to_string().parse::<usize>(){
            Ok(line_number) => {
                //line_number > self.ui.document_length()
                line_number > self.document.len()
            }
            Err(_) => false
        };
        if !is_numeric || exceeds_doc_length{
            self.ui.util_bar_mut().utility_widget_mut().text_box_mut().set_text_is_valid(false);
        }else{
            self.ui.util_bar_mut().utility_widget_mut().text_box_mut().set_text_is_valid(true);
        }
    }
    fn increment_primary_selection(&mut self){
        if self.document.selections().count() > 1{
            *self.document.selections_mut() = self.document.selections().increment_primary_selection();

            let text = self.document.text().clone();
            let selections = self.document.selections().clone();
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

            self.update_ui(&text, &selections);
        }else{
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::SingleSelection))
        }
    }
    fn insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Insert);
        //self.document.insert_char(c, CURSOR_SEMANTICS);
        self.document.insert_string(&c.to_string(), CURSOR_SEMANTICS);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui(&text, &selections);
    }
    fn insert_newline(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        //self.document.enter(CURSOR_SEMANTICS);
        self.document.insert_string("\n", CURSOR_SEMANTICS);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui(&text, &selections);

        if len != self.document.len(){  //if length has changed after newline
            //self.ui.document_widget_mut().set_length(self.document.len());
            self.ui.document_viewport_mut().document_widget_mut().set_length(self.document.len());
        }
    }
    fn insert_tab(&mut self){
        assert!(self.mode == Mode::Insert);
        //self.document.tab(CURSOR_SEMANTICS);
        self.document.insert_string("\t", CURSOR_SEMANTICS);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui(&text, &selections);
    }
    fn move_cursor_document_end(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        if self.document.selections().count() > 1{
            self.document.selections_mut().clear_non_primary_selections();
        }
        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_doc_end(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_document_start(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        if self.document.selections().count() > 1{
            self.document.selections_mut().clear_non_primary_selections();
        }
        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_doc_start(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_down(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_down(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_left(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_left(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_line_text_end(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_home(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_page_down(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_page_down(&text, &view, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_page_up(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_page_up(&text, &view, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_right(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_right(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_up(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        for selection in self.document.selections_mut().iter_mut(){
            *selection = selection.move_up(&text, CURSOR_SEMANTICS);
        }

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn move_cursor_word_end(&mut self){
        assert!(self.mode == Mode::Insert);
    }
    fn move_cursor_word_start(&mut self){
        assert!(self.mode == Mode::Insert);
    }
    fn no_op(&mut self){}
    fn open_new_terminal_window(&self){
        //open new terminal window at current working directory
        let _ = std::process::Command::new("alacritty")
            .spawn()
            .expect("failed to spawn new terminal at current directory");
    }
    pub fn parse_command(&self, args: &str) -> Result<(), ()>{
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        let mut args = args.split_whitespace();
        
        let command = args.next().unwrap();
        match command{
            "term" => {
//                // open new terminal window at current directory.. TODO: fix this closes child when parent closes
//                //command: alacritty --working-directory $PWD
//                // does this work with $TERM when $TERM isn't alacritty?
//                std::process::Command::new("alacritty")
//                //Command::new("$TERM") //this causes a panic
//                    //not needed here, because term spawned here defaults to this directory, but good to know
//                    //.current_dir("/home/j/Documents/programming/rust/nlo_text_editor/")
//                    //.output() // output keeps current process from working until child process closes
//                    .spawn()
//                    .expect("failed to spawn new terminal at current directory");
                self.open_new_terminal_window();
            }
            _ => {return Err(())}
        }
    
        Ok(())
    }
    fn paste(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        self.document.paste(CURSOR_SEMANTICS);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui(&text, &selections);

        if len != self.document.len(){  //if length has changed after paste
            //self.ui.document_widget_mut().set_length(self.document.len());
            self.ui.document_viewport_mut().document_widget_mut().set_length(self.document.len());
        }
    }
    fn quit(&mut self){
        assert!(self.mode == Mode::Insert);
        //if self.ui.document_modified(){
        if self.document.is_modified(){
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::FileIsModified));
        }else{
            self.should_quit = true;
        }
    }
    fn quit_ignoring_changes(&mut self){
        self.should_quit = true;
    }
    fn redo(&mut self){
        assert!(self.mode == Mode::Insert);

        if let Ok(_) = self.document.redo(CURSOR_SEMANTICS){
            let len = self.document.len();
            let text = self.document.text().clone();
            let selections = self.document.selections().clone();
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

            self.update_ui(&text, &selections);

            if len != self.document.len(){  //if length has changed after paste
                //self.ui.document_widget_mut().set_length(self.document.len());
                self.ui.document_viewport_mut().document_widget_mut().set_length(self.document.len());
            }
        }else{
            // warn redo stack empty
        }
    }
    fn resize(&mut self, x: u16, y: u16){
        self.ui.set_terminal_size(x, y);
        self.ui.update_layouts(self.mode);

        let text = self.ui.util_bar().utility_widget().text_box().text().clone();
        let selections = Selections::new(vec![self.ui.util_bar().utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        let text = self.ui.util_bar().alternate_utility_widget().text_box().text().clone();
        let selections = Selections::new(vec![self.ui.util_bar().alternate_utility_widget().text_box().selection().clone()], 0, &text);
        *self.ui.util_bar_mut().alternate_utility_widget_mut().text_box_mut().view_mut() = self.ui.util_bar().alternate_utility_widget().text_box().view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        //self.document.view_mut().set_size(self.ui.document_widget().rect().width as usize, self.ui.document_widget().rect().height as usize);
        self.document.view_mut().set_size(self.ui.document_viewport().document_widget().rect().width as usize, self.ui.document_viewport().document_widget().rect().height as usize);

        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui(&text, &selections);
    }
    fn save(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.save(){
            Ok(_) => {
                let text = self.document.text().clone();
                let selections = self.document.selections().clone();

                self.update_ui(&text, &selections);
            }
            Err(_) => {
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::FileSaveFailed));
            }
        }
    }
    fn scroll_view_down(&mut self, amount: usize){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        *self.document.view_mut() = self.document.view().scroll_down(amount, &text);

        let selections = self.document.selections().clone();
        self.update_ui(&text, &selections);
    }
    fn scroll_view_left(&mut self, amount: usize){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        *self.document.view_mut() = self.document.view().scroll_left(amount);

        let selections = self.document.selections().clone();
        self.update_ui(&text, &selections);
    }
    fn scroll_view_right(&mut self, amount: usize){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        *self.document.view_mut() = self.document.view().scroll_right(amount, &text);

        let selections = self.document.selections().clone();
        self.update_ui(&text, &selections);
    }
    fn scroll_view_up(&mut self, amount: usize){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        *self.document.view_mut() = self.document.view().scroll_up(amount);

        let selections = self.document.selections().clone();
        self.update_ui(&text, &selections);
    }
    fn select_all(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        if self.document.selections().count() > 1{
            self.document.selections_mut().clear_non_primary_selections();
        }
        *self.document.selections_mut().primary_mut() = self.document.selections().primary().select_all(&text, CURSOR_SEMANTICS);

        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui(&text, &selections);
        }else{
            self.update_cursor_positions(&text, &selections);
        }
    }
    fn set_mode_command(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Utility(UtilityKind::Command);
    }
    fn set_mode_find_replace(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Utility(UtilityKind::FindReplace);
    }
    fn set_mode_goto(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Utility(UtilityKind::Goto);
    }
    fn set_mode_space(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Space;
    }
    fn space_mode_exit(&mut self){
        assert!(self.mode == Mode::Space);
        self.mode = Mode::Insert;
    }
    fn undo(&mut self){
        assert!(self.mode == Mode::Insert);

        if let Ok(_) = self.document.undo(CURSOR_SEMANTICS){
            let len = self.document.len();
            let text = self.document.text().clone();
            let selections = self.document.selections().clone();
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

            self.update_ui(&text, &selections);

            if len != self.document.len(){  //if length has changed after paste
                //self.ui.document_widget_mut().set_length(self.document.len());
                self.ui.document_viewport_mut().document_widget_mut().set_length(self.document.len());
            }
        }else{
            // warn undo stack empty
        }
    }
    fn warning_mode_exit(&mut self){
        // assert warning mode
        self.mode = Mode::Insert;
    }
}

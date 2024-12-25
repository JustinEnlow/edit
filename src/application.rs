use std::error::Error;
use std::path::PathBuf;
use crossterm::cursor;
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::ui::UserInterface;
use edit_core::selection::{CursorSemantics, Movement, Selection, Selections, SelectionError, SelectionsError};
use edit_core::view::View;
use edit_core::document::Document;
use ropey::Rope;



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
const VIEW_SCROLL_AMOUNT: usize = 1;    //should this have separate vertical and horizontal definitions?



enum ScrollDirection{
    Down,
    Left,
    Right,
    Up
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode{
    Insert,
    Space,
    Utility(UtilityKind),   //this actually may be better as separate Modes so that adding/removing to them can be easier...
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
        let terminal_size = terminal.size()?;
        let terminal_rect = Rect::new(0, 0, terminal_size.width, terminal_size.height);
        Ok(Self{
            should_quit: false,
            mode: Mode::Insert,
            document: Document::new(CURSOR_SEMANTICS),
            ui: UserInterface::new(terminal_rect),
        })
    }

    pub fn run(&mut self, file_path: String, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn Error>>{
        let path = PathBuf::from(file_path).canonicalize()?;
        
        self.document = Document::open(&path, CURSOR_SEMANTICS)?;
        self.ui.status_bar.file_name_widget.file_name = self.document.file_name();
        self.ui.document_viewport.document_widget.doc_length = self.document.len();
        
        self.ui.update_layouts(self.mode);
        
        // init doc view size
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.scroll_and_update();

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

    //TODO: maybe make a keybind.rs for these next several fns?
    fn handle_insert_mode_keypress(&mut self, keycode: KeyCode, modifiers: KeyModifiers){
        match (keycode, modifiers){
            (KeyCode::Char(c), modifiers) => {
                if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){
                    if c == 'p'{self.decrement_primary_selection();}
                    if c == 'z'{self.redo();}
                }
                else if modifiers == KeyModifiers::CONTROL{
                    if c == ' '{self.set_mode_space();}
                    if c == 'q'{self.quit();}
                    if c == 's'{self.save();}
                    if c == 'g'{self.set_mode_goto();}
                    if c == 'f'{self.set_mode_find_replace();}
                    if c == 'l'{self.display_line_numbers();}
                    if c == 'k'{self.display_status_bar();}
                    if c == 'o'{self.set_mode_command();}
                    if c == 't'{self.open_new_terminal_window();}
                    if c == 'a'{self.select_all();}
                    if c == 'x'{self.cut();}
                    if c == 'c'{self.copy();}
                    if c == 'v'{self.paste();}
                    if c == 'p'{self.increment_primary_selection();}
                    if c == 'z'{self.undo();}
                }
                else if modifiers == KeyModifiers::SHIFT{self.insert_char(c);}
                else if modifiers == KeyModifiers::NONE{self.insert_char(c);}
                else{self.no_op();}
            }
            (KeyCode::PageDown, modifiers) => {
                if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){self.extend_selection_page_down();}
                else if modifiers == KeyModifiers::NONE{self.move_cursor_page_down();}
                else{self.no_op();}
            }
            (KeyCode::PageUp, modifiers) => {
                if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){self.extend_selection_page_up();}
                else if modifiers == KeyModifiers::NONE{self.move_cursor_page_up();}
                else{self.no_op();}
            }
            (KeyCode::Up, modifiers) => {
                if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){self.add_selection_above();}
                else if modifiers == KeyModifiers::SHIFT{self.extend_selection_up();}
                else if modifiers == KeyModifiers::ALT{self.scroll_view_up(VIEW_SCROLL_AMOUNT);}
                else if modifiers == KeyModifiers::NONE{self.move_cursor_up();}
                else{self.no_op();}
            }
            (KeyCode::Down, modifiers) => {
                if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){self.add_selection_below();}
                else if modifiers == KeyModifiers::SHIFT{self.extend_selection_down();}
                else if modifiers == KeyModifiers::ALT{self.scroll_view_down(VIEW_SCROLL_AMOUNT);}
                else if modifiers == KeyModifiers::NONE{self.move_cursor_down();}
                else{self.no_op();}
            }
            (KeyCode::Home, modifiers) => {
                if modifiers == KeyModifiers::CONTROL{self.move_cursor_document_start();}
                else if modifiers == KeyModifiers::SHIFT{self.extend_selection_home();}
                else if modifiers == KeyModifiers::NONE{self.move_cursor_line_start();}
                else{self.no_op();}
            }
            (KeyCode::End, modifiers) => {
                if modifiers == KeyModifiers::CONTROL{self.move_cursor_document_end();}
                else if modifiers == KeyModifiers::SHIFT{self.extend_selection_end();}
                else if modifiers == KeyModifiers::NONE{self.move_cursor_line_end();}
                else{self.no_op();}
            }
            (KeyCode::Right, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.extend_selection_right();}
                else if modifiers == KeyModifiers::ALT{self.scroll_view_right(VIEW_SCROLL_AMOUNT);}
                else if modifiers == KeyModifiers::NONE{self.move_cursor_right();}
                else{self.no_op();}
            }
            (KeyCode::Left, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.extend_selection_left();}
                else if modifiers == KeyModifiers::ALT{self.scroll_view_left(VIEW_SCROLL_AMOUNT);}
                else if modifiers == KeyModifiers::NONE{self.move_cursor_left();}
                else{self.no_op();}
            }
            (KeyCode::Tab, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.insert_tab();}
                else{self.no_op();}
            }
            (KeyCode::Enter, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.insert_newline();}
                else{self.no_op();}
            }
            (KeyCode::Delete, modifiers) => {
                if modifiers == KeyModifiers::CONTROL{/*self.delete_word_forwards();*/} //TODO: impl this functionality
                else if modifiers == KeyModifiers::NONE{self.delete();}
                else{self.no_op();}
            }
            (KeyCode::Backspace, modifiers) => {
                if modifiers == KeyModifiers::CONTROL{/*self.delete_word_backwards();*/}    //TODO: impl this functionality
                else if modifiers == KeyModifiers::NONE{self.backspace();}
                else{self.no_op();}
            }
            (KeyCode::Esc, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.esc_handle();}  //how can this be disambiguated as custom behavior vs builtin fn?
                else{self.no_op();}
            }
            _ => {self.no_op();}
        }
    }

    fn handle_space_mode_keypress(&mut self, keycode: KeyCode, modifiers: KeyModifiers){
        match (keycode, modifiers){
            (KeyCode::Esc, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.space_mode_exit();}
                else{self.no_op();}
            }
            (KeyCode::Char('c'), modifiers) => {
                if modifiers == KeyModifiers::NONE{self.center_view_vertically_around_cursor();}    //this still needs be made to exit space mode
                else{self.no_op();}
            }
            (KeyCode::Char('p'), modifiers) => {
                if modifiers == KeyModifiers::NONE{self.increment_primary_selection();}
                else{self.no_op();}
            }
            _ => {self.no_op();}
        }
    }

    fn handle_warning_mode_keypress(&mut self, keycode: KeyCode, modifiers: KeyModifiers){
        match (keycode, modifiers){
            (KeyCode::Char('q'), modifiers) => {
                if modifiers == KeyModifiers::CONTROL{self.quit_ignoring_changes();}
                else{self.no_op();}
            }
            (KeyCode::Esc, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.warning_mode_exit();}
                else{self.no_op();}
            }
            _ => {self.no_op();}
        }
    }

    fn handle_goto_mode_keypress(&mut self, keycode: KeyCode, modifiers: KeyModifiers){
        match (keycode, modifiers){
            (KeyCode::Right, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.goto_mode_extend_selection_right();}
                else if modifiers == KeyModifiers::NONE{self.goto_mode_move_cursor_right();}
                else{self.no_op();}
            }
            (KeyCode::Left, modifiers)  => {
                if modifiers == KeyModifiers::SHIFT{self.goto_mode_extend_selection_left();}
                else if modifiers == KeyModifiers::NONE{self.goto_mode_move_cursor_left();}
                else{self.no_op();}
            }
            (KeyCode::Home, modifiers)  => {
                if modifiers == KeyModifiers::SHIFT{self.goto_mode_extend_selection_home();}
                else if modifiers == KeyModifiers::NONE{self.goto_mode_move_cursor_line_start();}
                else{self.no_op();}
            }
            (KeyCode::End, modifiers)   => {
                if modifiers == KeyModifiers::SHIFT{self.goto_mode_extend_selection_end();}
                else if modifiers == KeyModifiers::NONE{self.goto_mode_move_cursor_line_end();}
                else{self.no_op();}
            }
            (KeyCode::Esc, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.goto_mode_exit();}
                else{self.no_op();}
            }
            (KeyCode::Enter, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.goto_mode_accept();}
                else{self.no_op();}
            }
            (KeyCode::Backspace, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.goto_mode_backspace();}
                else{self.no_op();}
            }
            (KeyCode::Delete, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.goto_mode_delete();}
                else{self.no_op();}
            }
            (KeyCode::Char(c), modifiers) => {
                if modifiers == KeyModifiers::NONE{self.goto_mode_insert_char(c);}
                else{self.no_op();}
            }
            _ => {self.no_op();}
        }
    }

    fn handle_find_replace_mode_keypress(&mut self, keycode: KeyCode, modifiers: KeyModifiers){
        match (keycode, modifiers){
            (KeyCode::Right, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.find_replace_mode_extend_selection_right();}
                else if modifiers == KeyModifiers::NONE{self.find_replace_mode_move_cursor_right();}
                else{self.no_op();}
            }
            (KeyCode::Left, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.find_replace_mode_extend_selection_left();}
                else if modifiers == KeyModifiers::NONE{self.find_replace_mode_move_cursor_left();}
                else{self.no_op();}
            }
            (KeyCode::Home, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.find_replace_mode_extend_selection_home();}
                else if modifiers == KeyModifiers::NONE{self.find_replace_mode_move_cursor_line_start();}
                else{self.no_op();}
            }
            (KeyCode::End, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.find_replace_mode_extend_selection_end();}
                else if modifiers == KeyModifiers::NONE{self.find_replace_mode_move_cursor_line_end();}
                else{self.no_op();}
            }
            (KeyCode::Char(c), modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.find_replace_mode_insert_char(c);}
                else if modifiers == KeyModifiers::NONE{self.find_replace_mode_insert_char(c);}
                else{self.no_op();}
            }
            (KeyCode::Esc, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.find_replace_mode_exit();}
                else{self.no_op();}
            }
            (KeyCode::Tab, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.find_replace_mode_switch_util_bar_focus();}
                else{self.no_op();}
            }
            (KeyCode::Up, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.find_replace_mode_previous_instance();}
                else{self.no_op();}
            }
            (KeyCode::Down, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.find_replace_mode_next_instance();}
                else{self.no_op();}
            }
            (KeyCode::Backspace, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.find_replace_mode_backspace();}
                else{self.no_op();}
            }
            (KeyCode::Delete, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.find_replace_mode_delete();}
                else{self.no_op();}
            }
            (KeyCode::Enter, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.find_replace_mode_accept();}
                else{self.no_op();}
            }
            _ => {self.no_op();}
        }
    }

    fn handle_command_mode_keypress(&mut self, keycode: KeyCode, modifiers: KeyModifiers){
        match (keycode, modifiers){
            (KeyCode::Char(c), modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.command_mode_insert_char(c);}
                else if modifiers == KeyModifiers::NONE{self.command_mode_insert_char(c);}
                else{self.no_op();}
            }
            (KeyCode::Right, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.command_mode_extend_selection_right();}
                else if modifiers == KeyModifiers::NONE{self.command_mode_move_cursor_right();}
                else{self.no_op();}
            }
            (KeyCode::Left, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.command_mode_extend_selection_left();}
                else if modifiers == KeyModifiers::NONE{self.command_mode_move_cursor_left();}
                else{self.no_op();}
            }
            (KeyCode::Home, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.command_mode_extend_selection_home();}
                else if modifiers == KeyModifiers::NONE{self.command_mode_move_cursor_line_start();}
                else{self.no_op();}
            }
            (KeyCode::End, modifiers) => {
                if modifiers == KeyModifiers::SHIFT{self.command_mode_extend_selection_end();}
                else if modifiers == KeyModifiers::NONE{self.command_mode_move_cursor_line_end();}
                else{self.no_op();}
            }
            (KeyCode::Esc, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.command_mode_exit();}
                else{self.no_op();}
            }
            (KeyCode::Enter, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.command_mode_accept();}
                else{self.no_op();}
            }
            (KeyCode::Backspace, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.command_mode_backspace();}
                else{self.no_op();}
            }
            (KeyCode::Delete, modifiers) => {
                if modifiers == KeyModifiers::NONE{self.command_mode_delete();}
                else{self.no_op();}
            }
            _ => {self.no_op();}
        }
    }

    fn handle_event(&mut self) -> Result<(), Box<dyn Error>>{
        match event::read()?{
            event::Event::Key(key_event) => {
                match self.mode{
                    Mode::Insert => {self.handle_insert_mode_keypress(key_event.code, key_event.modifiers);}
                    Mode::Space => {self.handle_space_mode_keypress(key_event.code, key_event.modifiers);}
                    Mode::Utility(UtilityKind::Warning(_)) => {self.handle_warning_mode_keypress(key_event.code, key_event.modifiers);}
                    Mode::Utility(UtilityKind::Goto) => {self.handle_goto_mode_keypress(key_event.code, key_event.modifiers);}
                    Mode::Utility(UtilityKind::FindReplace) => {self.handle_find_replace_mode_keypress(key_event.code, key_event.modifiers);}
                    Mode::Utility(UtilityKind::Command) => {self.handle_command_mode_keypress(key_event.code, key_event.modifiers);}
                }
            },
            event::Event::Resize(x, y) => self.resize(x, y),
            _ => self.no_op(),  //TODO: this no_op should be disambiguated from a keypress no_op, so that they can impl different behavior...
        }

        Ok(())
    }

    // could make separate files for categories of fns. builtin.rs and custom.rs...       custom::escape_handle()     builtin::add_selection_above()

    /////////////////////////////////////////////////////////////////////////// Reuse ////////////////////////////////////////////////////////////////////////////////
    fn update_ui(&mut self){
        let text = self.document.text();
        let selections = self.document.selections();
        self.ui.document_viewport.document_widget.text_in_view = self.document.view().text(text);
        self.ui.document_viewport.line_number_widget.line_numbers_in_view = self.document.view().line_numbers(text);
        self.ui.highlighter.set_primary_cursor_position(self.document.view().primary_cursor_position(text, selections, CURSOR_SEMANTICS));
        self.ui.highlighter.selections = self.document.view().selections(selections, text);
        self.ui.status_bar.document_cursor_position_widget.document_cursor_position = selections.primary().selection_to_selection2d(text, CURSOR_SEMANTICS).head().clone();
        self.ui.status_bar.modified_indicator_widget.document_modified_status = self.document.is_modified();
    }
    fn update_cursor_positions(&mut self){
        let text = self.document.text();
        let selections = self.document.selections();
        self.ui.highlighter.set_primary_cursor_position(self.document.view().primary_cursor_position(text, selections, CURSOR_SEMANTICS));
        self.ui.highlighter.selections = self.document.view().selections(selections, text);
        self.ui.status_bar.document_cursor_position_widget.document_cursor_position = selections.primary().selection_to_selection2d(text, CURSOR_SEMANTICS).head().clone()
    }
    // should this take a selection to follow, instead of always following primary?
    fn scroll_and_update(&mut self){
        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);

        self.update_ui();
    }
    // should this take a selection to follow, instead of always following primary?
    fn checked_scroll_and_update(&mut self){
        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        if self.document.view().should_scroll(selections.primary(), &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
            self.update_ui();
        }else{
            self.update_cursor_positions();
        }
    }
    fn update_util_bar_ui(&mut self){
        let text = self.ui.util_bar.utility_widget.text_box.text.clone();
        let selections = Selections::new(vec![self.ui.util_bar.utility_widget.text_box.selection.clone()], 0, &text);
        self.ui.util_bar.utility_widget.text_box.view = self.ui.util_bar.utility_widget.text_box.view.scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    fn update_alternate_util_bar_ui(&mut self){
        let text = self.ui.util_bar.alternate_utility_widget.text_box.text.clone();
        let selections = Selections::new(vec![self.ui.util_bar.alternate_utility_widget.text_box.selection.clone()], 0, &text);
        self.ui.util_bar.alternate_utility_widget.text_box.view = self.ui.util_bar.alternate_utility_widget.text_box.view.scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    /////////////////////////////////////////////////////////////////////////// Reuse ////////////////////////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////////////////// Custom ////////////////////////////////////////////////////////////////////////////////
    fn esc_handle(&mut self){
        assert!(self.mode == Mode::Insert);
        //TODO: if lsp suggestions displaying(currently unimplemented), exit that display
        if self.document.selections().count() > 1{
            self.clear_non_primary_selections();
        }
        else if self.document.selections().primary().is_extended(CURSOR_SEMANTICS){
            self.collapse_selections();
        }
        else{
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));
        }
    }
    /////////////////////////////////////////////////////////////////////////// Custom ////////////////////////////////////////////////////////////////////////////////
    
    /////////////////////////////////////////////////////////////////////////// Built in ////////////////////////////////////////////////////////////////////////////////
    fn add_selection_above(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        match self.document.selections().add_selection_above(&text, CURSOR_SEMANTICS){
            Ok(selections) => {
                *self.document.selections_mut() = selections;
                self.checked_scroll_and_update();   //follow self.document.selections().first()
            }
            Err(e) => {
                match e{
                    SelectionsError::CannotAddSelectionAbove => {self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));}
                    SelectionsError::SpansMultipleLines => {/*extend selection up*/}
                    _ => {/*warn unhandled error*/}
                }
            }
        }
    }
    //TODO: impl similarly to add_selection_above...
    fn add_selection_below(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        if let Ok(selections) = self.document.selections().add_selection_below(&text, CURSOR_SEMANTICS){
            *self.document.selections_mut() = selections;
            self.checked_scroll_and_update();
        }else{
            //warn action could not be performed
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));

            // could also match error. if error is multi-line selection, extend selection up
        }
    }
    fn backspace(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        self.document.backspace(CURSOR_SEMANTICS);

        self.scroll_and_update();

        if len != self.document.len(){  //if length has changed after backspace
            self.ui.document_viewport.document_widget.doc_length = self.document.len();
        }
    }
    fn center_view_vertically_around_cursor(&mut self){
        assert!(self.mode == Mode::Space);
        let text = self.document.text().clone();
        let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().center_vertically_around_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
        self.update_ui();
        //exit space mode
        self.mode = Mode::Insert;
    }
    fn clear_non_primary_selections(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.selections().clear_non_primary_selections(){
            Ok(new_selections) => {
                *self.document.selections_mut() = new_selections;
                self.checked_scroll_and_update();
            }
            Err(e) => {
                match e{
                    SelectionsError::SingleSelection => {self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::SingleSelection));}
                    _ => {/*I don't think any other SelectionsErrors are possible here*/}
                }
            }
        }
    }
    fn collapse_selections(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        for selection in self.document.selections_mut().iter_mut(){
            if let Ok(new_selection) = selection.collapse(&text, CURSOR_SEMANTICS){
                *selection = new_selection;
            }else{
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput))
            }
        }
        self.checked_scroll_and_update();
    }
    fn command_mode_accept(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        if self.parse_command(&self.ui.util_bar.utility_widget.text_box.text.to_string()).is_ok(){
            self.command_mode_exit();
        }else{
            self.command_mode_exit();
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::CommandParseFailed));
        }
        //ui.scroll(editor);
    }
    fn command_mode_backspace(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.backspace();
        self.update_util_bar_ui();
    }
    fn command_mode_delete(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.delete();
        self.update_util_bar_ui();
    }
    fn command_mode_exit(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.clear();
        self.mode = Mode::Insert;
    }
    fn command_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.extend_selection_end();
        self.update_util_bar_ui();
    }
    fn command_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.extend_selection_home();
        self.update_util_bar_ui();
    }
    fn command_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.extend_selection_left();
        self.update_util_bar_ui();
    }
    fn command_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.extend_selection_right();
        self.update_util_bar_ui();
    }
    fn command_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.insert_char(c);
        self.update_util_bar_ui();
    }
    fn command_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.move_cursor_left();
        self.update_util_bar_ui();
    }
    fn command_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_end();
        self.update_util_bar_ui();
    }
    fn command_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_start();
        self.update_util_bar_ui();
    }
    fn command_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Command));
        self.ui.util_bar.utility_widget.text_box.move_cursor_right();
        self.update_util_bar_ui();
    }
    fn copy(&mut self){ //TODO: how can the user be given visual feedback that the requested action was accomplished? util bar indicator, similar to warning mode, without restricting further keypresses?
        assert!(self.mode == Mode::Insert);
        // Errors if more than one selection
        if self.document.copy().is_err(){
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::MultipleSelections));
        }
    }
    fn cut(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        // Errors if more than one selection
        if self.document.cut(CURSOR_SEMANTICS).is_err(){
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::MultipleSelections));
        }else{
            self.scroll_and_update();

            if len != self.document.len(){  //if length has changed after cut
                self.ui.document_viewport.document_widget.doc_length = self.document.len();
            }
        }
    }
    fn delete(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        self.document.delete(CURSOR_SEMANTICS);

        self.scroll_and_update();

        if len != self.document.len(){  //if length has changed after delete
            self.ui.document_viewport.document_widget.doc_length = self.document.len();
        }
    }
    fn display_line_numbers(&mut self){
        assert!(self.mode == Mode::Insert);
        self.ui.document_viewport.toggle_line_numbers();
                
        self.ui.update_layouts(self.mode);

        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );

        self.update_ui();
    }
    fn display_status_bar(&mut self){
        assert!(self.mode == Mode::Insert);
        self.ui.status_bar.toggle_status_bar();
                
        self.ui.update_layouts(self.mode);

        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );

        self.update_ui();
    }
    fn extend_selection(&mut self, extend_fn: fn(&Selection, &Rope, CursorSemantics) -> Result<Selection, SelectionError>){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
    
        for selection in self.document.selections_mut().iter_mut(){
            //*selection = extend_fn(selection, &text, CURSOR_SEMANTICS);
            if let Ok(new_selection) = extend_fn(selection, &text, CURSOR_SEMANTICS){
                *selection = new_selection;
            }else{
                //warning
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput))
            }
        }
    
        self.checked_scroll_and_update();
    }
    fn extend_selection_down(&mut self){
        self.extend_selection(Selection::extend_down);
    }
    fn extend_selection_end(&mut self){
        self.extend_selection(Selection::extend_line_text_end);
    }
    fn extend_selection_home(&mut self){
        self.extend_selection(Selection::extend_home);
    }
    fn extend_selection_left(&mut self){
        self.extend_selection(Selection::extend_left);
    }
    fn extend_selection_page(&mut self, extend_fn: fn(&Selection, &Rope, &View, CursorSemantics) -> Result<Selection, SelectionError>){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        for selection in self.document.selections_mut().iter_mut(){
            //*selection = extend_fn(selection, &text, &view, CURSOR_SEMANTICS);
            if let Ok(new_selection) = extend_fn(selection, &text, &view, CURSOR_SEMANTICS){
                *selection = new_selection;
            }else{
                // warning
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput))
            }
        }

        self.checked_scroll_and_update();
    }
    fn extend_selection_page_down(&mut self){
        self.extend_selection_page(Selection::extend_page_down);
    }
    fn extend_selection_page_up(&mut self){
        self.extend_selection_page(Selection::extend_page_up);
    }
    fn extend_selection_right(&mut self){
        self.extend_selection(Selection::extend_right);
    }
    fn extend_selection_up(&mut self){
        self.extend_selection(Selection::extend_up);
    }
    fn find_replace_mode_accept(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        self.document.search(&self.ui.util_bar.utility_widget.text_box.text.to_string(), CURSOR_SEMANTICS);
        self.scroll_and_update();
        self.find_replace_mode_exit();
    }
    fn find_replace_mode_backspace(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.backspace();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.backspace();
            self.update_util_bar_ui();
        }

        self.find_replace_mode_text_validity_check();
    }
    fn find_replace_mode_delete(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.delete();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.delete();
            self.update_util_bar_ui();
        }

        self.find_replace_mode_text_validity_check();
    }
    fn find_replace_mode_exit(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        self.ui.util_bar.utility_widget.text_box.clear();
        self.ui.util_bar.alternate_utility_widget.text_box.clear();
        self.ui.util_bar.alternate_focused = false;
        self.mode = Mode::Insert;
    }
    fn find_replace_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.extend_selection_end();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.extend_selection_end();
            self.update_util_bar_ui();
        }
    }
    fn find_replace_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.extend_selection_home();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.extend_selection_home();
            self.update_util_bar_ui();
        }
    }
    fn find_replace_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.extend_selection_left();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.extend_selection_left();
            self.update_util_bar_ui();
        }
    }
    fn find_replace_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.extend_selection_right();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.extend_selection_right();
            self.update_util_bar_ui();
        }
    }
    fn find_replace_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.insert_char(c);
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.insert_char(c);
            self.update_util_bar_ui();
        }
        
        self.find_replace_mode_text_validity_check();
    }
    fn find_replace_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.move_cursor_left();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.move_cursor_left();
            self.update_util_bar_ui();
        }
    }
    fn find_replace_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.move_cursor_line_end();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.move_cursor_line_end();
            self.update_util_bar_ui();
        }
    }
    fn find_replace_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.move_cursor_line_start();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.move_cursor_line_start();
            self.update_util_bar_ui();
        }
    }
    fn find_replace_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.move_cursor_right();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.move_cursor_right();
            self.update_util_bar_ui();
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
        self.ui.util_bar.alternate_focused = !self.ui.util_bar.alternate_focused;
    }
    fn find_replace_mode_text_validity_check(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::FindReplace));
        //run text validity check
        if !self.document.text().clone().to_string().contains(&self.ui.util_bar.utility_widget.text_box.text.to_string()){
            self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        }else{
            self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
        }
    }
    fn goto_mode_accept(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        if let Ok(line_number) = self.ui.util_bar.utility_widget.text_box.text.to_string().parse::<usize>(){
            // if line_number <= self.document.len() && line_number > 0
            let line_number = line_number.saturating_sub(1);

            //if line_number < self.ui.document_length(){
            if line_number < self.document.len(){   //&& line_number > 0
                let text =  self.document.text().clone();
                
                //if self.document.selections().count() > 1{
                if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){
                    *self.document.selections_mut() = new_selections;
                }else{/*already single selection, which is what we want*/}

                if let Ok(new_selection) = self.document.selections().primary().set_from_line_number(line_number, &text, Movement::Move, CURSOR_SEMANTICS){
                    *self.document.selections_mut().primary_mut() = new_selection;
                    self.scroll_and_update();
                    self.goto_mode_exit();
                }else{
                    // warning
                    self.ui.util_bar.utility_widget.text_box.clear();
                    self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));
                }
            }else{
                self.goto_mode_exit();
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));
            }
        }else{
            self.ui.util_bar.utility_widget.text_box.clear();
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));
        }
    }
    fn goto_mode_backspace(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.backspace();
        self.update_util_bar_ui();
    
        self.goto_mode_text_validity_check();
    }
    fn goto_mode_delete(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.delete();
        self.update_util_bar_ui();
    
        self.goto_mode_text_validity_check();
    }
    fn goto_mode_exit(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.clear();
        self.mode = Mode::Insert;
    }
    fn goto_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.extend_selection_end();
        self.update_util_bar_ui();
    }
    fn goto_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.extend_selection_home();
        self.update_util_bar_ui();
    }
    fn goto_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.extend_selection_left();
        self.update_util_bar_ui();
    }
    fn goto_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.extend_selection_right();
        self.update_util_bar_ui();
    }
    fn goto_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.insert_char(c);
        self.update_util_bar_ui();
    
        self.goto_mode_text_validity_check();
    }
    fn goto_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.move_cursor_left();
        self.update_util_bar_ui();
    }
    fn goto_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_end();
        self.update_util_bar_ui();
    }
    fn goto_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_start();
        self.update_util_bar_ui();
    }
    fn goto_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        self.ui.util_bar.utility_widget.text_box.move_cursor_right();
        self.update_util_bar_ui();
    }
    fn goto_mode_text_validity_check(&mut self){
        assert!(self.mode == Mode::Utility(UtilityKind::Goto));
        // run text validity check
        let mut is_numeric = true;
        for grapheme in self.ui.util_bar.utility_widget.text_box.text.chars(){ // .graphemes(true)?
            if !grapheme.is_ascii_digit(){
                is_numeric = false;
            }
        }
        let exceeds_doc_length = match self.ui.util_bar.utility_widget.text_box.text.to_string().parse::<usize>(){
            Ok(line_number) => {
                //line_number > self.ui.document_length()
                line_number > self.document.len()
            }
            Err(_) => false
        };
        if !is_numeric || exceeds_doc_length{
            self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        }else{
            self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
        }
    }
    fn increment_primary_selection(&mut self){
        if let Ok(new_selections) = self.document.selections().increment_primary_selection(){
            *self.document.selections_mut() = new_selections;
            self.scroll_and_update();
            //TODO: if in space mode, exit space mode
        }else{
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::SingleSelection));
        }
    }
    // this should be organized alphabetically in source code fns
    fn decrement_primary_selection(&mut self){
        if let Ok(new_selections) = self.document.selections().decrement_primary_selection(){
            *self.document.selections_mut() = new_selections;
            self.scroll_and_update();
            //TODO: if in space mode, exit space mode
        }else{
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::SingleSelection));
        }
    }
    fn insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Insert);
        self.document.insert_string(&c.to_string(), CURSOR_SEMANTICS);

        self.scroll_and_update();
    }
    fn insert_newline(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        self.document.insert_string("\n", CURSOR_SEMANTICS);

        self.scroll_and_update();

        if len != self.document.len(){  //if length has changed after newline
            self.ui.document_viewport.document_widget.doc_length = self.document.len();
        }
    }
    fn insert_tab(&mut self){
        assert!(self.mode == Mode::Insert);
        self.document.insert_string("\t", CURSOR_SEMANTICS);

        self.scroll_and_update();
    }
    fn move_cursor(&mut self, movement_fn: fn(&Selection, &Rope, CursorSemantics) -> Result<Selection, SelectionError>){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
    
        //if self.document.selections().count() > 1{
        //    *self.document.selections_mut() = self.document.selections().clear_non_primary_selections();
        //}
        if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){
            *self.document.selections_mut() = new_selections;
        }else{
            //self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::SingleSelection));
        }
    
        for selection in self.document.selections_mut().iter_mut(){
            //*selection = movement_fn(selection, &text, CURSOR_SEMANTICS);
            if let Ok(new_selection) = movement_fn(selection, &text, CURSOR_SEMANTICS){
                *selection = new_selection;
            }else{
                // warning
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput))
            }
        }
    
        self.checked_scroll_and_update();
    }
    fn move_cursor_document_end(&mut self){
        self.move_cursor(Selection::move_doc_end);
    }
    fn move_cursor_document_start(&mut self){
        self.move_cursor(Selection::move_doc_start);
    }
    fn move_cursor_down(&mut self){
        self.move_cursor(Selection::move_down);
    }
    fn move_cursor_left(&mut self){
        self.move_cursor(Selection::move_left);
    }
    fn move_cursor_line_end(&mut self){
        self.move_cursor(Selection::move_line_text_end);
    }
    fn move_cursor_line_start(&mut self){
        self.move_cursor(Selection::move_home);
    }
    fn move_cursor_page(&mut self, movement_fn: fn(&Selection, &Rope, &View, CursorSemantics) -> Result<Selection, SelectionError>){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        //if self.document.selections().count() > 1{
        //    *self.document.selections_mut() = self.document.selections().clear_non_primary_selections();
        //}
        if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){
            *self.document.selections_mut() = new_selections;
        }

        for selection in self.document.selections_mut().iter_mut(){
            //*selection = movement_fn(selection, &text, &view, CURSOR_SEMANTICS);
            if let Ok(new_selection) = movement_fn(selection, &text, &view, CURSOR_SEMANTICS){
                *selection = new_selection;
            }else{
                //warning
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput))
            }
        }

        self.checked_scroll_and_update();
    }
    fn move_cursor_page_down(&mut self){
        self.move_cursor_page(Selection::move_page_down);
    }
    fn move_cursor_page_up(&mut self){
        self.move_cursor_page(Selection::move_page_up);
    }
    fn move_cursor_right(&mut self){
        self.move_cursor(Selection::move_right);
    }
    fn move_cursor_up(&mut self){
        self.move_cursor(Selection::move_up);
    }
    //fn move_cursor_word_end(&mut self){
    //    assert!(self.mode == Mode::Insert);
    //}
    //fn move_cursor_word_start(&mut self){
    //    assert!(self.mode == Mode::Insert);
    //}
    fn no_op(&mut self){/* warn unbound keypress */}
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

        self.scroll_and_update();

        if len != self.document.len(){  //if length has changed after paste
            self.ui.document_viewport.document_widget.doc_length = self.document.len();
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
            self.scroll_and_update();

            if len != self.document.len(){  //if length has changed after paste
                self.ui.document_viewport.document_widget.doc_length = self.document.len();
            }
        }else{
            // warn redo stack empty
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));
        }
    }
    fn resize(&mut self, x: u16, y: u16){
        self.ui.set_terminal_size(x, y);
        self.ui.update_layouts(self.mode);

        self.update_util_bar_ui();

        self.update_alternate_util_bar_ui();

        self.document.view_mut().set_size(self.ui.document_viewport.document_widget.rect.width as usize, self.ui.document_viewport.document_widget.rect.height as usize);

        self.scroll_and_update();
    }
    fn save(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.save(){
            Ok(_) => {
                self.update_ui();
            }
            Err(_) => {
                self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::FileSaveFailed));
            }
        }
    }
    fn scroll_view(&mut self, direction: ScrollDirection, amount: usize){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
    
        let new_view = match direction{
            ScrollDirection::Down => self.document.view().scroll_down(amount, &text),
            ScrollDirection::Left => self.document.view().scroll_left(amount),
            ScrollDirection::Right => self.document.view().scroll_right(amount, &text),
            ScrollDirection::Up => self.document.view().scroll_up(amount),
        };
    
        *self.document.view_mut() = new_view;
        self.update_ui();
    }
    fn scroll_view_down(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Down, amount);
    }
    fn scroll_view_left(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Left, amount);
    }
    fn scroll_view_right(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Right, amount);
    }
    fn scroll_view_up(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Up, amount);
    }
    fn select_all(&mut self){
        assert!(self.mode == Mode::Insert);
        if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){
            *self.document.selections_mut() = new_selections;
        }
        if let Ok(new_selection) = self.document.selections().primary().select_all(self.document.text(), CURSOR_SEMANTICS){
            *self.document.selections_mut().primary_mut() = new_selection;
        }else{
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput))
        }

        self.checked_scroll_and_update();
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
    // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
    fn undo(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        if let Ok(_) = self.document.undo(CURSOR_SEMANTICS){
            self.scroll_and_update();

            if len != self.document.len(){  //if length has changed after paste
                self.ui.document_viewport.document_widget.doc_length = self.document.len();
            }
        }else{
            // warn undo stack empty
            self.mode = Mode::Utility(UtilityKind::Warning(WarningKind::InvalidInput));
        }
    }
    fn warning_mode_exit(&mut self){
        assert!(matches!(self.mode, Mode::Utility(UtilityKind::Warning(_))));
        self.mode = Mode::Insert;
    }
    /////////////////////////////////////////////////////////////////////////// Built in ////////////////////////////////////////////////////////////////////////////////
}

use std::error::Error;
use std::path::PathBuf;
use crossterm::event;
use ratatui::layout::Rect;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::ui::UserInterface;
use edit_core::selection::{CursorSemantics, Movement, Selection, Selections, SelectionError, SelectionsError};
use edit_core::view::View;
use edit_core::document::{Document, DocumentError};
use ropey::Rope;
use crate::keybind;
use crate::config::{CURSOR_SEMANTICS, SHOW_SAME_STATE_WARNINGS};



pub enum ScrollDirection{
    Down,
    Left,
    Right,
    Up
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode{
    Insert,
    Space,
    Warning(WarningKind),
    Command,
    FindReplace,
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
    //SameState     //"Requested action results in the same state"  //TODO
}



pub struct Application{
    should_quit: bool,
    mode: Mode,
    document: Document,
    ui: UserInterface,
}
impl Application{
    pub fn new(file_path: &str, terminal: &Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<Self, Box<dyn Error>>{
        let terminal_size = terminal.size()?;
        let terminal_rect = Rect::new(0, 0, terminal_size.width, terminal_size.height);

        let mut instance = Self{
            should_quit: false,
            mode: Mode::Insert,
            document: Document::new(CURSOR_SEMANTICS),
            ui: UserInterface::new(terminal_rect)
        };

        let path = PathBuf::from(file_path).canonicalize()?;

        instance.document = Document::open(&path, CURSOR_SEMANTICS)?;
        instance.ui.status_bar.file_name_widget.file_name = instance.document.file_name();
        instance.ui.document_viewport.document_widget.doc_length = instance.document.len();
        
        instance.ui.update_layouts(instance.mode);
        
        //init backend doc view size
        instance.document.view_mut().set_size(
            instance.ui.document_viewport.document_widget.rect.width as usize,
            instance.ui.document_viewport.document_widget.rect.height as usize
        );
        instance.scroll_and_update(&instance.document.selections().primary().clone());

        Ok(instance)
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn Error>>{
        loop{
            self.ui.update_layouts(self.mode);
            self.ui.render(terminal, self.mode)?;
            self.handle_event()?;
            if self.should_quit{
                return Ok(());
            }
        }
    }

    fn handle_event(&mut self) -> Result<(), Box<dyn Error>>{
        match event::read()?{
            event::Event::Key(key_event) => {
                match self.mode{
                    Mode::Insert => {keybind::handle_insert_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Space => {keybind::handle_space_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Warning(_) => {keybind::handle_warning_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Goto => {keybind::handle_goto_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::FindReplace => {keybind::handle_find_replace_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Command => {keybind::handle_command_mode_keypress(self, key_event.code, key_event.modifiers);}
                }
            },
            event::Event::Resize(x, y) => self.resize(x, y),
            _ => self.no_op(),  //TODO: this no_op should be disambiguated from a keypress no_op, so that they can impl different behavior...
        }

        Ok(())
    }

    // could make separate files for categories of fns. builtin.rs and custom.rs...       custom::escape_handle()     builtin::add_selection_above()
    // or all in one commands.rs file?...
    /////////////////////////////////////////////////////////////////////////// Reuse ////////////////////////////////////////////////////////////////////////////////
    pub fn update_ui(&mut self){
        let text = self.document.text();
        let selections = self.document.selections();
        self.ui.document_viewport.document_widget.text_in_view = self.document.view().text(text);
        self.ui.document_viewport.line_number_widget.line_numbers_in_view = self.document.view().line_numbers(text);
        self.ui.highlighter.set_primary_cursor_position(self.document.view().primary_cursor_position(text, selections, CURSOR_SEMANTICS));
        self.ui.highlighter.selections = self.document.view().selections(selections, text);
        self.ui.status_bar.document_cursor_position_widget.document_cursor_position = selections.primary().selection_to_selection2d(text, CURSOR_SEMANTICS).head().clone();
        self.ui.status_bar.modified_indicator_widget.document_modified_status = self.document.is_modified();
    }
    pub fn update_cursor_positions(&mut self){
        let text = self.document.text();
        let selections = self.document.selections();
        self.ui.highlighter.set_primary_cursor_position(self.document.view().primary_cursor_position(text, selections, CURSOR_SEMANTICS));
        self.ui.highlighter.selections = self.document.view().selections(selections, text);
        self.ui.status_bar.document_cursor_position_widget.document_cursor_position = selections.primary().selection_to_selection2d(text, CURSOR_SEMANTICS).head().clone()
    }
    //TODO: should this take a selection to follow, instead of always following primary?
    pub fn scroll_and_update(&mut self, selection: &Selection){    //, selection_to_follow: &Selection
        let text = self.document.text().clone();
        //let selections = self.document.selections().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(/*selections.primary()*/selection, &text, CURSOR_SEMANTICS);

        self.update_ui();
    }
    //TODO: should this take a selection to follow, instead of always following primary?
    pub fn checked_scroll_and_update(&mut self, selection: &Selection){    //, selection_to_follow: &Selection
        let text = self.document.text().clone();
        //let selections = self.document.selections().clone();
        if self.document.view().should_scroll(/*selections.primary()*/selection, &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(/*selections.primary()*/selection, &text, CURSOR_SEMANTICS);
            self.update_ui();
        }else{
            self.update_cursor_positions();
        }
    }
    pub fn update_util_bar_ui(&mut self){
        let text = self.ui.util_bar.utility_widget.text_box.text.clone();
        let selections = Selections::new(vec![self.ui.util_bar.utility_widget.text_box.selection.clone()], 0, &text);
        self.ui.util_bar.utility_widget.text_box.view = self.ui.util_bar.utility_widget.text_box.view.scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    pub fn update_alternate_util_bar_ui(&mut self){
        let text = self.ui.util_bar.alternate_utility_widget.text_box.text.clone();
        let selections = Selections::new(vec![self.ui.util_bar.alternate_utility_widget.text_box.selection.clone()], 0, &text);
        self.ui.util_bar.alternate_utility_widget.text_box.view = self.ui.util_bar.alternate_utility_widget.text_box.view.scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    /////////////////////////////////////////////////////////////////////////// Reuse ////////////////////////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////////////////// Custom ////////////////////////////////////////////////////////////////////////////////
    pub fn esc_handle(&mut self){
        assert!(self.mode == Mode::Insert);
        //TODO: if lsp suggestions displaying(currently unimplemented), exit that display
        if self.document.selections().count() > 1{
            self.clear_non_primary_selections();
        }
        else if self.document.selections().primary().is_extended(CURSOR_SEMANTICS){
            self.collapse_selections();
        }
        else{
            if SHOW_SAME_STATE_WARNINGS{
                self.mode = Mode::Warning(WarningKind::InvalidInput);
            }
        }
    }
    /////////////////////////////////////////////////////////////////////////// Custom ////////////////////////////////////////////////////////////////////////////////
    
    /////////////////////////////////////////////////////////////////////////// Built in ////////////////////////////////////////////////////////////////////////////////
    pub fn add_selection_above(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        match self.document.selections().add_selection_above(&text, CURSOR_SEMANTICS){
            Ok(selections) => {
                *self.document.selections_mut() = selections;
                self.checked_scroll_and_update(&self.document.selections().first().clone());
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::CannotAddSelectionAbove => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    SelectionsError::SpansMultipleLines => {/*TODO: extend selection up*/}
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn add_selection_below(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        match self.document.selections().add_selection_below(&text, CURSOR_SEMANTICS){
            Ok(selections) => {
                *self.document.selections_mut() = selections;
                self.checked_scroll_and_update(&self.document.selections().last().clone());
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::CannotAddSelectionBelow => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    SelectionsError::SpansMultipleLines => {/*TODO: extend selection down*/}
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn backspace(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.backspace(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                if len != self.document.len(){  //if length has changed after backspace
                    self.ui.document_viewport.document_widget.doc_length = self.document.len();
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::SelectionAtDocBounds => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn center_view_vertically_around_cursor(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space);
        let text = self.document.text().clone();
        *self.document.view_mut() = self.document.view().center_vertically_around_cursor(&self.document.selections().primary().clone(), &text, CURSOR_SEMANTICS);   //TODO: can this fail?
        self.update_ui();

        if self.mode == Mode::Space{
            self.mode = Mode::Insert;
        }
    }
    pub fn clear_non_primary_selections(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.selections().clear_non_primary_selections(){
            Ok(new_selections) => {
                *self.document.selections_mut() = new_selections;
                self.checked_scroll_and_update(&self.document.selections().primary().clone());
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.mode = Mode::Warning(WarningKind::SingleSelection);}
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn collapse_selections(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        for selection in self.document.selections_mut().iter_mut(){ //TODO: consider how to handle errors when iterating over multiple selections...
            match selection.collapse(&text, CURSOR_SEMANTICS){
                Ok(new_selection) => {
                    *selection = new_selection;
                }
                Err(e) => {
                    let this_file = std::panic::Location::caller().file();
                    let line_number = std::panic::Location::caller().line();
                    match e{
                        SelectionError::ResultsInSameState => {
                            if SHOW_SAME_STATE_WARNINGS{
                                self.mode = Mode::Warning(WarningKind::InvalidInput);
                            }
                        }
                        _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                    }
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone());   //TODO: should this be moved up into the Ok match arm?  //can't borrow self in Ok match arm above because we are iterating through multiple selections
    }
    pub fn command_mode_accept(&mut self){
        assert!(self.mode == Mode::Command);
        if self.parse_command(&self.ui.util_bar.utility_widget.text_box.text.to_string()).is_ok(){
            self.command_mode_exit();
        }else{
            self.command_mode_exit();
            self.mode = Mode::Warning(WarningKind::CommandParseFailed);
        }
        //ui.scroll(editor);
    }
    pub fn command_mode_backspace(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.backspace();
        self.update_util_bar_ui();
    }
    pub fn command_mode_delete(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.delete();
        self.update_util_bar_ui();
    }
    pub fn command_mode_exit(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.clear();
        self.mode = Mode::Insert;
    }
    pub fn command_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.extend_selection_end();
        self.update_util_bar_ui();
    }
    pub fn command_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.extend_selection_home();
        self.update_util_bar_ui();
    }
    pub fn command_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.extend_selection_left();
        self.update_util_bar_ui();
    }
    pub fn command_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.extend_selection_right();
        self.update_util_bar_ui();
    }
    pub fn command_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.insert_char(c);
        self.update_util_bar_ui();
    }
    pub fn command_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.move_cursor_left();
        self.update_util_bar_ui();
    }
    pub fn command_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_end();
        self.update_util_bar_ui();
    }
    pub fn command_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_start();
        self.update_util_bar_ui();
    }
    pub fn command_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Command);
        self.ui.util_bar.utility_widget.text_box.move_cursor_right();
        self.update_util_bar_ui();
    }
    pub fn copy(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.copy(){
            Ok(_) => {
                //TODO: how can the user be given visual feedback that the requested action was accomplished? util bar indicator, similar to warning mode, without restricting further keypresses?
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::SelectionsError(selections_error) => {
                        match selections_error{
                            SelectionsError::MultipleSelections => {self.mode = Mode::Warning(WarningKind::MultipleSelections);}
                            _ => {panic!("{selections_error:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                } 
            }
        }
    }
    pub fn cut(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.cut(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                if len != self.document.len(){  //if length has changed after cut
                    self.ui.document_viewport.document_widget.doc_length = self.document.len();
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::SelectionAtDocBounds => {}
                    DocumentError::SelectionsError(selections_error) => {
                        match selections_error{
                            SelectionsError::MultipleSelections => {self.mode = Mode::Warning(WarningKind::MultipleSelections);}
                            _ => {panic!("{selections_error:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn delete(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.delete(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                if len != self.document.len(){  //if length has changed after delete
                    self.ui.document_viewport.document_widget.doc_length = self.document.len();
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::SelectionAtDocBounds => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn display_line_numbers(&mut self){
        assert!(self.mode == Mode::Insert);
        self.ui.document_viewport.toggle_line_numbers();
                
        self.ui.update_layouts(self.mode);
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui();
    }
    pub fn display_status_bar(&mut self){
        assert!(self.mode == Mode::Insert);
        self.ui.status_bar.toggle_status_bar();
                
        self.ui.update_layouts(self.mode);
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui();
    }
    pub fn extend_selection(&mut self, extend_fn: fn(&Selection, &Rope, CursorSemantics) -> Result<Selection, SelectionError>){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
    
        for selection in self.document.selections_mut().iter_mut(){
            match extend_fn(selection, &text, CURSOR_SEMANTICS){
                Ok(new_selection) => {
                    *selection = new_selection;
                }
                Err(e) => {
                    let this_file = std::panic::Location::caller().file();
                    let line_number = std::panic::Location::caller().line();
                    match e{
                        SelectionError::ResultsInSameState => {
                            if SHOW_SAME_STATE_WARNINGS{
                                self.mode = Mode::Warning(WarningKind::InvalidInput);
                            }
                        }
                        _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.");}
                    }
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone());
    }
    pub fn extend_selection_down(&mut self){
        self.extend_selection(Selection::extend_down);
    }
    pub fn extend_selection_end(&mut self){
        self.extend_selection(Selection::extend_line_text_end);
    }
    pub fn extend_selection_home(&mut self){
        self.extend_selection(Selection::extend_home);
    }
    pub fn extend_selection_left(&mut self){
        self.extend_selection(Selection::extend_left);
    }
    pub fn extend_selection_page(&mut self, extend_fn: fn(&Selection, &Rope, &View, CursorSemantics) -> Result<Selection, SelectionError>){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        for selection in self.document.selections_mut().iter_mut(){
            match extend_fn(selection, &text, &view, CURSOR_SEMANTICS){
                Ok(new_selection) => {
                    *selection = new_selection;
                }
                Err(e) => {
                    let this_file = std::panic::Location::caller().file();
                    let line_number = std::panic::Location::caller().line();
                    match e{
                        SelectionError::ResultsInSameState => {
                            if SHOW_SAME_STATE_WARNINGS{
                                self.mode = Mode::Warning(WarningKind::InvalidInput);
                            }
                        }
                        _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.");}
                    }
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone());
    }
    pub fn extend_selection_page_down(&mut self){
        self.extend_selection_page(Selection::extend_page_down);
    }
    pub fn extend_selection_page_up(&mut self){
        self.extend_selection_page(Selection::extend_page_up);
    }
    pub fn extend_selection_right(&mut self){
        self.extend_selection(Selection::extend_right);
    }
    pub fn extend_selection_word_boundary_forward(&mut self){
        self.extend_selection(Selection::extend_right_word_boundary);
    }
    pub fn extend_selection_word_boundary_backward(&mut self){
        self.extend_selection(Selection::extend_left_word_boundary);
    }
    pub fn extend_selection_up(&mut self){
        self.extend_selection(Selection::extend_up);
    }
    pub fn find_replace_mode_accept(&mut self){
        assert!(self.mode == Mode::FindReplace);
        self.document.search(&self.ui.util_bar.utility_widget.text_box.text.to_string());
        self.scroll_and_update(&self.document.selections().primary().clone());
        self.find_replace_mode_exit();
    }
    pub fn find_replace_mode_backspace(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.backspace();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.backspace();
            self.update_util_bar_ui();
        }

        self.find_replace_mode_text_validity_check();
    }
    pub fn find_replace_mode_delete(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.delete();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.delete();
            self.update_util_bar_ui();
        }

        self.find_replace_mode_text_validity_check();
    }
    pub fn find_replace_mode_exit(&mut self){
        assert!(self.mode == Mode::FindReplace);
        self.ui.util_bar.utility_widget.text_box.clear();
        self.ui.util_bar.alternate_utility_widget.text_box.clear();
        self.ui.util_bar.alternate_focused = false;
        self.mode = Mode::Insert;
    }
    pub fn find_replace_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.extend_selection_end();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.extend_selection_end();
            self.update_util_bar_ui();
        }
    }
    pub fn find_replace_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.extend_selection_home();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.extend_selection_home();
            self.update_util_bar_ui();
        }
    }
    pub fn find_replace_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.extend_selection_left();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.extend_selection_left();
            self.update_util_bar_ui();
        }
    }
    pub fn find_replace_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.extend_selection_right();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.extend_selection_right();
            self.update_util_bar_ui();
        }
    }
    pub fn find_replace_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.insert_char(c);
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.insert_char(c);
            self.update_util_bar_ui();
        }
        
        self.find_replace_mode_text_validity_check();
    }
    pub fn find_replace_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.move_cursor_left();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.move_cursor_left();
            self.update_util_bar_ui();
        }
    }
    pub fn find_replace_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.move_cursor_line_end();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.move_cursor_line_end();
            self.update_util_bar_ui();
        }
    }
    pub fn find_replace_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.move_cursor_line_start();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.move_cursor_line_start();
            self.update_util_bar_ui();
        }
    }
    pub fn find_replace_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::FindReplace);
        if self.ui.util_bar.alternate_focused{
            self.ui.util_bar.alternate_utility_widget.text_box.move_cursor_right();
            self.update_alternate_util_bar_ui();
        }else{
            self.ui.util_bar.utility_widget.text_box.move_cursor_right();
            self.update_util_bar_ui();
        }
    }
    pub fn find_replace_mode_next_instance(&mut self){
        assert!(self.mode == Mode::FindReplace);
    }
    pub fn find_replace_mode_previous_instance(&mut self){
        assert!(self.mode == Mode::FindReplace);
    }
    pub fn find_replace_mode_switch_util_bar_focus(&mut self){
        assert!(self.mode == Mode::FindReplace);
        self.ui.util_bar.alternate_focused = !self.ui.util_bar.alternate_focused;
    }
    pub fn find_replace_mode_text_validity_check(&mut self){
        assert!(self.mode == Mode::FindReplace);
        //run text validity check
        if !self.document.text().clone().to_string().contains(&self.ui.util_bar.utility_widget.text_box.text.to_string()){
            self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        }else{
            self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
        }
    }
    pub fn goto_mode_accept(&mut self){
        assert!(self.mode == Mode::Goto);
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
                    self.scroll_and_update(&self.document.selections().primary().clone());
                    self.goto_mode_exit();
                }else{
                    // warning
                    self.ui.util_bar.utility_widget.text_box.clear();
                    self.mode = Mode::Warning(WarningKind::InvalidInput);
                }
            }else{
                self.goto_mode_exit();
                self.mode = Mode::Warning(WarningKind::InvalidInput);
            }
        }else{
            self.ui.util_bar.utility_widget.text_box.clear();
            self.mode = Mode::Warning(WarningKind::InvalidInput);
        }
    }
    pub fn goto_mode_backspace(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.backspace();
        self.update_util_bar_ui();
    
        self.goto_mode_text_validity_check();
    }
    pub fn goto_mode_delete(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.delete();
        self.update_util_bar_ui();
    
        self.goto_mode_text_validity_check();
    }
    pub fn goto_mode_exit(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.clear();
        self.mode = Mode::Insert;
    }
    pub fn goto_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.extend_selection_end();
        self.update_util_bar_ui();
    }
    pub fn goto_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.extend_selection_home();
        self.update_util_bar_ui();
    }
    pub fn goto_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.extend_selection_left();
        self.update_util_bar_ui();
    }
    pub fn goto_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.extend_selection_right();
        self.update_util_bar_ui();
    }
    pub fn goto_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.insert_char(c);
        self.update_util_bar_ui();
    
        self.goto_mode_text_validity_check();
    }
    pub fn goto_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.move_cursor_left();
        self.update_util_bar_ui();
    }
    pub fn goto_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_end();
        self.update_util_bar_ui();
    }
    pub fn goto_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_start();
        self.update_util_bar_ui();
    }
    pub fn goto_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Goto);
        self.ui.util_bar.utility_widget.text_box.move_cursor_right();
        self.update_util_bar_ui();
    }
    pub fn goto_mode_text_validity_check(&mut self){
        assert!(self.mode == Mode::Goto);
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
    pub fn increment_primary_selection(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space);
        match self.document.selections().increment_primary_selection(){
            Ok(new_selections) => {
                *self.document.selections_mut() = new_selections;
                self.scroll_and_update(&self.document.selections().primary().clone());
                if self.mode == Mode::Space{
                    self.mode = Mode::Insert;
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.mode = Mode::Warning(WarningKind::SingleSelection);}
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    //TODO: this should be organized alphabetically in source code fns
    pub fn decrement_primary_selection(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space);
        match self.document.selections().decrement_primary_selection(){
            Ok(new_selections) => {
                *self.document.selections_mut() = new_selections;
                self.scroll_and_update(&self.document.selections().primary().clone());
                if self.mode == Mode::Space{
                    self.mode = Mode::Insert;
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.mode = Mode::Warning(WarningKind::SingleSelection);}
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Insert);
        match self.document.insert_string(&c.to_string(), CURSOR_SEMANTICS){
            Ok(_) => {self.scroll_and_update(&self.document.selections().primary().clone());}
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::InvalidInput => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn insert_newline(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.insert_string("\n", CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                if len != self.document.len(){  //if length has changed after newline
                    self.ui.document_viewport.document_widget.doc_length = self.document.len();
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::InvalidInput => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn insert_tab(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.insert_string("\t", CURSOR_SEMANTICS){
            Ok(_) => {self.scroll_and_update(&self.document.selections().primary().clone());}
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::InvalidInput => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn move_cursor(&mut self, movement_fn: fn(&Selection, &Rope, CursorSemantics) -> Result<Selection, SelectionError>){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
    
        if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){
            *self.document.selections_mut() = new_selections;
        }// intentionally ignoring any errors
    
        for selection in self.document.selections_mut().iter_mut(){
            match movement_fn(selection, &text, CURSOR_SEMANTICS){
                Ok(new_selection) => {*selection = new_selection;}
                Err(e) => {
                    let this_file = std::panic::Location::caller().file();
                    let line_number = std::panic::Location::caller().line();
                    match e{
                        SelectionError::ResultsInSameState => {
                            if SHOW_SAME_STATE_WARNINGS{
                                self.mode = Mode::Warning(WarningKind::InvalidInput);
                            }
                        }
                        _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                    }
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone());
    }
    pub fn move_cursor_document_end(&mut self){
        self.move_cursor(Selection::move_doc_end);
    }
    pub fn move_cursor_document_start(&mut self){
        self.move_cursor(Selection::move_doc_start);
    }
    pub fn move_cursor_down(&mut self){
        self.move_cursor(Selection::move_down);
    }
    pub fn move_cursor_left(&mut self){
        self.move_cursor(Selection::move_left);
    }
    pub fn move_cursor_line_end(&mut self){
        self.move_cursor(Selection::move_line_text_end);
    }
    pub fn move_cursor_line_start(&mut self){
        self.move_cursor(Selection::move_home);
    }
    pub fn move_cursor_page(&mut self, movement_fn: fn(&Selection, &Rope, &View, CursorSemantics) -> Result<Selection, SelectionError>){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let view = self.document.view().clone();

        if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){
            *self.document.selections_mut() = new_selections;
        }// intentionally ignoring any errors

        for selection in self.document.selections_mut().iter_mut(){
            match movement_fn(selection, &text, &view, CURSOR_SEMANTICS){
                Ok(new_selection) => {*selection = new_selection;}
                Err(e) => {
                    let this_file = std::panic::Location::caller().file();
                    let line_number = std::panic::Location::caller().line();
                    match e{
                        SelectionError::ResultsInSameState => {
                            if SHOW_SAME_STATE_WARNINGS{
                                self.mode = Mode::Warning(WarningKind::InvalidInput);
                            }
                        }
                        _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                    }
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone());
    }
    pub fn move_cursor_page_down(&mut self){
        self.move_cursor_page(Selection::move_page_down);
    }
    pub fn move_cursor_page_up(&mut self){
        self.move_cursor_page(Selection::move_page_up);
    }
    pub fn move_cursor_right(&mut self){
        self.move_cursor(Selection::move_right);
    }
    pub fn move_cursor_word_boundary_forward(&mut self){
        self.move_cursor(Selection::move_right_word_boundary);
    }
    pub fn move_cursor_word_boundary_backward(&mut self){
        self.move_cursor(Selection::move_left_word_boundary);
    }
    pub fn move_cursor_up(&mut self){
        self.move_cursor(Selection::move_up);
    }
    pub fn no_op(&mut self){/* TODO: warn unbound keypress */}
    pub fn open_new_terminal_window(&self){
        let _ = std::process::Command::new("alacritty")     //TODO: have user define TERMINAL const in config.rs   //or check env vars for $TERM?
            //.arg("msg")     // these extra commands just make new instances use the same backend(daemon?)
            //.arg("create-window")
            //.current_dir(std::env::current_dir().unwrap())    //not needed here, because term spawned here defaults to this directory, but good to know
            .spawn()
            .expect("failed to spawn new terminal at current directory");
    }
    pub fn parse_command(&self, args: &str) -> Result<(), ()>{
        assert!(self.mode == Mode::Command);
        let mut args = args.split_whitespace();
        
        let command = args.next().unwrap();
        match command{
            "term" => {self.open_new_terminal_window();}
            _ => {return Err(())}
        }
    
        Ok(())
    }
    pub fn paste(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.paste(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                if len != self.document.len(){  //if length has changed after paste
                    self.ui.document_viewport.document_widget.doc_length = self.document.len();
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::InvalidInput => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn quit(&mut self){
        assert!(self.mode == Mode::Insert);
        //if self.ui.document_modified(){
        if self.document.is_modified(){
            self.mode = Mode::Warning(WarningKind::FileIsModified);
        }else{
            self.should_quit = true;
        }
    }
    pub fn quit_ignoring_changes(&mut self){
        self.should_quit = true;
    }
    pub fn redo(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.redo(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                if len != self.document.len(){  //if length has changed after paste
                    self.ui.document_viewport.document_widget.doc_length = self.document.len();
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::NoChangesToRedo => {self.mode = Mode::Warning(WarningKind::InvalidInput);}
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn resize(&mut self, x: u16, y: u16){
        self.ui.set_terminal_size(x, y);
        self.ui.update_layouts(self.mode);

        self.update_util_bar_ui();

        self.update_alternate_util_bar_ui();

        self.document.view_mut().set_size(self.ui.document_viewport.document_widget.rect.width as usize, self.ui.document_viewport.document_widget.rect.height as usize);

        self.scroll_and_update(&self.document.selections().primary().clone());
    }
    pub fn save(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.save(){
            Ok(_) => {
                self.update_ui();
            }
            Err(_) => {
                self.mode = Mode::Warning(WarningKind::FileSaveFailed);
            }
        }
    }
    pub fn scroll_view(&mut self, direction: ScrollDirection, amount: usize){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
    
        let new_view = match direction{ //TODO: should these functions error if already at doc bounds?...
            ScrollDirection::Down => self.document.view().scroll_down(amount, &text),
            ScrollDirection::Left => self.document.view().scroll_left(amount),
            ScrollDirection::Right => self.document.view().scroll_right(amount, &text),
            ScrollDirection::Up => self.document.view().scroll_up(amount),
        };
    
        *self.document.view_mut() = new_view;
        self.update_ui();
    }
    pub fn scroll_view_down(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Down, amount);
    }
    pub fn scroll_view_left(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Left, amount);
    }
    pub fn scroll_view_right(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Right, amount);
    }
    pub fn scroll_view_up(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Up, amount);
    }
    pub fn select_all(&mut self){
        assert!(self.mode == Mode::Insert);
        if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){
            *self.document.selections_mut() = new_selections;
        }
        match self.document.selections().primary().select_all(self.document.text(), CURSOR_SEMANTICS){
            Ok(new_selection) => {
                *self.document.selections_mut().primary_mut() = new_selection;
                self.checked_scroll_and_update(&self.document.selections().primary().clone());
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionError::ResultsInSameState => {
                        if SHOW_SAME_STATE_WARNINGS{
                            self.mode = Mode::Warning(WarningKind::InvalidInput);
                        }
                    }
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn set_mode_command(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Command;
    }
    pub fn set_mode_find_replace(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::FindReplace;
    }
    pub fn set_mode_goto(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Goto;
    }
    pub fn set_mode_space(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Space;
    }
    pub fn space_mode_exit(&mut self){
        assert!(self.mode == Mode::Space);
        self.mode = Mode::Insert;
    }
    // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
    pub fn undo(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.undo(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                if len != self.document.len(){  //if length has changed after paste
                    self.ui.document_viewport.document_widget.doc_length = self.document.len();
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::NoChangesToUndo => {self.mode = Mode::Warning(WarningKind::InvalidInput);}
                    _ => {panic!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")}
                }
            }
        }
    }
    pub fn warning_mode_exit(&mut self){
        assert!(matches!(self.mode, Mode::Warning(_)));
        self.mode = Mode::Insert;
    }
    /////////////////////////////////////////////////////////////////////////// Built in ////////////////////////////////////////////////////////////////////////////////
}

use std::error::Error;
use std::path::PathBuf;
use crossterm::event;
use ratatui::layout::Rect;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::ui::UserInterface;
use edit_core::selection::{CursorSemantics, Movement, Selection, SelectionError};
use edit_core::selections::{Selections, SelectionsError};
use edit_core::view::{View, ViewError};
use edit_core::document::{Document, DocumentError};
use ropey::Rope;
use crate::keybind;
use crate::config::{CURSOR_SEMANTICS, SHOW_SAME_STATE_WARNINGS};



enum ScrollDirection{
    Up,
    Down,
    Left,
    Right
}

#[derive(Clone, PartialEq)]
pub enum Mode{
    Insert,
    Space,
    Warning(WarningKind),
    Command,
    Find,
    Goto,
}

#[derive(Clone, PartialEq, Eq)]
pub enum WarningKind{
    FileIsModified,
    FileSaveFailed,
    CommandParseFailed,
    SingleSelection,
    MultipleSelections,
    InvalidInput,
    SameState,
    UnhandledError(String)    //prob should still panic if results in an invalid state...
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
        
        instance.ui.update_layouts(instance.mode.clone());
        
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
            self.ui.update_layouts(self.mode.clone());
            self.ui.render(terminal, self.mode.clone())?;
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
                    Mode::Find => {keybind::handle_find_replace_mode_keypress(self, key_event.code, key_event.modifiers);}
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
        self.ui.highlighter.set_client_cursor_positions(self.document.view().cursor_positions(text, selections, CURSOR_SEMANTICS));
        self.ui.highlighter.selections = self.document.view().selections(selections, text);
        self.ui.status_bar.selections_widget.primary_selection_index = selections.primary_selection_index();
        self.ui.status_bar.selections_widget.num_selections = selections.count();
        self.ui.status_bar.document_cursor_position_widget.document_cursor_position = selections.primary().selection_to_selection2d(text, CURSOR_SEMANTICS).head().clone();
        self.ui.status_bar.modified_indicator_widget.document_modified_status = self.document.is_modified();
    }
    pub fn update_cursor_positions(&mut self){
        let text = self.document.text();
        let selections = self.document.selections();
        self.ui.highlighter.set_primary_cursor_position(self.document.view().primary_cursor_position(text, selections, CURSOR_SEMANTICS));
        self.ui.highlighter.set_client_cursor_positions(self.document.view().cursor_positions(text, selections, CURSOR_SEMANTICS));
        self.ui.highlighter.selections = self.document.view().selections(selections, text);
        self.ui.status_bar.selections_widget.primary_selection_index = selections.primary_selection_index();
        self.ui.status_bar.selections_widget.num_selections = selections.count();
        self.ui.status_bar.document_cursor_position_widget.document_cursor_position = selections.primary().selection_to_selection2d(text, CURSOR_SEMANTICS).head().clone()
    }
    pub fn scroll_and_update(&mut self, selection: &Selection){
        let text = self.document.text().clone();
        *self.document.view_mut() = self.document.view().scroll_following_cursor(selection, &text, CURSOR_SEMANTICS);
        self.update_ui();
    }
    pub fn checked_scroll_and_update(&mut self, selection: &Selection){
        let text = self.document.text().clone();
        if self.document.view().should_scroll(selection, &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(selection, &text, CURSOR_SEMANTICS);
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
    /////////////////////////////////////////////////////////////////////////// Reuse ////////////////////////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////////////////// Built in ////////////////////////////////////////////////////////////////////////////////
    pub fn no_op(&mut self){/* TODO: warn unbound keypress */}
    pub fn resize(&mut self, x: u16, y: u16){
        self.ui.set_terminal_size(x, y);
        self.ui.update_layouts(self.mode.clone());
        self.update_util_bar_ui();
        self.document.view_mut().set_size(self.ui.document_viewport.document_widget.rect.width as usize, self.ui.document_viewport.document_widget.rect.height as usize);
        self.scroll_and_update(&self.document.selections().primary().clone());
    }

//Insert
    //Custom Functions
    pub fn esc_handle(&mut self){
        assert!(self.mode == Mode::Insert);
        if self.ui.util_bar.utility_widget.display_copied_indicator{self.ui.util_bar.utility_widget.display_copied_indicator = false;}
        //TODO: if lsp suggestions displaying(currently unimplemented), exit that display
        else if self.document.selections().count() > 1{
            self.clear_non_primary_selections();
        }
        else if self.document.selections().primary().is_extended(CURSOR_SEMANTICS){
            self.collapse_selections();
        }
        else{
            if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}
        }
    }

    //UI Controls
    pub fn display_line_numbers(&mut self){
        assert!(self.mode == Mode::Insert);
        self.ui.document_viewport.toggle_line_numbers();
                
        self.ui.update_layouts(self.mode.clone());
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui();
    }
    pub fn display_status_bar(&mut self){
        assert!(self.mode == Mode::Insert);
        self.ui.status_bar.toggle_status_bar();
                
        self.ui.update_layouts(self.mode.clone());
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui();
    }

    //Editor Controls
    pub fn set_mode_command(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Command;
    }
    pub fn set_mode_find_replace(&mut self){    //TODO: should be set_mode_find
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Find;
    }
    pub fn set_mode_goto(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Goto;
    }
    pub fn set_mode_space(&mut self){
        assert!(self.mode == Mode::Insert);
        self.mode = Mode::Space;
    }
    pub fn save(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.save(){
            Ok(_) => {self.update_ui();}
            Err(_) => {self.mode = Mode::Warning(WarningKind::FileSaveFailed);}
        }
    }
    pub fn quit(&mut self){
        assert!(self.mode == Mode::Insert);
        //if self.ui.document_modified(){   //this is the old impl when editor was set up for client/server and state needed to be stored in ui
        if self.document.is_modified(){self.mode = Mode::Warning(WarningKind::FileIsModified);}
        else{self.should_quit = true;}
    }
    pub fn quit_ignoring_changes(&mut self){self.should_quit = true;}

    //Editing Functions
    pub fn insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Insert);
        match self.document.insert_string(&c.to_string(), CURSOR_SEMANTICS){
            Ok(_) => {self.scroll_and_update(&self.document.selections().primary().clone());}
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::InvalidInput => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
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
                //if length has changed after newline
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::InvalidInput => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
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
                    DocumentError::InvalidInput => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn delete(&mut self){   //TODO: with mulicursors, if last selection at doc end, allow other selections to delete without last showing same state error. may have to check edit_core too.
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.delete(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                //if length has changed after delete
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::SelectionAtDocBounds => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn delete_to_next_word_boundary(&mut self){ //TODO: why does this panic at doc start?
        //assert!(self.mode == Mode::Insert);
        //self.extend_selection_word_boundary_forward();
        //self.delete();
    }
    pub fn delete_to_previous_word_boundary(&mut self){ //TODO: why does this panic at doc start?
        //assert!(self.mode == Mode::Insert);
        //self.extend_selection_word_boundary_backward();
        //self.delete();
    }
    pub fn backspace(&mut self){    //TODO: with mulicursors, if first selection at doc start, allow other selections to backspace without first showing same state error. may have to check edit_core too.
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.backspace(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                //if length has changed after backspace
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::SelectionAtDocBounds => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn cut(&mut self){  //TODO: may want to trigger copied indicator ui widget here too, not just in copy
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.cut(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                //if length has changed after cut
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::SelectionAtDocBounds => {}   //TODO: figure out when this happens, and set proper warning
                    DocumentError::SelectionsError(selections_error) => {
                        match selections_error{
                            SelectionsError::MultipleSelections => {self.mode = Mode::Warning(WarningKind::MultipleSelections);}
                            _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{selections_error:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                        }
                    }
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn copy(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.copy(){
            Ok(_) => {
                self.ui.util_bar.utility_widget.display_copied_indicator = true;
                self.update_ui();
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::SelectionsError(selections_error) => {
                        match selections_error{
                            SelectionsError::MultipleSelections => {self.mode = Mode::Warning(WarningKind::MultipleSelections);}
                            _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{selections_error:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                        }
                    }
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                } 
            }
        }
    }
    pub fn paste(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.paste(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                //if length has changed after paste
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::InvalidInput => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
    pub fn undo(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.undo(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                //if length has changed after paste
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::NoChangesToUndo => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}   //TODO: should undo/redo specific error mode be added?
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn redo(&mut self){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        match self.document.redo(CURSOR_SEMANTICS){
            Ok(_) => {
                self.scroll_and_update(&self.document.selections().primary().clone());
                //if length has changed after paste
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    DocumentError::NoChangesToRedo => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}  //TODO: should undo/redo specific error mode be added?
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }

    //Selection Functions
    /// Moves/Extends single/multi cursor, and handles overlapping resultant selections
    fn move_cursor_potentially_overlapping<F>(&mut self, move_fn: F)
    where
        F: Fn(&Selection, &Rope, CursorSemantics) -> Result<Selection, SelectionError>
    {
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        let selection_count = self.document.selections().count();
        for selection in self.document.selections_mut().iter_mut(){
            match move_fn(selection, &text, CURSOR_SEMANTICS){
                Ok(new_selection) => {*selection = new_selection;}
                Err(e) => {
                    let this_file = std::panic::Location::caller().file();
                    let line_number = std::panic::Location::caller().line();
                    match e{
                        SelectionError::ResultsInSameState => {if selection_count == 1 && SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                        _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                    }
                }
            }
        }
        if selection_count > 1{
            *self.document.selections_mut() = match self.document.selections().merge_overlapping(&text, CURSOR_SEMANTICS){
                Ok(val) => val,
                Err(_) => panic!()
            };
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone());
    }
    /// Moves/Extends single/multi cursor, and handles non overlapping resultant selections
    fn move_cursor_non_overlapping<F>(&mut self, move_fn: F)
    where
        F: Fn(&Selection, &Rope, CursorSemantics) -> Result<Selection, SelectionError>
    {
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        let mut movement_succeeded = false;
        for selection in self.document.selections_mut().iter_mut(){
            match move_fn(selection, &text, CURSOR_SEMANTICS){
                Ok(new_selection) => {
                    *selection = new_selection;
                    movement_succeeded = true;
                }
                Err(e) => {
                    let this_file = std::panic::Location::caller().file();
                    let line_number = std::panic::Location::caller().line();
                    match e{
                        SelectionError::ResultsInSameState => {/*same state handled later in fn*/}
                        _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                    }
                }
            }
        }
        if !movement_succeeded && SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}
        self.checked_scroll_and_update(&self.document.selections().primary().clone());
    }
    //TODO: is this truly the desired behavior?...vs code seems to move grouped multicursors down by a page instead
    fn move_cursor_page(&mut self, movement_fn: fn(&Selection, &Rope, &View, CursorSemantics) -> Result<Selection, SelectionError>){
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
                        SelectionError::ResultsInSameState => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                        _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                    }
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone());
    }
    /// Moves/Extends single/multi cursor, and handles clearing non primary selections before move/extend
    fn move_cursor_clearing_non_primary<F>(&mut self, move_fn: F)   //TODO: this may work for move_cursor_page fns too
    where
        F: Fn(&Selection, &Rope, CursorSemantics) -> Result<Selection, SelectionError>
    {
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){
            *self.document.selections_mut() = new_selections;
            //should this do self.checked_scroll_and_update()?  
        }//intentionally ignoring any errors

        let selection = self.document.selections_mut().primary_mut();
        match move_fn(selection, &text, CURSOR_SEMANTICS){
            Ok(new_selection) => {*selection = new_selection;}
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionError::ResultsInSameState => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone());
    }
    pub fn move_cursor_up(&mut self){
        self.move_cursor_potentially_overlapping(Selection::move_up);
    }
    pub fn move_cursor_down(&mut self){
        self.move_cursor_potentially_overlapping(Selection::move_down);
    }
    pub fn move_cursor_left(&mut self){
        self.move_cursor_potentially_overlapping(Selection::move_left);
    }
    pub fn move_cursor_right(&mut self){
        self.move_cursor_potentially_overlapping(Selection::move_right);
    }
    pub fn move_cursor_word_boundary_forward(&mut self){
        self.move_cursor_potentially_overlapping(Selection::move_right_word_boundary);
    }
    pub fn move_cursor_word_boundary_backward(&mut self){
        self.move_cursor_potentially_overlapping(Selection::move_left_word_boundary);
    }
    pub fn move_cursor_line_end(&mut self){
        self.move_cursor_non_overlapping(Selection::move_line_text_end);
    }
    pub fn move_cursor_line_start(&mut self){   //TODO: rename to move_cursor_home  //also, maybe impl move_cursor_text_start and move_cursor_line_start
        self.move_cursor_non_overlapping(Selection::move_home);
    }
    pub fn move_cursor_document_start(&mut self){
        self.move_cursor_clearing_non_primary(Selection::move_doc_start);
    }
    pub fn move_cursor_document_end(&mut self){
        self.move_cursor_clearing_non_primary(Selection::move_doc_end);
    }
    pub fn move_cursor_page_up(&mut self){
        self.move_cursor_page(Selection::move_page_up);
    }
    pub fn move_cursor_page_down(&mut self){
        self.move_cursor_page(Selection::move_page_down);
    }
    pub fn extend_selection_up(&mut self){
        //self.extend_selection(Selection::extend_up);    //when multi cursor and first at doc start, non first selections move correctly and dont trigger same state warning, but selections arent merging when overlapping
        self.move_cursor_potentially_overlapping(Selection::extend_up);
    }
    pub fn extend_selection_down(&mut self){
        self.move_cursor_potentially_overlapping(Selection::extend_down);
    }
    pub fn extend_selection_left(&mut self){
        //self.extend_selection(Selection::extend_left);  //if multi cursor and first selection at doc start, we get a same state warning, when other selections should extend while first remains in place...
        self.move_cursor_potentially_overlapping(Selection::extend_left);   //this doesn't work either, but for slightly different reasons. selections extend without showing same state warning, but for some reason, they aren't merging when overlapping...only when first selection at doc start...
    }
    pub fn extend_selection_right(&mut self){
        self.move_cursor_potentially_overlapping(Selection::extend_right);
    }
    pub fn extend_selection_word_boundary_backward(&mut self){
        //self.extend_selection(Selection::extend_left_word_boundary);    //when multi cursor and first at doc start, non first selections move correctly and dont trigger same state warning, but selections arent merging when overlapping
        self.move_cursor_potentially_overlapping(Selection::extend_left_word_boundary); //same problem
    }
    pub fn extend_selection_word_boundary_forward(&mut self){
        self.move_cursor_potentially_overlapping(Selection::extend_right_word_boundary);
    }
    pub fn extend_selection_end(&mut self){
        self.move_cursor_non_overlapping(Selection::extend_line_text_end);
    }
    pub fn extend_selection_home(&mut self){
        self.move_cursor_non_overlapping(Selection::extend_home);
    }
    //pub fn extend_doc_start(&mut self){
    //    self.move_cursor_clearing_non_primary(Selection::extend_doc_start);
    //}
    //pub fn extend_doc_end(&mut self){
    //    self.move_cursor_clearing_non_primary(Selection::extend_doc_end);
    //}
    pub fn extend_selection_page_up(&mut self){ //TODO: this should prob move all cursors instead of clearing them
        self.move_cursor_page(Selection::extend_page_up);
    }
    pub fn extend_selection_page_down(&mut self){   //TODO: this should prob move all cursors instead of clearing them
        self.move_cursor_page(Selection::extend_page_down);
    }
    pub fn select_all(&mut self){
        self.move_cursor_clearing_non_primary(Selection::select_all);
    }
    pub fn collapse_selections(&mut self){
        self.move_cursor_non_overlapping(Selection::collapse);
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
                    SelectionsError::SingleSelection => {self.mode = Mode::Warning(WarningKind::SingleSelection);}  //this could be a SameState Warning
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn add_selection_above(&mut self){  //still not working quite correctly in some positions(near line endings)
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
                    SelectionsError::CannotAddSelectionAbove => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    SelectionsError::SpansMultipleLines => {self.mode = Mode::Warning(WarningKind::InvalidInput);/*TODO: extend selection up instead*/}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn add_selection_below(&mut self){  //still not working quite correctly in some positions(near line endings)
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
                    SelectionsError::CannotAddSelectionBelow => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    SelectionsError::SpansMultipleLines => {self.mode = Mode::Warning(WarningKind::InvalidInput);/*TODO: extend selection down instead*/}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn remove_primary_selection(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.selections().remove_primary_selection(){
            Ok(selections) => {
                *self.document.selections_mut() = selections;
                self.checked_scroll_and_update(&self.document.selections().primary().clone());
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.mode = Mode::Warning(WarningKind::SingleSelection);}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn increment_primary_selection(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space);
        match self.document.selections().increment_primary_selection(){
            Ok(new_selections) => {
                *self.document.selections_mut() = new_selections;
                self.scroll_and_update(&self.document.selections().primary().clone());  //should this be checked scroll and update?
                if self.mode == Mode::Space{
                    self.mode = Mode::Insert;
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.mode = Mode::Warning(WarningKind::SingleSelection);}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    pub fn decrement_primary_selection(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space);
        match self.document.selections().decrement_primary_selection(){
            Ok(new_selections) => {
                *self.document.selections_mut() = new_selections;
                self.scroll_and_update(&self.document.selections().primary().clone());  //should this be checked scroll and update?
                if self.mode == Mode::Space{
                    self.mode = Mode::Insert;
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.mode = Mode::Warning(WarningKind::SingleSelection);}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
    }
    
    //View Scroll Functions
    fn scroll_view(&mut self, direction: ScrollDirection, amount: usize){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();

        let result = match direction{
            ScrollDirection::Up => self.document.view().scroll_up(amount),
            ScrollDirection::Down => self.document.view().scroll_down(amount, &text),
            ScrollDirection::Left => self.document.view().scroll_left(amount),
            ScrollDirection::Right => self.document.view().scroll_right(amount, &text)
        };

        match result{
            Ok(new_view) => {
                *self.document.view_mut() = new_view;
                self.update_ui();
            }
            Err(e) => {
                match e{
                    ViewError::InvalidInput => {self.mode = Mode::Warning(WarningKind::InvalidInput);}
                    ViewError::ResultsInSameState => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                }
            }
        }
    }
    pub fn scroll_view_up(&mut self, amount: usize){
        self.scroll_view(ScrollDirection::Up, amount);
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
    
//Space(any fn that could be implemented in Insert mode, but are generally used from space mode)
    pub fn center_view_vertically_around_cursor(&mut self){ //TODO: still need to handle already centered cursor not showing same state warning
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space);
        let text = self.document.text().clone();
        match self.document.view().center_vertically_around_cursor(&self.document.selections().primary().clone(), &text, CURSOR_SEMANTICS){
            Ok(new_view) => {
                *self.document.view_mut() = new_view;
                self.update_ui();
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    ViewError::ResultsInSameState => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}
                    _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))
                }
            }
        }
        if self.mode == Mode::Space{self.mode = Mode::Insert;}
    }
    pub fn space_mode_exit(&mut self){
        assert!(self.mode == Mode::Space);
        self.mode = Mode::Insert;
    }
    
//Warning
    pub fn warning_mode_exit(&mut self){
        assert!(matches!(self.mode, Mode::Warning(_)));
        self.mode = Mode::Insert;
    }

//      //TODO: if num entered, then directional key pressed, move that direction that number of times
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
                    self.mode = Mode::Warning(WarningKind::InvalidInput);   //is invalid input correct here?
                }
            }else{
                self.goto_mode_exit();
                self.mode = Mode::Warning(WarningKind::InvalidInput);   //is invalid input correct here?
            }
        }else{
            self.ui.util_bar.utility_widget.text_box.clear();
            self.mode = Mode::Warning(WarningKind::InvalidInput);   //is invalid input correct here?
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
        //if !is_numeric || exceeds_doc_length{
        //    self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        //}else{
        //    self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
        //}
        self.ui.util_bar.utility_widget.text_box.text_is_valid = is_numeric && !exceeds_doc_length;
    }

//Find
    pub fn find_mode_accept(&mut self){
        assert!(self.mode == Mode::Find);
        self.document.search(&self.ui.util_bar.utility_widget.text_box.text.to_string());
        self.scroll_and_update(&self.document.selections().primary().clone());
        self.find_mode_exit();
    }
    pub fn find_mode_backspace(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.backspace();
        self.update_util_bar_ui();

        self.find_mode_text_validity_check();
    }
    pub fn find_mode_delete(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.delete();
        self.update_util_bar_ui();

        self.find_mode_text_validity_check();
    }
    pub fn find_mode_exit(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.clear();
        self.mode = Mode::Insert;
    }
    pub fn find_mode_extend_selection_end(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.extend_selection_end();
        self.update_util_bar_ui();
    }
    pub fn find_mode_extend_selection_home(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.extend_selection_home();
        self.update_util_bar_ui();
    }
    pub fn find_mode_extend_selection_left(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.extend_selection_left();
        self.update_util_bar_ui();
    }
    pub fn find_mode_extend_selection_right(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.extend_selection_right();
        self.update_util_bar_ui();
    }
    pub fn find_mode_insert_char(&mut self, c: char){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.insert_char(c);
        self.update_util_bar_ui();
        
        self.find_mode_text_validity_check();
    }
    pub fn find_mode_move_cursor_left(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.move_cursor_left();
        self.update_util_bar_ui();
    }
    pub fn find_mode_move_cursor_line_end(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_end();
        self.update_util_bar_ui();
    }
    pub fn find_mode_move_cursor_line_start(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.move_cursor_line_start();
        self.update_util_bar_ui();
    }
    pub fn find_mode_move_cursor_right(&mut self){
        assert!(self.mode == Mode::Find);
        self.ui.util_bar.utility_widget.text_box.move_cursor_right();
        self.update_util_bar_ui();
    }
    pub fn find_mode_text_validity_check(&mut self){
        assert!(self.mode == Mode::Find);
        //if self.document.text().clone().to_string().contains(&self.ui.util_bar.utility_widget.text_box.text.to_string()){
        //    self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
        //}else{
        //    self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        //}
        self.ui.util_bar.utility_widget.text_box.text_is_valid = self.document.text().clone().to_string().contains(&self.ui.util_bar.utility_widget.text_box.text.to_string());
    }

//Command
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
    pub fn command_mode_accept(&mut self){
        assert!(self.mode == Mode::Command);
        if self.parse_command(&self.ui.util_bar.utility_widget.text_box.text.to_string()).is_ok(){
            self.command_mode_exit();
        }else{
            self.command_mode_exit();
            self.mode = Mode::Warning(WarningKind::CommandParseFailed);
        }
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
    /////////////////////////////////////////////////////////////////////////// Built in ////////////////////////////////////////////////////////////////////////////////
}

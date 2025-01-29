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
use crate::config::{CURSOR_SEMANTICS, SHOW_SAME_STATE_WARNINGS, VIEW_SCROLL_AMOUNT};



pub enum UtilAction{
    Backspace,
    Delete,
    InsertChar(char),
    ExtendEnd,
    ExtendHome,
    ExtendLeft,
    ExtendRight,
    MoveEnd,
    MoveHome,
    MoveLeft,
    MoveRight,
}
pub enum ViewAction{
    CenterVerticallyAroundCursor,
    //CenterHorizontallyAroundCursor,
    //AlignWithCursorAtTop,
    //AlignWithCursorAtBottom,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}
pub enum EditAction{
    InsertChar(char),
    InsertNewline,
    InsertTab,
    Delete,
    //DeleteToNextWordBoundary,
    //DeleteToPrevWordBoundary,
    Backspace,
    Cut,
    Paste,
    Undo,
    Redo
}

#[derive(Clone, PartialEq)]
pub enum Mode{
    /// for editing text and moving/extending selections
    Insert,
    /// for display of warnings/errors. blocks input until mode exited.
    Warning(WarningKind),   //maybe same state warnings should be in notify, so they don't block
    /// for display of notifications. does not block input.
    Notify,  //could be used for text copied indicator, etc.. could also do "action performed outside of view" for non-visible actions
    Space,  //honestly, View mode could prob just replace this for now...
    //View,  //adjust view with single input keybinds   (directional scrolling wouldn't exit mode, but center view and other fns might)     //i think view should be a pop-up style mode
    /// for jumping to specified line number    //potentially more in the future...
    Goto,
    /// for issuing editor commands
    Command,
    // all selection manipulation with regex matching can be sub modes of a mode called Match. this would be a popup mode, that then triggers the interactive text box when sub mode entered
    /// for selecting any matching regex from inside selections
    Find,
    /// for retaining everything within selections that isn't a matching regex pattern
    Split,
    // select text within but excluding instances of a single search pattern, a char pair, or a text object
    //SelectInside,
    //select text within and including instances of a single search pattern, a char pair, or a text object
    //SelectIncluding,
    //select the next occurring instance of a search pattern
    //SearchNextAhead,
    //select the prev occurring instance of a search pattern
    //SearchPrevBehind
    //select until the next occuring instance of a search pattern
    //SearchUntilAhead,
    //select until the prev occuring instance of a search pattern
    //SearchUntilBehind,
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
    UnhandledError(String),    //prob should still panic if results in an invalid state...
    UnhandledKeypress,
    UnhandledEvent
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
        
        instance.ui.update_layouts(&instance.mode);
        //init backend doc view size
        instance.document.view_mut().set_size(
            instance.ui.document_viewport.document_widget.rect.width as usize,
            instance.ui.document_viewport.document_widget.rect.height as usize
        );

        // prefer this over scroll_and_update, even when response fns are the same, because it saves us from unnecessarily reassigning the view
        instance.checked_scroll_and_update(&instance.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_document);

        Ok(instance)
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn Error>>{
        loop{
            self.ui.update_layouts(&self.mode);
            self.ui.render(terminal, &self.mode)?;
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
                    Mode::Notify => {
                        // changes mode back to Insert, without updating UI, so notifications show until next keypress
                        if self.mode == Mode::Notify{self.mode = Mode::Insert;} //ensure we return back to insert mode
                        keybind::handle_insert_mode_keypress(self, key_event.code, key_event.modifiers);
                    }
                    Mode::Split => {keybind::handle_split_mode_keypress(self, key_event.code, key_event.modifiers);}
                }
            },
            event::Event::Resize(x, y) => self.resize(x, y),
            _ => self.no_op_event(),
        }

        Ok(())
    }

    // could make separate files for categories of fns. builtin.rs and custom.rs...       custom::escape_handle()     builtin::add_selection_above()
    // or all in one commands.rs file?...
    /////////////////////////////////////////////////////////////////////////// Reuse ////////////////////////////////////////////////////////////////////////////////
    
    /// Set all data related to document viewport UI.
    pub fn update_ui_data_document(&mut self){
        let text = self.document.text();
        
        self.ui.document_viewport.document_widget.text_in_view = self.document.view().text(text);
        self.ui.document_viewport.line_number_widget.line_numbers_in_view = self.document.view().line_numbers(text);
        self.update_ui_data_selections();
        self.ui.status_bar.modified_indicator_widget.document_modified_status = self.document.is_modified();
    }
    /// Set only data related to selections in document viewport UI.
    pub fn update_ui_data_selections(&mut self){
        let text = self.document.text();
        let selections = self.document.selections();
        
        self.ui.document_viewport.highlighter.set_primary_cursor_position(self.document.view().primary_cursor_position(text, selections, CURSOR_SEMANTICS));
        self.ui.document_viewport.highlighter.set_client_cursor_positions(self.document.view().cursor_positions(text, selections, CURSOR_SEMANTICS));
        self.ui.document_viewport.highlighter.selections = self.document.view().selections(selections, text);
        self.ui.status_bar.selections_widget.primary_selection_index = selections.primary_selection_index();
        self.ui.status_bar.selections_widget.num_selections = selections.count();
        self.ui.status_bar.document_cursor_position_widget.document_cursor_position = selections.primary().selection_to_selection2d(text, CURSOR_SEMANTICS).head().clone();
    }
    // prefer this over checked_scroll_and_update only when the view should ALWAYS scroll.      //TODO: comment out this fn, and verify all callers actually need this fn and not checked_scroll_and_update
    //pub fn scroll_and_update(&mut self, selection: &Selection){ //TODO: maybe scrolling should be separate from scrolling?...
    //    let text = self.document.text().clone();
    //    *self.document.view_mut() = self.document.view().scroll_following_cursor(selection, &text, CURSOR_SEMANTICS);
    //    self.update_ui_data_document();
    //}
    // prefer this over scroll_and_update, even when response fns are the same, because it saves us from unnecessarily reassigning the view
    pub fn checked_scroll_and_update<F, A>(&mut self, cursor_to_follow: &Selection, scroll_response_fn: F, non_scroll_response_fn: A)
        where F: Fn(&mut Application), A: Fn(&mut Application)
    {
        let text = self.document.text().clone();
        if self.document.view().should_scroll(cursor_to_follow, &text, CURSOR_SEMANTICS){
            *self.document.view_mut() = self.document.view().scroll_following_cursor(cursor_to_follow, &text, CURSOR_SEMANTICS);
            scroll_response_fn(self);
        }else{
            non_scroll_response_fn(self);
        }
    }
    pub fn update_ui_data_util_bar(&mut self){
        let text = self.ui.util_bar.utility_widget.text_box.text.clone();
        let selections = Selections::new(vec![self.ui.util_bar.utility_widget.text_box.selection.clone()], 0, &text);
        self.ui.util_bar.utility_widget.text_box.view = self.ui.util_bar.utility_widget.text_box.view.scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
    }
    /////////////////////////////////////////////////////////////////////////// Reuse ////////////////////////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////////////////// Built in ////////////////////////////////////////////////////////////////////////////////
    pub fn no_op_keypress(&mut self){
        self.mode = Mode::Warning(WarningKind::UnhandledKeypress);
        // needed to handle warning mode set when text in util_bar
        self.ui.util_bar.utility_widget.text_box.clear();
    }
    pub fn no_op_event(&mut self){
        self.mode = Mode::Warning(WarningKind::UnhandledEvent);
        // needed to handle warning mode set when text in util_bar
        self.ui.util_bar.utility_widget.text_box.clear();
    }
    pub fn resize(&mut self, x: u16, y: u16){
        self.ui.set_terminal_size(x, y);
        self.ui.update_layouts(&self.mode);
        self.update_ui_data_util_bar(); //TODO: can this be called later in fn impl?
        // ui layouts need to be updated before doc size set, so doc size can be calculated correctly
        self.document.view_mut().set_size(self.ui.document_viewport.document_widget.rect.width as usize, self.ui.document_viewport.document_widget.rect.height as usize);
        // scrolling so cursor is in a reasonable place, and updating so any ui changes render correctly
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_document);
    }

//Insert
    //Custom Functions
    pub fn esc_handle(&mut self){
        assert!(self.mode == Mode::Insert);
        //if self.ui.util_bar.utility_widget.display_copied_indicator{self.ui.util_bar.utility_widget.display_copied_indicator = false;}
        //TODO: if lsp suggestions displaying(currently unimplemented), exit that display
        /*else */if self.document.selections().count() > 1{
            self.clear_non_primary_selections();
        }
        else if self.document.selections().primary().is_extended(CURSOR_SEMANTICS){
            self.collapse_selections();
        }
        else{
            if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}
        }
    }

    //Editor Controls
    pub fn set_mode(&mut self, to_mode: Mode){
        self.mode = to_mode.clone();
        
        let mut to_mode_uses_util_text = false;
        let mut update_layouts_and_document = false;
        let mut store_current_selections = false;
        match to_mode{
            Mode::Space | Mode::Insert | Mode::Notify | Mode::Warning(_)=> {/*do nothing*/}     //note: it seems setting to insert mode doesn't always do nothing. util_modes make it do other things
            Mode::Goto | Mode::Command => {
                to_mode_uses_util_text = true;
                update_layouts_and_document = true;
            }
            Mode::Find | Mode::Split => {
                to_mode_uses_util_text = true;
                update_layouts_and_document = true;
                store_current_selections = true;
            }
        }

        if to_mode_uses_util_text{self.ui.util_bar.utility_widget.text_box.clear();}
        if update_layouts_and_document{
            self.ui.update_layouts(&self.mode);
            self.document.view_mut().set_size(
                self.ui.document_viewport.document_widget.rect.width as usize,
                self.ui.document_viewport.document_widget.rect.height as usize
            );
            self.update_ui_data_document();
        }
        if store_current_selections{
            self.ui.util_bar.utility_widget.selections_before_search = Some(self.document.selections().clone());
        }
    }
    pub fn quit(&mut self){
        assert!(self.mode == Mode::Insert);
        //if self.ui.document_modified(){   //this is the old impl when editor was set up for client/server and state needed to be stored in ui
        if self.document.is_modified(){self.set_mode(Mode::Warning(WarningKind::FileIsModified));}
        else{self.should_quit = true;}
    }
    pub fn quit_ignoring_changes(&mut self){self.should_quit = true;}

    //Document related functions
    pub fn save(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.save(){
            Ok(()) => {self.update_ui_data_document();}
            Err(_) => {self.set_mode(Mode::Warning(WarningKind::FileSaveFailed));}
        }
    }
    fn handle_document_error(&mut self, e: DocumentError){
        let this_file = std::panic::Location::caller().file();
        let line_number = std::panic::Location::caller().line();
        match e{
            DocumentError::InvalidInput => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
            DocumentError::SelectionAtDocBounds => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
            DocumentError::NoChangesToUndo => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
            DocumentError::NoChangesToRedo => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
            DocumentError::SelectionsError(s) => {
                match s{
                    SelectionsError::MultipleSelections => {self.set_mode(Mode::Warning(WarningKind::MultipleSelections));}
                    _ => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{s:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))),
                }
            }
        }
    }
    pub fn copy(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.copy(){
            Ok(()) => {
                self.set_mode(Mode::Notify);
                self.update_ui_data_document(); //TODO: is this really needed for something?...
            }
            Err(e) => {self.handle_document_error(e);}
        }
    }
    pub fn edit_action(&mut self, action: EditAction){
        assert!(self.mode == Mode::Insert);
        let len = self.document.len();
        
        let result = match action{
            EditAction::InsertChar(c) => self.document.insert_string(&c.to_string(), CURSOR_SEMANTICS),
            EditAction::InsertNewline => self.document.insert_string("\n", CURSOR_SEMANTICS),
            EditAction::InsertTab => self.document.insert_string("\t", CURSOR_SEMANTICS),
            EditAction::Delete => self.document.delete(CURSOR_SEMANTICS),
            EditAction::Backspace => self.document.backspace(CURSOR_SEMANTICS),
            EditAction::Cut => self.document.cut(CURSOR_SEMANTICS),
            EditAction::Paste => self.document.paste(CURSOR_SEMANTICS),
            EditAction::Undo => self.document.undo(CURSOR_SEMANTICS),   // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
            EditAction::Redo => self.document.redo(CURSOR_SEMANTICS)
        };

        match result{
            Ok(()) => {
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_document);
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {self.handle_document_error(e);}
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
                        _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))    //TODO: figure out how to make this use new set_mode method...
                    }
                }
            }
        }
        if selection_count > 1{
            *self.document.selections_mut() = if let Ok(val) = self.document.selections().merge_overlapping(&text, CURSOR_SEMANTICS){val}else{panic!()};
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
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
                        _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))    //TODO: figure out how to make this use new set_mode method...
                    }
                }
            }
        }
        //if !movement_succeeded && SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}
        if !movement_succeeded && SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
    }
    //TODO: is this truly the desired behavior?...vs code seems to move grouped multicursors down by a page instead
    //TODO: maybe the behavior should be more like move_cursor_potentially_overlapping?...
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
                        SelectionError::ResultsInSameState => {if SHOW_SAME_STATE_WARNINGS{self.mode = Mode::Warning(WarningKind::SameState);}}     //TODO: figure out how to make this use new set_mode method...
                        _ => self.mode = Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))    //TODO: figure out how to make this use new set_mode method...
                    }
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
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
                    SelectionError::ResultsInSameState => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
                    _ => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))),
                }
            }
        }
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
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
        self.move_cursor_potentially_overlapping(Selection::extend_up);
    }
    pub fn extend_selection_down(&mut self){
        self.move_cursor_potentially_overlapping(Selection::extend_down);
    }
    pub fn extend_selection_left(&mut self){
        self.move_cursor_potentially_overlapping(Selection::extend_left);
    }
    pub fn extend_selection_right(&mut self){
        self.move_cursor_potentially_overlapping(Selection::extend_right);
    }
    pub fn extend_selection_word_boundary_backward(&mut self){
        self.move_cursor_potentially_overlapping(Selection::extend_left_word_boundary);
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
    pub fn select_line(&mut self){
        self.move_cursor_non_overlapping(Selection::select_line);
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
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.set_mode(Mode::Warning(WarningKind::SingleSelection));}   //this could be a SameState Warning
                    _ => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here."))))
                }
            }
        }
    }
    pub fn add_selection_above(&mut self){
        assert!(self.mode == Mode::Insert);
        let text = self.document.text().clone();
        match self.document.selections().add_selection_above(&text, CURSOR_SEMANTICS){
            Ok(selections) => {
                *self.document.selections_mut() = selections;
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::CannotAddSelectionAbove => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
                    SelectionsError::SpansMultipleLines => {self.set_mode(Mode::Warning(WarningKind::InvalidInput));} //TODO: extend selection up instead?...
                    _ => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here."))))
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
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::CannotAddSelectionBelow => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
                    SelectionsError::SpansMultipleLines => {self.set_mode(Mode::Warning(WarningKind::InvalidInput));} //TODO: extend selection down instead?...
                    _ => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here."))))
                }
            }
        }
    }
    pub fn remove_primary_selection(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.selections().remove_primary_selection(){
            Ok(selections) => {
                *self.document.selections_mut() = selections;
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.set_mode(Mode::Warning(WarningKind::SingleSelection));}
                    _ => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here."))))
                }
            }
        }
    }
    pub fn increment_primary_selection(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space);
        match self.document.selections().increment_primary_selection(){
            Ok(new_selections) => {
                *self.document.selections_mut() = new_selections;
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
                if self.mode == Mode::Space{
                    self.set_mode(Mode::Insert);
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.set_mode(Mode::Warning(WarningKind::SingleSelection));}
                    _ => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here."))))
                }
            }
        }
    }
    pub fn decrement_primary_selection(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space);
        match self.document.selections().decrement_primary_selection(){
            Ok(new_selections) => {
                *self.document.selections_mut() = new_selections;
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
                if self.mode == Mode::Space{
                    self.set_mode(Mode::Insert);
                }
            }
            Err(e) => {
                let this_file = std::panic::Location::caller().file();
                let line_number = std::panic::Location::caller().line();
                match e{
                    SelectionsError::SingleSelection => {self.set_mode(Mode::Warning(WarningKind::SingleSelection));}
                    _ => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{e:#?} at {this_file}::{line_number}. This Error shouldn't be possible here."))))
                }
            }
        }
    }
    
//View
    pub fn view_action(&mut self, action: ViewAction){      //TODO: make separate view mode, and call this from there
        assert!(self.mode == Mode::Insert || self.mode == Mode::Space); //TODO: assert!(self.mode == Mode::View);
        let view = self.document.view();

        let result = match action{
            ViewAction::CenterVerticallyAroundCursor => {view.center_vertically_around_cursor(self.document.selections().primary(), self.document.text(), CURSOR_SEMANTICS)}
            ViewAction::ScrollUp => {view.scroll_up(VIEW_SCROLL_AMOUNT)}
            ViewAction::ScrollDown => {view.scroll_down(VIEW_SCROLL_AMOUNT, self.document.text())}
            ViewAction::ScrollLeft => {view.scroll_left(VIEW_SCROLL_AMOUNT)}
            ViewAction::ScrollRight => {view.scroll_right(VIEW_SCROLL_AMOUNT, self.document.text())}
        };
        match result{
            Ok(new_view) => {
                *self.document.view_mut() = new_view;
                self.update_ui_data_document();
            }
            Err(e) => {
                match e{
                    ViewError::InvalidInput => {self.set_mode(Mode::Warning(WarningKind::InvalidInput));}
                    ViewError::ResultsInSameState => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
                }
            }
        }
        if self.mode == Mode::Space{    //TODO: will have to change how this works when Mode::View implemented
            self.set_mode(Mode::Insert);
        }
    }
    
//Space(any fn that could be implemented in Insert mode, but are generally used from space mode)
    
//Warning

//Goto  //TODO: if num entered, then directional key pressed, move that direction that number of times
    pub fn goto_mode_accept(&mut self){
        assert!(self.mode == Mode::Goto);
        
        let mut show_warning = false;
        if let Ok(line_number) = self.ui.util_bar.utility_widget.text_box.text.to_string().parse::<usize>(){
            if line_number == 0{show_warning = true;}   //we have no line number 0, so this is invalid
            else{
                // make line number 0 based for interfacing correctly with backend impl
                let line_number = line_number.saturating_sub(1);
                
                // clears non primary, if more than one selection. otherwise, does nothing.
                if let Ok(new_selections) = self.document.selections().clear_non_primary_selections(){*self.document.selections_mut() = new_selections;}
                
                // go to line number
                if let Ok(new_selection) = self.document.selections().primary().set_from_line_number(line_number, self.document.text(), Movement::Move, CURSOR_SEMANTICS){
                    *self.document.selections_mut().primary_mut() = new_selection;
                    self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_selections, Application::update_ui_data_selections);
                }else{show_warning = true;}
            }
        }else{show_warning = true;}
        
        self.goto_mode_exit();
        if show_warning{
            self.set_mode(Mode::Warning(WarningKind::InvalidInput));
        }
    }
    pub fn goto_mode_backspace(&mut self){
        self.generic_util_action(Mode::Goto, UtilAction::Backspace);
        self.goto_mode_text_validity_check();
    }
    pub fn goto_mode_delete(&mut self){
        self.generic_util_action(Mode::Goto, UtilAction::Delete);
        self.goto_mode_text_validity_check();
    }
    pub fn goto_mode_exit(&mut self){
        assert!(self.mode == Mode::Goto);
        self.set_mode(Mode::Insert);
        //
        self.ui.update_layouts(&self.mode);
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
        //
    }
    pub fn goto_mode_insert_char(&mut self, c: char){
        self.generic_util_action(Mode::Goto, UtilAction::InsertChar(c));
        self.goto_mode_text_validity_check();
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
        self.ui.util_bar.utility_widget.text_box.text_is_valid = is_numeric && !exceeds_doc_length;
    }

//Find
    fn incremental_search(&mut self){
        if let Some(selections) = self.ui.util_bar.utility_widget.selections_before_search.clone(){
            if let Ok(selections) = selections.search(&self.ui.util_bar.utility_widget.text_box.text.to_string(), self.document.text()){
                self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
                *self.document.selections_mut() = selections;
            }else{  //TODO: may want to match on error to make sure we are handling this correctly
                self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
                // make sure this is the desired behavior...
                *self.document.selections_mut() = self.ui.util_bar.utility_widget.selections_before_search.clone().unwrap();
                //
            }
        }
    }
    pub fn find_mode_backspace(&mut self){
        self.generic_util_action(Mode::Find, UtilAction::Backspace);
        self.incremental_search();
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
    }
    pub fn find_mode_delete(&mut self){
        self.generic_util_action(Mode::Find, UtilAction::Delete);
        self.incremental_search();
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
    }
    pub fn find_mode_exit(&mut self){
        assert!(self.mode == Mode::Find);
        self.set_mode(Mode::Insert);
        //
        self.ui.update_layouts(&self.mode);
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
        //
    }
    pub fn find_mode_insert_char(&mut self, c: char){
        self.generic_util_action(Mode::Find, UtilAction::InsertChar(c));
        self.incremental_search();
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
    }

//Split
    fn incremental_split(&mut self){
        if let Some(selections) = self.ui.util_bar.utility_widget.selections_before_search.clone(){
            //if let Ok(selections) = selections.search(&self.ui.util_bar.utility_widget.text_box.text.to_string(), self.document.text()){
            if let Ok(selections) = selections.split(&self.ui.util_bar.utility_widget.text_box.text.to_string(), self.document.text()){
                self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
                *self.document.selections_mut() = selections;
            }else{  //TODO: may want to match on error to make sure we are handling this correctly
                self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
                // make sure this is the desired behavior...
                *self.document.selections_mut() = self.ui.util_bar.utility_widget.selections_before_search.clone().unwrap();
                //
            }
        }
    }
    pub fn split_mode_backspace(&mut self){
        self.generic_util_action(Mode::Split, UtilAction::Backspace);
        self.incremental_split();
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
    }
    pub fn split_mode_delete(&mut self){
        self.generic_util_action(Mode::Split, UtilAction::Delete);
        self.incremental_split();
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
    }
    pub fn split_mode_exit(&mut self){
        assert!(self.mode == Mode::Split);
        self.set_mode(Mode::Insert);
        //
        self.ui.update_layouts(&self.mode);
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
        //
    }
    pub fn split_mode_insert_char(&mut self, c: char){
        self.generic_util_action(Mode::Split, UtilAction::InsertChar(c));
        self.incremental_split();
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
    }

//Command
    pub fn toggle_line_numbers(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Command);
        self.ui.document_viewport.toggle_line_numbers();
                
        self.ui.update_layouts(&self.mode);
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
    }
    pub fn toggle_status_bar(&mut self){
        assert!(self.mode == Mode::Insert || self.mode == Mode::Command);
        self.ui.status_bar.toggle_status_bar();
                
        self.ui.update_layouts(&self.mode);
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
    }
    pub fn open_new_terminal_window(&self){
        let _ = std::process::Command::new("alacritty")     //TODO: have user define TERMINAL const in config.rs   //or check env vars for $TERM?
            //.arg("msg")     // these extra commands just make new instances use the same backend(daemon?)
            //.arg("create-window")
            //.current_dir(std::env::current_dir().unwrap())    //not needed here, because term spawned here defaults to this directory, but good to know
            .spawn()
            .expect("failed to spawn new terminal at current directory");
    }
    pub fn command_mode_accept(&mut self){
        assert!(self.mode == Mode::Command);
        let mut warn = false;
        match self.ui.util_bar.utility_widget.text_box.text.to_string().as_str(){
            "term" | "t" => {self.open_new_terminal_window();}
            "toggle_line_numbers" | "ln" => {self.toggle_line_numbers();}
            "toggle_status_bar" | "sb" => {self.toggle_status_bar();}
            _ => {warn = true;}
        }
        self.command_mode_exit();
        if warn{self.mode = Mode::Warning(WarningKind::CommandParseFailed);}
    }
    pub fn command_mode_exit(&mut self){
        assert!(self.mode == Mode::Command);
        self.set_mode(Mode::Insert);
        //
        self.ui.update_layouts(&self.mode);
        self.document.view_mut().set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
        //
    }

//Generic Util Functionality
    pub fn generic_util_action(&mut self, mode: Mode, action: UtilAction){
        assert!(self.mode == mode); //TODO: verify if this is really necessary...
        let text_box = &mut self.ui.util_bar.utility_widget.text_box;
        match action{
            UtilAction::Backspace => text_box.backspace(),
            UtilAction::Delete => text_box.delete(),
            UtilAction::InsertChar(c) => text_box.insert_char(c),
            UtilAction::ExtendEnd => text_box.extend_selection_end(),
            UtilAction::ExtendHome => text_box.extend_selection_home(),
            UtilAction::ExtendLeft => text_box.extend_selection_left(),
            UtilAction::ExtendRight => text_box.extend_selection_right(),
            UtilAction::MoveEnd => text_box.move_cursor_line_end(),
            UtilAction::MoveHome => text_box.move_cursor_line_start(),
            UtilAction::MoveLeft => text_box.move_cursor_left(),
            UtilAction::MoveRight => text_box.move_cursor_right(),
        }
        self.update_ui_data_util_bar();
    }
    /////////////////////////////////////////////////////////////////////////// Built in ////////////////////////////////////////////////////////////////////////////////
}

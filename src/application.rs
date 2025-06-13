//TODO: figure out how to launch another terminal session, start another edit session, and pass it text via stdin
    //let _ = std::process::Command::new("alacritty")
    //                    .args(&["-e", "bash", "-c", "<program to run>"])
    //                    .spawn()
    //                    .expect("Failed to launch Alacritty");

//TODO: implement desired kakoune/sam/acme features here first, then design edit with client/server architecture with filesystem ipc...

//TODO: if error message displayed in scratch buffer, select filename and error location, trigger goto command(acme mouse right click, not the built-in goto-mode...).
// ex: file_name.rs:10:22
// if the buffer with filename is open in session, and is the active buffer for this client, go to that location in the buffer
// if the buffer with filename is open in session, but is not the active buffer for this client, set it as the active buffer for this client, and go to that location in the buffer
// if the buffer with filename not open in session, open that file in session, set it as the active buffer for this client, and go to that location in the buffer
// TODO: if used with multiple edit clients, may require integration with the window manager to set focus to the window of a specific edit client...

//TODO: if buffer history(undo/redo) implement a display method, edit could expose those as command expansion variables/tags
// ex: no_op %sh{echo %var{history} | edit -p} or no_op %sh{edit -p < %var{history}}      //no_op means edit will not evaluate any output resulting from the run of the shell session
// this would pipe a displayable version of the buffer's undo/redo stack(s)/tree(s) in a scratch buffer in a new edit client
// alternatively, if we impl the filesystem approach, user would just pipe tmp/edit/sessions/session_id/buffers/buffer_id/history to edit -p
// ex: no_op %sh{edit -p < tmp/edit/sessions/%var{session_id}/buffers/%var{buffer_id}/history}

// I don't want my editor implementation to also be a file system explorer, and so will not provide capabilities for navigating the
// file system from within the editor, or opening buffers from within the editor.
// user can navigate to a file in a terminal(or some external application) and open that buffer in the editor, 
// or pass text from a terminal(or some external application) in to a scratch buffer in the editor
// but the eventual client/server design shouldn't be restricted from doing so, if the user desires. it just won't be supported
// as a built in feature...
// we should prob support switching to other open buffers within a session inside the editor, though

use std::error::Error;
use std::path::PathBuf;
use crossterm::event;
use ratatui::layout::Rect;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::keybind;
use crate::config::{CURSOR_SEMANTICS, SHOW_SAME_STATE_WARNINGS, VIEW_SCROLL_AMOUNT, USE_HARD_TAB, TAB_WIDTH, USE_FULL_FILE_PATH};



pub enum ApplicationError{
    ReadOnlyBuffer,
    InvalidInput,
    SelectionAtDocBounds,
    NoChangesToUndo,
    NoChangesToRedo,
    SelectionsError(crate::selections::SelectionsError),
}
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
    //AlignSelectedTextVertically,
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
    Redo,
    //SwapUp,   (if text selected, swap selected text with line above. if no selection, swap current line with line above)
    //SwapDown, (if text selected, swap selected text with line below. if no selection, swap current line with line below)
    //RotateTextInSelections,
    AddSurround(char, char),
}
pub enum SelectionAction{
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorWordBoundaryForward,
    MoveCursorWordBoundaryBackward,
    MoveCursorLineEnd,
    MoveCursorHome,
    MoveCursorBufferStart,
    MoveCursorBufferEnd,
    MoveCursorPageUp,
    MoveCursorPageDown,
    ExtendSelectionUp,
    ExtendSelectionDown,
    ExtendSelectionLeft,
    ExtendSelectionRight,
    ExtendSelectionWordBoundaryBackward,
    ExtendSelectionWordBoundaryForward,
    ExtendSelectionLineEnd,
    ExtendSelectionHome,
    //ExtendSelectionBufferStart,
    //ExtendSelectionBufferEnd,
    //ExtendSelectionPageUp,
    //ExtendSelectionPageDown,
    SelectLine,
    SelectAll,
    CollapseSelections,
    ClearNonPrimarySelections,
    AddSelectionAbove,
    AddSelectionBelow,
    RemovePrimarySelection,
    IncrementPrimarySelection,
    DecrementPrimarySelection,
    Surround,
    SurroundingPair,
    FlipDirection
}

#[derive(Clone, PartialEq)]
pub enum Mode{
    /// for editing text and moving/extending selections
    Insert,
    /// for display of warnings/errors. blocks input until mode exited.
    Warning(WarningKind),   //maybe same state warnings should be in notify, so they don't block
    /// for display of notifications. does not block input.
    Notify,  //could be used for text copied indicator, etc.. could also do "action performed outside of view" for non-visible actions
    /// for adjusting the visible area of text
    View,
    /// for jumping to specified line number    //potentially more in the future...
    Goto,
    /// for issuing editor commands
    Command,
    /// for selecting any matching regex from inside selections
    Find,
    /// for retaining everything within selections that isn't a matching regex pattern
    Split,
    /// for selecting text objects
    Object,
    /// for inserting bracket pairs around selection(s) contents
    AddSurround,    //maybe change to AddSurroundingPair or AddBracketPair

    // NOTE: may not ever implement the following, but good to think about...
    //select the next occurring instance of a search pattern
    //SearchNextAhead,
    //select the prev occurring instance of a search pattern
    //SearchPrevBehind
    //select until the next occuring instance of a search pattern
    //SelectUntilNext,
    //select until the prev occuring instance of a search pattern
    //SelectUntilPrev,
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
    mode_stack: Vec<Mode>,
    ui: crate::ui::UserInterface,

    pub buffer: crate::buffer::Buffer,      //TODO?: BufferType? File|Scratch   //buffer type is already encoded in the file_path on Buffer being optional. if file_path == None, the buffer is a scratch buffer

    pub undo_stack: Vec<crate::history::ChangeSet>,   //maybe have separate buffer and selections undo/redo stacks?...
    pub redo_stack: Vec<crate::history::ChangeSet>,
    pub selections: crate::selections::Selections,
    pub view: crate::view::View,
    pub clipboard: String,
}
impl Application{
    #[cfg(test)] pub fn new_test_app(buffer_text: &str, file_path: Option<PathBuf>, read_only: bool, view: &crate::view::View) -> Self{
        let buffer = crate::buffer::Buffer::new(buffer_text, file_path.clone(), read_only);
        let mut instance = Self{
            should_quit: false,
            mode_stack: vec![Mode::Insert],
            ui: crate::ui::UserInterface::new(Rect::new(view.horizontal_start as u16, view.vertical_start as u16, view.width as u16, view.height as u16)),
            buffer: buffer.clone(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selections: crate::selections::Selections::new(
                vec![
                    crate::selection::Selection::new_from_range(
                        crate::range::Range::new(0, 1), 
                        crate::selection::ExtensionDirection::None, 
                        &buffer, 
                        CURSOR_SEMANTICS)
                ], 
                0, 
                &buffer, 
                CURSOR_SEMANTICS
            ),
            view: crate::view::View::new(0, 0, 0, 0),
            clipboard: String::new()
        };

        instance.ui.status_bar.file_name_widget.file_name = if USE_FULL_FILE_PATH{
            instance.buffer.file_path()
        }else{
            instance.buffer.file_name()
        };
        
        instance.ui.document_viewport.document_widget.doc_length = instance.buffer.len_lines();
        
        instance.ui.update_layouts(&instance.mode());
        //init backend doc view size
        instance.view.set_size(
            instance.ui.document_viewport.document_widget.rect.width as usize,
            instance.ui.document_viewport.document_widget.rect.height as usize
        );

        // prefer this over scroll_and_update, even when response fns are the same, because it saves us from unnecessarily reassigning the view
        instance.checked_scroll_and_update(
            &instance.selections.primary().clone(), 
            Application::update_ui_data_document, 
            Application::update_ui_data_document
        );

        instance
    }
    pub fn new(buffer_text: &str, file_path: Option<PathBuf>, read_only: bool, terminal: &Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<Self, Box<dyn Error>>{
        let terminal_size = terminal.size()?;
        let terminal_rect = Rect::new(0, 0, terminal_size.width, terminal_size.height);

        let buffer = crate::buffer::Buffer::new(buffer_text, file_path.clone(), read_only);
        let mut instance = Self{
            should_quit: false,
            mode_stack: vec![Mode::Insert],
            ui: crate::ui::UserInterface::new(terminal_rect),
            buffer: buffer.clone(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selections: crate::selections::Selections::new(
                vec![
                    crate::selection::Selection::new_from_range(
                        crate::range::Range::new(0, /*1*/buffer.next_grapheme_boundary_index(0)), 
                        crate::selection::ExtensionDirection::None, 
                        &buffer, 
                        CURSOR_SEMANTICS)
                ], 
                0, 
                &buffer, 
                CURSOR_SEMANTICS
            ),
            view: crate::view::View::new(0, 0, 0, 0),
            clipboard: String::new()
        };
        
        instance.ui.status_bar.file_name_widget.file_name = if USE_FULL_FILE_PATH{
            instance.buffer.file_path()
        }else{
            instance.buffer.file_name()
        };
        
        instance.ui.document_viewport.document_widget.doc_length = instance.buffer.len_lines();
        
        instance.ui.update_layouts(&instance.mode());
        //init backend doc view size
        instance.view.set_size(
            instance.ui.document_viewport.document_widget.rect.width as usize,
            instance.ui.document_viewport.document_widget.rect.height as usize
        );

        // prefer this over scroll_and_update, even when response fns are the same, because it saves us from unnecessarily reassigning the view
        //instance.checked_scroll_and_update(&instance.document.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_document);
        instance.checked_scroll_and_update(
            &instance.selections.primary().clone(), 
            Application::update_ui_data_document, 
            Application::update_ui_data_document
        );

        Ok(instance)
    }
    
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn Error>>{
        loop{
            self.ui.update_layouts(&self.mode());
            self.ui.render(terminal, &self.mode())?;
            self.handle_event()?;
            if self.should_quit{
                return Ok(());
            }
        }
    }

    fn handle_event(&mut self) -> Result<(), Box<dyn Error>>{
        match event::read()?{
            //TODO: handle_keypress fns could take a mode as context, then mode specific functionality wouldn't need to be in separate fns...
            //that context could also be used to fill available commands in mode specific popup menus
            event::Event::Key(key_event) => {
                match self.mode(){
                    Mode::Insert => {keybind::handle_insert_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::View => {keybind::handle_view_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Warning(_) => {keybind::handle_warning_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Goto => {keybind::handle_goto_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Find => {keybind::handle_find_replace_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Command => {keybind::handle_command_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Notify => {
                        //unhandled keybinds in notify mode fall through to insert mode //TODO: do the same for suggestions mode(not impled yet)
                        keybind::handle_notify_mode_keypress(self, key_event.code, key_event.modifiers);
                    }
                    Mode::Split => {keybind::handle_split_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Object => {keybind::handle_object_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::AddSurround => {keybind::handle_add_surround_mode_keypress(self, key_event.code, key_event.modifiers);}
                }
            },
            event::Event::Mouse(idk) => {
                //TODO: maybe mode specific mouse handling...
                match idk.kind{
                    event::MouseEventKind::Down(_) => {/*self.no_op_event();*/}
                    event::MouseEventKind::Up(_) => {/*self.no_op_event();*/}
                    event::MouseEventKind::Drag(_) => {/*self.no_op_event();*/}
                    event::MouseEventKind::Moved => {/*self.no_op_event();*/}
                    event::MouseEventKind::ScrollDown => {/*self.no_op_event();*/}
                    event::MouseEventKind::ScrollUp => {/*self.no_op_event();*/}
                }
            }
            event::Event::Resize(x, y) => self.resize(x, y),
            event::Event::FocusLost => {/*do nothing*/} //maybe quit displaying cursor(s)/selection(s)?...
            event::Event::FocusGained => {/*do nothing*/}   //display cursor(s)/selection(s)?...
            event::Event::Paste(_) => {/*self.no_op_event();*/}
        }

        Ok(())
    }

    pub fn mode(&self) -> Mode{
        self.mode_stack.last().unwrap().clone()
    }
    pub fn mode_pop(&mut self){
        //set any mode specific exit behavior
        let mut update_layouts_and_document = false;
        let mut clear_util_bar_text = false;
        let mut clear_saved_selections = false;
        match self.mode(){
            Mode::Command | Mode::Goto => {
                update_layouts_and_document = true;
                clear_util_bar_text = true;
            }
            Mode::Find | Mode::Split => {
                update_layouts_and_document = true;
                clear_util_bar_text = true;
                clear_saved_selections = true;
            }
            Mode::Object | Mode::Notify | Mode::View | Mode::Warning(_) | Mode::AddSurround => {}
            Mode::Insert => {self.mode_push(Mode::Warning(WarningKind::InvalidInput));}
        }

        //remove current mode from stack
        self.mode_stack.pop();

        //handle exit behavior
        if update_layouts_and_document{
            self.ui.update_layouts(&self.mode());
            self.view.set_size(
                self.ui.document_viewport.document_widget.rect.width as usize,
                self.ui.document_viewport.document_widget.rect.height as usize
            );
            self.update_ui_data_document();
        }
        if clear_util_bar_text{
            self.ui.util_bar.utility_widget.text_box.clear();
        }
        if clear_saved_selections{
            self.ui.util_bar.utility_widget.preserved_selections = None;
        }
    }
    pub fn mode_push(&mut self, to_mode: Mode){
        if self.mode() == to_mode{/*do nothing*/}   //don't push mode to stack because we are already there
        else{
            //set any mode specific entry behavior
            let mut save_selections = false;
            let mut update_layouts_and_document = false;
            match to_mode{
                Mode::Find | Mode::Split => {
                    save_selections = true;
                    if !self.ui.status_bar.display{ // potential fix for status bar bug in todo.rs
                        update_layouts_and_document = true;
                    }
                }
                Mode::Command | Mode::Goto => {
                    if !self.ui.status_bar.display{ // potential fix for status bar bug in todo.rs
                        update_layouts_and_document = true;
                    }
                }
                Mode::Object | Mode::Insert | Mode::Notify | Mode::View | Mode::Warning(_) | Mode::AddSurround => {/* do nothing */}
            }

            //add mode to top of stack
            self.mode_stack.push(to_mode);

            //handle entry behavior
            if save_selections{
                self.ui.util_bar.utility_widget.preserved_selections = Some(self.selections.clone());
            }
            if update_layouts_and_document{
                self.ui.update_layouts(&self.mode());
                self.view.set_size(
                    self.ui.document_viewport.document_widget.rect.width as usize,
                    self.ui.document_viewport.document_widget.rect.height as usize
                );
                self.update_ui_data_document();
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    //pub fn insert(&mut self, new_text: &str){
    //    //create pending changeset
    //    //for each selection
    //        //insert new_text at/replacing selection (depends on selection extension)
    //        //handle hook behavior
    //            //if new_text multichar
    //                //extend selection to encompass new_text (extension direction could be input language dependent(like arabic could be backwards))
    //            //else if new_text single char
    //                //move cursor (movement direction could be input language dependent(like arabic could be backwards))
    //            //update subsequent selection positions to reflect new changes
    //            //add change to pending changeset (figure out how to group related subsequent changes(like typing each char in a word) in to one single changeset)
    //}
    //pub fn remove(&mut self){
    //
    //}
    //pub fn replace(&mut self, new_text: &str){
    //
    //}
    ////////////////////////////////////////////////////////////////////////////

    /// Set all data related to document viewport UI.
    pub fn update_ui_data_document(&mut self){
        let text = &self.buffer;
        
        self.ui.document_viewport.document_widget.text_in_view = self.view.text(text);
        self.ui.document_viewport.line_number_widget.line_numbers_in_view = self.view.line_numbers(text);
        self.update_ui_data_selections();
        self.ui.status_bar.modified_indicator_widget.document_modified_status = self.buffer.is_modified();
    }
    /// Set only data related to selections in document viewport UI.
    pub fn update_ui_data_selections(&mut self){
        let text = &self.buffer;
        let selections = &self.selections;
        
        self.ui.document_viewport.highlighter.set_primary_cursor_position(self.view.primary_cursor_position(text, selections, CURSOR_SEMANTICS));
        self.ui.document_viewport.highlighter.set_client_cursor_positions(self.view.cursor_positions(text, selections, CURSOR_SEMANTICS));
        self.ui.document_viewport.highlighter.selections = self.view.selections(selections, text);
        self.ui.status_bar.selections_widget.primary_selection_index = selections.primary_selection_index;
        self.ui.status_bar.selections_widget.num_selections = selections.count();
        self.ui.status_bar.document_cursor_position_widget.document_cursor_position = selections.primary().selection_to_selection2d(text, CURSOR_SEMANTICS).head().clone();
    }
    // prefer this over checked_scroll_and_update only when the view should ALWAYS scroll.      //TODO: comment out this fn, and verify all callers actually need this fn and not checked_scroll_and_update
    //pub fn scroll_and_update(&mut self, selection: &Selection){ //TODO: maybe scrolling should be separate from scrolling?...
    //    let text = self.document.text().clone();
    //    *self.document.view_mut() = self.document.view().scroll_following_cursor(selection, &text, CURSOR_SEMANTICS);
    //    self.update_ui_data_document();
    //}
    //TODO: should edit_core handle updating the view, then return view information?
    // prefer this over scroll_and_update, even when response fns are the same, because it saves us from unnecessarily reassigning the view
    pub fn checked_scroll_and_update<F, A>(&mut self, cursor_to_follow: &crate::selection::Selection, scroll_response_fn: F, non_scroll_response_fn: A)
        where F: Fn(&mut Application), A: Fn(&mut Application)
    {
        let text = &self.buffer;
        if self.view.should_scroll(cursor_to_follow, &text, CURSOR_SEMANTICS){
            self.view = self.view.scroll_following_cursor(cursor_to_follow, &text, CURSOR_SEMANTICS);
            scroll_response_fn(self);
        }else{
            non_scroll_response_fn(self);
        }
    }
    pub fn update_ui_data_util_bar(&mut self){
        let buffer = &self.ui.util_bar.utility_widget.text_box.buffer;
        let selections = crate::selections::Selections::new(
            vec![self.ui.util_bar.utility_widget.text_box.selection.clone()], 
            0, 
            buffer,
            CURSOR_SEMANTICS
        );
        self.ui.util_bar.utility_widget.text_box.view = self.ui.util_bar.utility_widget.text_box.view.scroll_following_cursor(
            selections.primary(), 
            buffer,
            CURSOR_SEMANTICS
        );
    }


    pub fn no_op_keypress(&mut self){
        self.mode_push(Mode::Warning(WarningKind::UnhandledKeypress));
    }
    pub fn no_op_event(&mut self){
        self.mode_push(Mode::Warning(WarningKind::UnhandledEvent));
    }
    pub fn resize(&mut self, x: u16, y: u16){
        self.ui.set_terminal_size(x, y);
        // ui layouts need to be updated before doc size set, so doc size can be calculated correctly
        self.ui.update_layouts(&self.mode());
        self.update_ui_data_util_bar(); //TODO: can this be called later in fn impl?
        self.view.set_size(
            self.ui.document_viewport.document_widget.rect.width as usize, 
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        // scrolling so cursor is in a reasonable place, and updating so any ui changes render correctly
        self.checked_scroll_and_update(
            &self.selections.primary().clone(),
            Application::update_ui_data_document, 
            Application::update_ui_data_document
        );
    }

    pub fn esc_handle(&mut self){
        assert!(self.mode() == Mode::Insert);
        //TODO: if lsp suggestions displaying(currently unimplemented), exit that display   //lsp suggestions could be a separate mode with keybind fallthrough to insert...
        /*else */if self.selections.count() > 1{
            self.selection_action(&SelectionAction::ClearNonPrimarySelections);
        }
        else if self.selections.primary().is_extended(){
            self.selection_action(&SelectionAction::CollapseSelections);
        }
        else{
            if SHOW_SAME_STATE_WARNINGS{self.mode_push(Mode::Warning(WarningKind::SameState));}
        }
    }

    pub fn quit(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        if self.buffer.is_modified(){
            self.mode_push(Mode::Warning(WarningKind::FileIsModified));
        }
        else{self.should_quit = true;}
    }
    pub fn quit_ignoring_changes(&mut self){
        assert!(self.mode() == Mode::Warning(WarningKind::FileIsModified) || self.mode() == Mode::Command);
        self.should_quit = true;
    }

    pub fn save(&mut self){
        assert!(self.mode() == Mode::Insert);
        match crate::utilities::save::application_impl(self){
            Ok(()) => {self.update_ui_data_document();}
            Err(_) => {self.mode_push(Mode::Warning(WarningKind::FileSaveFailed));}
        }
    }
    fn handle_application_error(&mut self, e: ApplicationError){
        let this_file = std::panic::Location::caller().file();  //actually, these should prob be assigned in calling fn, and passed in, so that error location is the caller and not always here...
        let line_number = std::panic::Location::caller().line();
        match e{
            ApplicationError::ReadOnlyBuffer => {self.mode_push(Mode::Warning(WarningKind::UnhandledError("buffer is read only".to_string())));}
            ApplicationError::InvalidInput => {self.mode_push(Mode::Warning(WarningKind::InvalidInput));}
            ApplicationError::SelectionAtDocBounds |
            ApplicationError::NoChangesToUndo |
            ApplicationError::NoChangesToRedo => {if SHOW_SAME_STATE_WARNINGS{self.mode_push(Mode::Warning(WarningKind::SameState));}}
            ApplicationError::SelectionsError(s) => {
                match s{
                    crate::selections::SelectionsError::ResultsInSameState |
                    crate::selections::SelectionsError::CannotAddSelectionAbove |
                    crate::selections::SelectionsError::CannotAddSelectionBelow => {if SHOW_SAME_STATE_WARNINGS{self.mode_push(Mode::Warning(WarningKind::SameState));}}
                    crate::selections::SelectionsError::MultipleSelections => {self.mode_push(Mode::Warning(WarningKind::MultipleSelections));}
                    crate::selections::SelectionsError::SingleSelection => {self.mode_push(Mode::Warning(WarningKind::SingleSelection));}
                    crate::selections::SelectionsError::NoSearchMatches |
                    //TODO: this error can now happen. figure out how to handle it...
                    crate::selections::SelectionsError::SpansMultipleLines => self.mode_push(Mode::Warning(WarningKind::UnhandledError(format!("{s:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))),
                }
            }
        }
    }
    pub fn copy(&mut self){
        assert!(self.mode() == Mode::Insert);
        match crate::utilities::copy::application_impl(self){
            Ok(()) => {
                self.mode_push(Mode::Notify);
                self.update_ui_data_document(); //TODO: is this really needed for something?...
            }
            Err(e) => {
                self.handle_application_error(e);
            }
        }
    }
    pub fn edit_action(&mut self, action: &EditAction){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::AddSurround);
        let len = self.buffer.len_lines();
        use crate::utilities::{insert_string, delete, backspace, cut, paste, undo, redo, add_surrounding_pair};
        let result = match action{
            EditAction::InsertChar(c) => insert_string::application_impl(self, &c.to_string(), USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::InsertNewline => insert_string::application_impl(self, "\n", USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::InsertTab => insert_string::application_impl(self, "\t", USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::Delete => delete::application_impl(self, CURSOR_SEMANTICS),
            EditAction::Backspace => backspace::application_impl(self, USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::Cut => cut::application_impl(self, CURSOR_SEMANTICS),
            EditAction::Paste => paste::application_impl(self, USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::Undo => undo::application_impl(self, CURSOR_SEMANTICS),   // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
            EditAction::Redo => redo::application_impl(self, CURSOR_SEMANTICS),
            EditAction::AddSurround(l, t) => add_surrounding_pair::application_impl(self, *l, *t, CURSOR_SEMANTICS),
        };
        if self.mode() != Mode::Insert{self.mode_pop();}
        match result{
            Ok(()) => {
                self.checked_scroll_and_update(
                    &self.selections.primary().clone(), 
                    Application::update_ui_data_document, 
                    Application::update_ui_data_document
                );
                if len != self.buffer.len_lines(){self.ui.document_viewport.document_widget.doc_length = self.buffer.len_lines();}
            }
            Err(e) => {
                self.handle_application_error(e);
            }
        }
    }

    pub fn selection_action(&mut self, action: &SelectionAction){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Object);
        let result = match action{
            SelectionAction::MoveCursorUp => {crate::utilities::move_cursor_up::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorDown => {crate::utilities::move_cursor_down::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorLeft => {crate::utilities::move_cursor_left::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorRight => {crate::utilities::move_cursor_right::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorWordBoundaryForward => {crate::utilities::move_cursor_word_boundary_forward::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorWordBoundaryBackward => {crate::utilities::move_cursor_word_boundary_backward::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorLineEnd => {crate::utilities::move_cursor_line_end::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorHome => {crate::utilities::move_cursor_home::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorBufferStart => {crate::utilities::move_cursor_buffer_start::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorBufferEnd => {crate::utilities::move_cursor_buffer_end::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorPageUp => {crate::utilities::move_cursor_page_up::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorPageDown => {crate::utilities::move_cursor_page_down::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionUp => {crate::utilities::extend_selection_up::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionDown => {crate::utilities::extend_selection_down::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionLeft => {crate::utilities::extend_selection_left::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionRight => {crate::utilities::extend_selection_right::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionWordBoundaryBackward => {crate::utilities::extend_selection_word_boundary_backward::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionWordBoundaryForward => {crate::utilities::extend_selection_word_boundary_forward::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionLineEnd => {crate::utilities::extend_selection_line_end::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionHome => {crate::utilities::extend_selection_home::application_impl(self, CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionDocumentStart => {self.document.extend_selection_document_start(CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionDocumentEnd => {self.document.extend_selection_document_end(CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionPageUp => {self.document.extend_selection_page_up(CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionPageDown => {self.document.extend_selection_page_down(CURSOR_SEMANTICS)}
            SelectionAction::SelectLine => {crate::utilities::select_line::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::SelectAll => {crate::utilities::select_all::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::CollapseSelections => {crate::utilities::collapse_selections_to_cursor::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::ClearNonPrimarySelections => {crate::utilities::clear_non_primary_selections::application_impl(self)}
            SelectionAction::AddSelectionAbove => {crate::utilities::add_selection_above::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::AddSelectionBelow => {crate::utilities::add_selection_below::application_impl(self, CURSOR_SEMANTICS)}
            SelectionAction::RemovePrimarySelection => {crate::utilities::remove_primary_selection::application_impl(self)}
            SelectionAction::IncrementPrimarySelection => {crate::utilities::increment_primary_selection::application_impl(self)}
            SelectionAction::DecrementPrimarySelection => {crate::utilities::decrement_primary_selection::application_impl(self)}
            SelectionAction::Surround => {crate::utilities::surround::application_impl(self, CURSOR_SEMANTICS)}
        
            //These may technically be distinct from the other selection actions, because they could be called from object mode, and would need to pop the mode stack after calling...
            //TODO: SelectionAction::Word => {self.document.word()}
            //TODO: SelectionAction::Sentence => {self.document.sentence()}
            //TODO: SelectionAction::Paragraph => {self.document.paragraph()}
            SelectionAction::SurroundingPair => {crate::utilities::nearest_surrounding_pair::application_impl(self, CURSOR_SEMANTICS)}  //TODO: rename SurroundingBracketPair
            //TODO: SelectionAction::QuotePair => {self.document.nearest_quote_pair()}                      //TODO: rename SurroundingQuotePair
            //TODO: SelectionAction::ExclusiveSurroundingPair => {self.document.exclusive_surrounding_pair()}
            //TODO: SelectionAction::InclusiveSurroundingPair => {self.document.inclusive_surrounding_pair()}
        
            SelectionAction::FlipDirection => {crate::utilities::flip_direction::application_impl(self, CURSOR_SEMANTICS)}
        };

        //maybe.    so far, only needed for selection actions called from object mode
        if self.mode() != Mode::Insert{
            self.mode_pop();
        }
        //

        match result{
            Ok(()) => {
                self.checked_scroll_and_update(&self.selections.primary().clone(), 
                    Application::update_ui_data_document, 
                    Application::update_ui_data_selections
                );
            }
            Err(e) => {
                self.handle_application_error(e);
            }
        }
    }

    pub fn view_action(&mut self, action: &ViewAction){      //TODO: make sure this can still be called from insert, so users can assign a direct keybind if desired
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::View);

        let mut should_exit = false;
        let result = match action{
            ViewAction::CenterVerticallyAroundCursor => {
                should_exit = true;
                crate::utilities::center_view_vertically_around_cursor::application_impl(self, CURSOR_SEMANTICS)
            }
            ViewAction::ScrollUp => crate::utilities::scroll_view_up::application_impl(self, VIEW_SCROLL_AMOUNT),
            ViewAction::ScrollDown => crate::utilities::scroll_view_down::application_impl(self, VIEW_SCROLL_AMOUNT),
            ViewAction::ScrollLeft => crate::utilities::scroll_view_left::application_impl(self, VIEW_SCROLL_AMOUNT),
            ViewAction::ScrollRight => crate::utilities::scroll_view_right::application_impl(self, VIEW_SCROLL_AMOUNT)
        };
        match result{
            Ok(()) => {
                self.update_ui_data_document();
                if self.mode() != Mode::Insert && should_exit{self.mode_pop();}
            }
            Err(e) => {
                self.handle_application_error(e)
            }
        }
    }

    //TODO: util_action maybe should be a sub-action of EditorAction(which itself still needs to be implemented...)
    //TODO: split into util_edit_action and util_selection_action?...
    pub fn util_action(&mut self, action: &UtilAction){
        let text_box = &mut self.ui.util_bar.utility_widget.text_box;
        match action{
            UtilAction::Backspace => text_box.backspace(),
            UtilAction::Delete => text_box.delete(),
            UtilAction::InsertChar(c) => text_box.insert_char(*c),
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

        //perform any mode specific follow up actions   //shouldn't need to call these if action was a selection action instead of an edit action
        match self.mode(){
            Mode::Object |
            Mode::Insert |
            Mode::View |
            Mode::Notify |
            Mode::Warning(_) |
            Mode::AddSurround => {/*do nothing*/}
            Mode::Goto => {
                self.goto_mode_text_validity_check();
            }
            Mode::Find => {
                self.incremental_search();
                self.checked_scroll_and_update(
                    &self.selections.primary().clone(), 
                    Application::update_ui_data_document, 
                    Application::update_ui_data_selections
                );
            }
            Mode::Split => {
                self.incremental_split();
                self.checked_scroll_and_update(
                    &self.selections.primary().clone(), 
                    Application::update_ui_data_document, 
                    Application::update_ui_data_selections
                );
            }
            Mode::Command => {/*do nothing*/}
        }
    }

    pub fn goto_mode_accept(&mut self){
        assert!(self.mode() == Mode::Goto);
        let mut show_warning = false;
        if let Ok(line_number) = self.ui.util_bar.utility_widget.text_box.buffer.inner.to_string().parse::<usize>(){
            if line_number == 0{show_warning = true;}   //we have no line number 0, so this is invalid
            else{
                let line_number = line_number.saturating_sub(1);    // make line number 0 based for interfacing correctly with backend impl
                match crate::utilities::move_to_line_number::application_impl(self, line_number, CURSOR_SEMANTICS){
                    Ok(()) => {self.checked_scroll_and_update(&self.selections.primary().clone(), Application::update_ui_data_selections, Application::update_ui_data_selections);}  //TODO: pretty sure one of these should be update_ui_data_document
                    Err(_) => {show_warning = true;}    //TODO: match error and handle
                }
            }
        }else{show_warning = true;}
        if show_warning{self.mode_push(Mode::Warning(WarningKind::InvalidInput));}
        else{self.mode_pop()}
    }
    //TODO: add go to matching surrounding char(curly, square, paren, single quote, double quote, etc)
    //TODO: this could prob use move_vertically/move_horizontally from edit_core...
    //TODO: can this be accomplished in edit_core instead?...
    // Not entirely sure I want this behavior...
    pub fn goto_mode_selection_action(&mut self, action: &SelectionAction){  //TODO: this is pretty slow when user enters a large number into util text box
        assert!(self.mode() == Mode::Goto);
        if let Ok(amount) = self.ui.util_bar.utility_widget.text_box.buffer.inner.to_string().parse::<usize>(){
            self.mode_pop();
            for _ in 0..amount{
                if matches!(self.mode(), Mode::Warning(_)){break;}    //trying to speed this up by preventing this from running `amount` times, if there has already been an error
                self.selection_action(action);  //TODO: if this reaches doc boundaries, this will display same state warning. which it technically may not be the same state as when this fn was called...
            }
        }else{
            self.mode_push(Mode::Warning(WarningKind::InvalidInput));
        }
        //also, this doesn't work with goto_mode_text_validity_check
    }
    pub fn goto_mode_text_validity_check(&mut self){
        assert!(self.mode() == Mode::Goto);
        // run text validity check
        let mut is_numeric = true;
        for grapheme in self.ui.util_bar.utility_widget.text_box.buffer.inner.chars(){ // .graphemes(true)?
            if !grapheme.is_ascii_digit(){is_numeric = false;}
        }
        let exceeds_doc_length = match self.ui.util_bar.utility_widget.text_box.buffer.inner.to_string().parse::<usize>(){
            Ok(line_number) => {line_number > self.buffer.len_lines()}  //line_number > self.ui.document_length()
            Err(_) => false
        };
        self.ui.util_bar.utility_widget.text_box.text_is_valid = is_numeric && !exceeds_doc_length;
    }

    pub fn restore_selections_and_exit(&mut self){
        self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        self.selections = self.ui.util_bar.utility_widget.preserved_selections.clone().unwrap();    //shouldn't be called unless this value is Some()
        self.checked_scroll_and_update(&self.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
        self.mode_pop();
    }
    fn incremental_search(&mut self){   //this def doesn't work correctly with utf-8 yet
        let preserved_selections = self.ui.util_bar.utility_widget.preserved_selections.clone();
        let search_text = self.ui.util_bar.utility_widget.text_box.buffer.inner.to_string().clone();
        match preserved_selections{
            Some(preserved_selections) => {
                match crate::utilities::incremental_search_in_selection::application_impl(self, &search_text, &preserved_selections, CURSOR_SEMANTICS){
                    Ok(()) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = true;}
                    Err(_) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = false;}
                }
            }
            None => {/* maybe error?... */unreachable!()}
        }
    }
    fn incremental_split(&mut self){
        let preserved_selections = self.ui.util_bar.utility_widget.preserved_selections.clone();
        let split_text = self.ui.util_bar.utility_widget.text_box.buffer.inner.to_string().clone();
        match preserved_selections{
            Some(preserved_selections) => {
                match crate::utilities::incremental_split_in_selection::application_impl(self, &split_text, &preserved_selections, CURSOR_SEMANTICS){
                    Ok(()) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = true;}
                    Err(_) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = false;}
                }
            }
            None => {/* maybe error?... */unreachable!()}
        }
    }

    pub fn toggle_line_numbers(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        self.ui.document_viewport.toggle_line_numbers();
                
        self.ui.update_layouts(&self.mode());
        self.view.set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
    }
    pub fn toggle_status_bar(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        self.ui.status_bar.toggle_status_bar();
                
        self.ui.update_layouts(&self.mode());
        self.view.set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
    }
    pub fn open_new_terminal_window(){
        let _ = std::process::Command::new("alacritty")     //TODO: have user define TERMINAL const in config.rs   //or check env vars for $TERM?
            //.arg("msg")     // these extra commands just make new instances use the same backend(daemon?)
            //.arg("create-window")
            //.current_dir(std::env::current_dir().unwrap())    //not needed here, because term spawned here defaults to this directory, but good to know
            .spawn()
            .expect("failed to spawn new terminal at current directory");
    }
    //should command parsing be implemented in edit_core?...
    //TODO: command mode should have completion suggestions
    pub fn command_mode_accept(&mut self){
        assert!(self.mode() == Mode::Command);
        let mut warn = false;
        match self.ui.util_bar.utility_widget.text_box.buffer.inner.to_string().as_str(){
            "term" | "t" => {Application::open_new_terminal_window();}
            "toggle_line_numbers" | "ln" => {self.toggle_line_numbers();}
            "toggle_status_bar" | "sb" => {self.toggle_status_bar();}
            "quit" | "q" => {
                self.quit();
                return; //needed so app can quit without running the rest of the code in this fn
            }
            "quit!" | "q!" => {
                self.quit_ignoring_changes();
                return;
            }
            //write | w         //write buffer contents to file //should this optionally take a filepath to save to? then we don't need to implement save as    //would have to split util bar text on ' ' into separate args
            _ => {warn = true;}
        }
        if warn{self.mode_push(Mode::Warning(WarningKind::CommandParseFailed));}
        else{self.mode_pop()}
    }


    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_replace(
        buffer: &mut crate::buffer::Buffer, 
        replacement_text: &str, 
        selection: &mut crate::selection::Selection, 
        semantics: crate::selection::CursorSemantics
    ) -> crate::history::Change{ //TODO: Error if replacement_text is empty(or if selection empty? is this possible?)
        let old_selection = selection.clone();
        let delete_change = Application::apply_delete(buffer, selection, semantics.clone());
        let replaced_text = if let crate::history::Operation::Insert{inserted_text} = delete_change.inverse(){inserted_text}else{unreachable!();};  // inverse of delete change should always be insert
        let _ = Application::apply_insert(buffer, replacement_text, selection, semantics.clone());   //intentionally discard returned Change

        crate::history::Change::new(
            crate::history::Operation::Replace{replacement_text: replacement_text.to_string()}, 
            old_selection, 
            selection.clone(), 
            crate::history::Operation::Replace{replacement_text: replaced_text}
        )
    }
    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_insert(
        buffer: &mut crate::buffer::Buffer, 
        string: &str, 
        selection: &mut crate::selection::Selection, 
        semantics: crate::selection::CursorSemantics
    ) -> crate::history::Change{    //TODO: Error if string is empty
        let old_selection = selection.clone();
        buffer.insert(selection.cursor(buffer, semantics.clone()), string);
        for _ in 0..string.len(){
            if let Ok(new_selection) = crate::utilities::move_cursor_right::selection_impl(selection, buffer, semantics.clone()){
                *selection = new_selection;
            }
        }

        crate::history::Change::new(
            crate::history::Operation::Insert{inserted_text: string.to_string()}, 
            old_selection, 
            selection.clone(), 
            crate::history::Operation::Delete
        )
    }
    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_delete(
        buffer: &mut crate::buffer::Buffer, 
        selection: &mut crate::selection::Selection, 
        semantics: crate::selection::CursorSemantics
    ) -> crate::history::Change{  //TODO: Error if cursor and anchor at end of text
        use std::cmp::Ordering;
        
        let old_selection = selection.clone();
        let original_text = buffer.clone();

        let (start, end, new_cursor) = match selection.cursor(buffer, semantics.clone()).cmp(&selection.anchor()){
            Ordering::Less => {(selection.head(), selection.anchor(), selection.cursor(buffer, semantics.clone()))}
            Ordering::Greater => {
                match semantics{
                    crate::selection::CursorSemantics::Bar => {(selection.anchor(), selection.head(), selection.anchor())}
                    crate::selection::CursorSemantics::Block => {
                        if selection.cursor(&buffer, semantics.clone()) == buffer.len_chars(){(selection.anchor(), selection.cursor(buffer, semantics.clone()), selection.anchor())}
                        else{(selection.anchor(), selection.head(), selection.anchor())}
                    }
                }
            }
            Ordering::Equal => {
                if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){ //do nothing    //or preferrably return error   //could have condition check in calling fn
                    return crate::history::Change::new(
                        crate::history::Operation::Delete, 
                        old_selection, 
                        selection.clone(), 
                        crate::history::Operation::Insert{inserted_text: String::new()}
                    );   //change suggested by clippy lint
                }
                
                match semantics.clone(){
                    crate::selection::CursorSemantics::Bar => {(selection.head(), selection.head().saturating_add(1), selection.anchor())}
                    crate::selection::CursorSemantics::Block => {(selection.anchor(), selection.head(), selection.anchor())}
                }
            }
        };

        let change_text = original_text.slice(start, end);
        buffer.remove(start..end);
        if let Ok(new_selection) = selection.put_cursor(new_cursor, &original_text, crate::selection::Movement::Move, semantics, true){
            *selection = new_selection;
        }

        crate::history::Change::new(
            crate::history::Operation::Delete, 
            old_selection, 
            selection.clone(), 
            crate::history::Operation::Insert{inserted_text: change_text.to_string()}
        )
    }
}

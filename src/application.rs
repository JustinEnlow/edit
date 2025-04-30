use std::error::Error;
use std::path::PathBuf;
use crossterm::event;
use ratatui::layout::Rect;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::ui::UserInterface;
use edit_core::selection::Selection;
use edit_core::selections::{Selections, SelectionsError};
use edit_core::document::{Document, DocumentError};
use crate::keybind;
use crate::config::{CURSOR_SEMANTICS, SHOW_SAME_STATE_WARNINGS, VIEW_SCROLL_AMOUNT, USE_HARD_TAB, TAB_WIDTH, USE_FULL_FILE_PATH};



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
    MoveCursorDocumentStart,
    MoveCursorDocumentEnd,
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
    //ExtendSelectionDocumentStart,
    //ExtendSelectionDocumentEnd,
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
    document: Document,
    ui: UserInterface,
}
impl Application{
    pub fn new(file_path: &str, terminal: &Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<Self, Box<dyn Error>>{
        let terminal_size = terminal.size()?;
        let terminal_rect = Rect::new(0, 0, terminal_size.width, terminal_size.height);

        let mut instance = Self{
            should_quit: false,
            mode_stack: vec![Mode::Insert],
            document: Document::new(CURSOR_SEMANTICS),
            ui: UserInterface::new(terminal_rect)
        };

        let path = PathBuf::from(file_path).canonicalize()?;

        instance.document = Document::open(&path, CURSOR_SEMANTICS)?;
        instance.ui.status_bar.file_name_widget.file_name = instance.document.file_name(USE_FULL_FILE_PATH);
        instance.ui.document_viewport.document_widget.doc_length = instance.document.len();
        
        instance.ui.update_layouts(&instance.mode());
        //init backend doc view size
        instance.document.client_view.set_size(
            instance.ui.document_viewport.document_widget.rect.width as usize,
            instance.ui.document_viewport.document_widget.rect.height as usize
        );

        // prefer this over scroll_and_update, even when response fns are the same, because it saves us from unnecessarily reassigning the view
        instance.checked_scroll_and_update(&instance.document.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_document);

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
                    event::MouseEventKind::Down(_) => {self.no_op_event();}
                    event::MouseEventKind::Up(_) => {self.no_op_event();}
                    event::MouseEventKind::Drag(_) => {self.no_op_event();}
                    event::MouseEventKind::Moved => {self.no_op_event();}
                    event::MouseEventKind::ScrollDown => {self.no_op_event();}
                    event::MouseEventKind::ScrollUp => {self.no_op_event();}
                }
            }
            event::Event::Resize(x, y) => self.resize(x, y),
            event::Event::FocusLost => {/*do nothing*/} //maybe quit displaying cursor(s)/selection(s)?...
            event::Event::FocusGained => {/*do nothing*/}   //display cursor(s)/selection(s)?...
            event::Event::Paste(_) => {self.no_op_event();}
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
            self.document.client_view.set_size(
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
                self.ui.util_bar.utility_widget.preserved_selections = Some(self.document.selections.clone());
            }
            if update_layouts_and_document{
                self.ui.update_layouts(&self.mode());
                self.document.client_view.set_size(
                    self.ui.document_viewport.document_widget.rect.width as usize,
                    self.ui.document_viewport.document_widget.rect.height as usize
                );
                self.update_ui_data_document();
            }
        }
    }

    /// Set all data related to document viewport UI.
    pub fn update_ui_data_document(&mut self){
        let text = &self.document.text;
        
        self.ui.document_viewport.document_widget.text_in_view = self.document.client_view.text(text);
        self.ui.document_viewport.line_number_widget.line_numbers_in_view = self.document.client_view.line_numbers(text);
        self.update_ui_data_selections();
        self.ui.status_bar.modified_indicator_widget.document_modified_status = self.document.is_modified();
    }
    /// Set only data related to selections in document viewport UI.
    pub fn update_ui_data_selections(&mut self){
        let text = &self.document.text;
        let selections = &self.document.selections;
        
        self.ui.document_viewport.highlighter.set_primary_cursor_position(self.document.client_view.primary_cursor_position(text, selections, CURSOR_SEMANTICS));
        self.ui.document_viewport.highlighter.set_client_cursor_positions(self.document.client_view.cursor_positions(text, selections, CURSOR_SEMANTICS));
        self.ui.document_viewport.highlighter.selections = self.document.client_view.selections(selections, text);
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
    pub fn checked_scroll_and_update<F, A>(&mut self, cursor_to_follow: &Selection, scroll_response_fn: F, non_scroll_response_fn: A)
        where F: Fn(&mut Application), A: Fn(&mut Application)
    {
        let text = self.document.text.clone();
        if self.document.client_view.should_scroll(cursor_to_follow, &text, CURSOR_SEMANTICS){
            self.document.client_view = self.document.client_view.scroll_following_cursor(cursor_to_follow, &text, CURSOR_SEMANTICS);
            scroll_response_fn(self);
        }else{
            non_scroll_response_fn(self);
        }
    }
    pub fn update_ui_data_util_bar(&mut self){
        let text = self.ui.util_bar.utility_widget.text_box.text.clone();
        let selections = Selections::new(vec![self.ui.util_bar.utility_widget.text_box.selection.clone()], 0, &text, CURSOR_SEMANTICS);
        self.ui.util_bar.utility_widget.text_box.view = self.ui.util_bar.utility_widget.text_box.view.scroll_following_cursor(selections.primary(), &text, CURSOR_SEMANTICS);
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
        self.document.client_view.set_size(self.ui.document_viewport.document_widget.rect.width as usize, self.ui.document_viewport.document_widget.rect.height as usize);
        // scrolling so cursor is in a reasonable place, and updating so any ui changes render correctly
        self.checked_scroll_and_update(&self.document.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_document);
    }

    pub fn esc_handle(&mut self){
        assert!(self.mode() == Mode::Insert);
        //TODO: if lsp suggestions displaying(currently unimplemented), exit that display   //lsp suggestions could be a separate mode with keybind fallthrough to insert...
        /*else */if self.document.selections.count() > 1{
            //self.clear_non_primary_selections();
            self.selection_action(&SelectionAction::ClearNonPrimarySelections);
        }
        else if self.document.selections.primary().is_extended(CURSOR_SEMANTICS){
            self.selection_action(&SelectionAction::CollapseSelections);
        }
        else{
            if SHOW_SAME_STATE_WARNINGS{self.mode_push(Mode::Warning(WarningKind::SameState));}
        }
    }

    pub fn quit(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        //if self.ui.document_modified(){   //this is the old impl when editor was set up for client/server and state needed to be stored in ui
        if self.document.is_modified(){
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
        match edit_core::utilities::save::document_impl(&mut self.document){
            Ok(()) => {self.update_ui_data_document();}
            Err(_) => {self.mode_push(Mode::Warning(WarningKind::FileSaveFailed));}
        }
    }
    fn handle_document_error(&mut self, e: DocumentError){
        let this_file = std::panic::Location::caller().file();  //actually, these should prob be assigned in calling fn, and passed in, so that error location is the caller and not always here...
        let line_number = std::panic::Location::caller().line();
        match e{
            DocumentError::InvalidInput => {self.mode_push(Mode::Warning(WarningKind::InvalidInput));}
            DocumentError::SelectionAtDocBounds |
            DocumentError::NoChangesToUndo |
            DocumentError::NoChangesToRedo => {if SHOW_SAME_STATE_WARNINGS{self.mode_push(Mode::Warning(WarningKind::SameState));}}
            DocumentError::SelectionsError(s) => {
                match s{
                    SelectionsError::ResultsInSameState |
                    SelectionsError::CannotAddSelectionAbove |
                    SelectionsError::CannotAddSelectionBelow => {if SHOW_SAME_STATE_WARNINGS{self.mode_push(Mode::Warning(WarningKind::SameState));}}
                    SelectionsError::MultipleSelections => {self.mode_push(Mode::Warning(WarningKind::MultipleSelections));}
                    SelectionsError::SingleSelection => {self.mode_push(Mode::Warning(WarningKind::SingleSelection));}
                    SelectionsError::NoSearchMatches |
                    //TODO: this error can now happen. figure out how to handle it...
                    SelectionsError::SpansMultipleLines => self.mode_push(Mode::Warning(WarningKind::UnhandledError(format!("{s:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))),
                }
            }
        }
    }
    pub fn copy(&mut self){
        assert!(self.mode() == Mode::Insert);
        match edit_core::utilities::copy::document_impl(&mut self.document){
            Ok(()) => {
                self.mode_push(Mode::Notify);
                self.update_ui_data_document(); //TODO: is this really needed for something?...
            }
            Err(e) => {self.handle_document_error(e);}
        }
    }
    pub fn edit_action(&mut self, action: &EditAction){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::AddSurround);
        let len = self.document.len();
        use edit_core::utilities::{insert_string, delete, backspace, cut, paste, undo, redo, add_surrounding_pair};
        let result = match action{
            EditAction::InsertChar(c) => insert_string::document_impl(&mut self.document, &c.to_string(), USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::InsertNewline => insert_string::document_impl(&mut self.document, "\n", USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::InsertTab => insert_string::document_impl(&mut self.document, "\t", USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::Delete => delete::document_impl(&mut self.document, CURSOR_SEMANTICS),
            EditAction::Backspace => backspace::document_impl(&mut self.document, USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::Cut => cut::document_impl(&mut self.document, CURSOR_SEMANTICS),
            EditAction::Paste => paste::document_impl(&mut self.document, USE_HARD_TAB, TAB_WIDTH, CURSOR_SEMANTICS),
            EditAction::Undo => undo::document_impl(&mut self.document, CURSOR_SEMANTICS),   // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
            EditAction::Redo => redo::document_impl(&mut self.document, CURSOR_SEMANTICS),
            EditAction::AddSurround(l, t) => add_surrounding_pair::document_impl(&mut self.document, *l, *t, CURSOR_SEMANTICS),
        };
        if self.mode() != Mode::Insert{self.mode_pop();}
        match result{
            Ok(()) => {
                self.checked_scroll_and_update(&self.document.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_document);
                if len != self.document.len(){self.ui.document_viewport.document_widget.doc_length = self.document.len();}
            }
            Err(e) => {self.handle_document_error(e);}
        }
    }

    pub fn selection_action(&mut self, action: &SelectionAction){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Object);
        let result = match action{
            SelectionAction::MoveCursorUp => {edit_core::utilities::move_cursor_up::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorDown => {edit_core::utilities::move_cursor_down::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorLeft => {edit_core::utilities::move_cursor_left::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorRight => {edit_core::utilities::move_cursor_right::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorWordBoundaryForward => {edit_core::utilities::move_cursor_word_boundary_forward::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorWordBoundaryBackward => {edit_core::utilities::move_cursor_word_boundary_backward::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorLineEnd => {edit_core::utilities::move_cursor_line_end::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorHome => {edit_core::utilities::move_cursor_home::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorDocumentStart => {edit_core::utilities::move_cursor_document_start::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorDocumentEnd => {edit_core::utilities::move_cursor_document_end::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorPageUp => {edit_core::utilities::move_cursor_page_up::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorPageDown => {edit_core::utilities::move_cursor_page_down::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionUp => {edit_core::utilities::extend_selection_up::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionDown => {edit_core::utilities::extend_selection_down::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionLeft => {edit_core::utilities::extend_selection_left::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionRight => {edit_core::utilities::extend_selection_right::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionWordBoundaryBackward => {edit_core::utilities::extend_selection_word_boundary_backward::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionWordBoundaryForward => {edit_core::utilities::extend_selection_word_boundary_forward::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionLineEnd => {edit_core::utilities::extend_selection_line_end::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionHome => {edit_core::utilities::extend_selection_home::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionDocumentStart => {self.document.extend_selection_document_start(CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionDocumentEnd => {self.document.extend_selection_document_end(CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionPageUp => {self.document.extend_selection_page_up(CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionPageDown => {self.document.extend_selection_page_down(CURSOR_SEMANTICS)}
            SelectionAction::SelectLine => {edit_core::utilities::select_line::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::SelectAll => {edit_core::utilities::select_all::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::CollapseSelections => {edit_core::utilities::collapse_selections_to_cursor::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::ClearNonPrimarySelections => {edit_core::utilities::clear_non_primary_selections::document_impl(&mut self.document)}
            SelectionAction::AddSelectionAbove => {edit_core::utilities::add_selection_above::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::AddSelectionBelow => {edit_core::utilities::add_selection_below::document_impl(&mut self.document, CURSOR_SEMANTICS)}
            SelectionAction::RemovePrimarySelection => {edit_core::utilities::remove_primary_selection::document_impl(&mut self.document)}
            SelectionAction::IncrementPrimarySelection => {edit_core::utilities::increment_primary_selection::document_impl(&mut self.document)}
            SelectionAction::DecrementPrimarySelection => {edit_core::utilities::decrement_primary_selection::document_impl(&mut self.document)}
            SelectionAction::Surround => {edit_core::utilities::surround::document_impl(&mut self.document, CURSOR_SEMANTICS)}

            //These may technically be distinct from the other selection actions, because they could be called from object mode, and would need to pop the mode stack after calling...
            //TODO: SelectionAction::Word => {self.document.word()}
            //TODO: SelectionAction::Sentence => {self.document.sentence()}
            //TODO: SelectionAction::Paragraph => {self.document.paragraph()}
            SelectionAction::SurroundingPair => {edit_core::utilities::nearest_surrounding_pair::document_impl(&mut self.document, CURSOR_SEMANTICS)}  //TODO: rename SurroundingBracketPair
            //TODO: SelectionAction::QuotePair => {self.document.nearest_quote_pair()}                      //TODO: rename SurroundingQuotePair
            //TODO: SelectionAction::ExclusiveSurroundingPair => {self.document.exclusive_surrounding_pair()}
            //TODO: SelectionAction::InclusiveSurroundingPair => {self.document.inclusive_surrounding_pair()}

            SelectionAction::FlipDirection => {edit_core::utilities::flip_direction::document_impl(&mut self.document, CURSOR_SEMANTICS)}
        };

        //maybe.    so far, only needed for selection actions called from object mode
        if self.mode() != Mode::Insert{ //if self.mode() == Mode::Object
            self.mode_pop();
        }
        //

        match result{
            Ok(()) => {
                self.checked_scroll_and_update(&self.document.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Err(e) => {self.handle_document_error(e);}
        }
    }

    pub fn view_action(&mut self, action: &ViewAction){      //TODO: make sure this can still be called from insert, so users can assign a direct keybind if desired
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::View);

        let mut should_exit = false;
        let result = match action{
            ViewAction::CenterVerticallyAroundCursor => {
                should_exit = true;
                edit_core::utilities::center_view_vertically_around_cursor::document_impl(&mut self.document, CURSOR_SEMANTICS)
            }
            ViewAction::ScrollUp => edit_core::utilities::scroll_view_up::document_impl(&mut self.document, VIEW_SCROLL_AMOUNT),
            ViewAction::ScrollDown => edit_core::utilities::scroll_view_down::document_impl(&mut self.document, VIEW_SCROLL_AMOUNT),
            ViewAction::ScrollLeft => edit_core::utilities::scroll_view_left::document_impl(&mut self.document, VIEW_SCROLL_AMOUNT),
            ViewAction::ScrollRight => edit_core::utilities::scroll_view_right::document_impl(&mut self.document, VIEW_SCROLL_AMOUNT)
        };
        match result{
            Ok(()) => {
                self.update_ui_data_document();
                if self.mode() != Mode::Insert && should_exit{self.mode_pop();}
            }
            Err(e) => self.handle_document_error(e),
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
                self.checked_scroll_and_update(&self.document.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Mode::Split => {
                self.incremental_split();
                self.checked_scroll_and_update(&self.document.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Mode::Command => {/*do nothing*/}
        }
    }

    pub fn goto_mode_accept(&mut self){
        assert!(self.mode() == Mode::Goto);
        let mut show_warning = false;
        if let Ok(line_number) = self.ui.util_bar.utility_widget.text_box.text.to_string().parse::<usize>(){
            if line_number == 0{show_warning = true;}   //we have no line number 0, so this is invalid
            else{
                let line_number = line_number.saturating_sub(1);    // make line number 0 based for interfacing correctly with backend impl
                match edit_core::utilities::move_to_line_number::document_impl(&mut self.document, line_number, CURSOR_SEMANTICS){
                    Ok(()) => {self.checked_scroll_and_update(&self.document.selections.primary().clone(), Application::update_ui_data_selections, Application::update_ui_data_selections);}  //TODO: pretty sure one of these should be update_ui_data_document
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
        if let Ok(amount) = self.ui.util_bar.utility_widget.text_box.text.to_string().parse::<usize>(){
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
        for grapheme in self.ui.util_bar.utility_widget.text_box.text.chars(){ // .graphemes(true)?
            if !grapheme.is_ascii_digit(){is_numeric = false;}
        }
        let exceeds_doc_length = match self.ui.util_bar.utility_widget.text_box.text.to_string().parse::<usize>(){
            Ok(line_number) => {line_number > self.document.len()}  //line_number > self.ui.document_length()
            Err(_) => false
        };
        self.ui.util_bar.utility_widget.text_box.text_is_valid = is_numeric && !exceeds_doc_length;
    }

    pub fn restore_selections_and_exit(&mut self){
        self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        self.document.selections = self.ui.util_bar.utility_widget.preserved_selections.clone().unwrap();    //shouldn't be called unless this value is Some()
        self.checked_scroll_and_update(&self.document.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
        self.mode_pop();
    }
    fn incremental_search(&mut self){   //this def doesn't work correctly with utf-8 yet
        match &self.ui.util_bar.utility_widget.preserved_selections{
            Some(preserved_selections) => {
                match edit_core::utilities::incremental_search_in_selection::document_impl(&mut self.document, &self.ui.util_bar.utility_widget.text_box.text.to_string(), preserved_selections, CURSOR_SEMANTICS){
                    Ok(()) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = true;}
                    Err(_) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = false;}
                }
            }
            None => {/* maybe error?... */unreachable!()}
        }
    }
    fn incremental_split(&mut self){
        match &self.ui.util_bar.utility_widget.preserved_selections{
            Some(preserved_selections) => {
                match edit_core::utilities::incremental_split_in_selection::document_impl(&mut self.document, &self.ui.util_bar.utility_widget.text_box.text.to_string(), preserved_selections, CURSOR_SEMANTICS){
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
        self.document.client_view.set_size(
            self.ui.document_viewport.document_widget.rect.width as usize,
            self.ui.document_viewport.document_widget.rect.height as usize
        );
        self.update_ui_data_document();
    }
    pub fn toggle_status_bar(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        self.ui.status_bar.toggle_status_bar();
                
        self.ui.update_layouts(&self.mode());
        self.document.client_view.set_size(
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
        match self.ui.util_bar.utility_widget.text_box.text.to_string().as_str(){
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
}

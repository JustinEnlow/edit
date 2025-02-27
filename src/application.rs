use std::error::Error;
use std::path::PathBuf;
use crossterm::event;
use ratatui::layout::Rect;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::ui::UserInterface;
use edit_core::selection::Selection;
use edit_core::selections::{Selections, SelectionsError};
use edit_core::view::ViewError;
use edit_core::document::{Document, DocumentError};
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
    //RotateTextInSelections
}
#[derive(Clone)]
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
    ExtendSelectionPageUp,
    ExtendSelectionPageDown,
    SelectLine,
    SelectAll,
    CollapseSelections,
    ClearNonPrimarySelections,
    AddSelectionAbove,
    AddSelectionBelow,
    RemovePrimarySelection,
    IncrementPrimarySelection,
    DecrementPrimarySelection,
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
    // all selection manipulation with regex matching can be sub modes of a mode called Match. this would be a popup mode, that then triggers the interactive text box when sub mode entered
    /// for selecting any matching regex from inside selections
    Find,
    /// for retaining everything within selections that isn't a matching regex pattern
    Split,
    // select text within but excluding instances of a single search pattern, a char pair, or a text object
    //SelectInsideExclusive,  //ctrl+e
    //select text within and including instances of a single search pattern, a char pair, or a text object
    //SelectInsideInclusive,  //ctrl+i
    // select surrounding instances of single search pattern, a char pair, or a text object
    //SelectSurrounding
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
                    Mode::View => {keybind::handle_view_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Warning(_) => {keybind::handle_warning_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Goto => {keybind::handle_goto_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Find => {keybind::handle_find_replace_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Command => {keybind::handle_command_mode_keypress(self, key_event.code, key_event.modifiers);}
                    Mode::Notify => {
                        // changes mode back to Insert, without updating UI, so notifications show until next keypress
                        //if self.mode == Mode::Notify{self.mode = Mode::Insert;} //ensure we return back to insert mode  //TODO: if is redundant in match, can set mode without check
                        self.set_mode(Mode::Insert);
                        keybind::handle_insert_mode_keypress(self, key_event.code, key_event.modifiers);
                    }
                    Mode::Split => {keybind::handle_split_mode_keypress(self, key_event.code, key_event.modifiers);}
                }
            },
            event::Event::Mouse(idk) => {
                //TODO: maybe mode specific mouse handling...
                match idk.kind{
                    event::MouseEventKind::Down(something) => {}
                    event::MouseEventKind::Up(something) => {}
                    event::MouseEventKind::Drag(something) => {}
                    event::MouseEventKind::Moved => {}
                    event::MouseEventKind::ScrollDown => {}
                    event::MouseEventKind::ScrollUp => {}
                }
            }
            event::Event::Resize(x, y) => self.resize(x, y),
            event::Event::FocusLost => {/*do nothing*/}
            event::Event::FocusGained => {/*do nothing*/}
            event::Event::Paste(idk) => {}
            //_ => self.no_op_event(),
        }

        Ok(())
    }

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
        // ui layouts need to be updated before doc size set, so doc size can be calculated correctly
        self.ui.update_layouts(&self.mode);
        self.update_ui_data_util_bar(); //TODO: can this be called later in fn impl?
        self.document.view_mut().set_size(self.ui.document_viewport.document_widget.rect.width as usize, self.ui.document_viewport.document_widget.rect.height as usize);
        // scrolling so cursor is in a reasonable place, and updating so any ui changes render correctly
        self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_document);
    }

    pub fn esc_handle(&mut self){
        assert!(self.mode == Mode::Insert);
        //if self.ui.util_bar.utility_widget.display_copied_indicator{self.ui.util_bar.utility_widget.display_copied_indicator = false;}
        //TODO: if lsp suggestions displaying(currently unimplemented), exit that display
        /*else */if self.document.selections().count() > 1{
            //self.clear_non_primary_selections();
            self.selection_action(SelectionAction::ClearNonPrimarySelections);
        }
        else if self.document.selections().primary().is_extended(CURSOR_SEMANTICS){
            //self.collapse_selections();
            self.selection_action(SelectionAction::CollapseSelections);
        }
        else{
            if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}
        }
    }

    pub fn set_mode(&mut self, to_mode: Mode){
        let mut to_mode_uses_util_text = false;
        let mut update_layouts_and_document = false;
        let mut store_current_selections = false;
        match to_mode{
            Mode::Insert => {
                if self.mode == Mode::Goto 
                || self.mode == Mode::Find 
                || self.mode == Mode::Split 
                || self.mode == Mode::Command{
                    update_layouts_and_document = true;
                }else{/*do nothing*/}
            }
            Mode::View | Mode::Notify | Mode::Warning(_)=> {/*do nothing*/}
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
        self.mode = to_mode.clone();

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

    pub fn save(&mut self){
        assert!(self.mode == Mode::Insert);
        match self.document.save(){
            Ok(()) => {self.update_ui_data_document();}
            Err(_) => {self.set_mode(Mode::Warning(WarningKind::FileSaveFailed));}
        }
    }
    fn handle_document_error(&mut self, e: DocumentError){
        let this_file = std::panic::Location::caller().file();  //actually, these should prob be assigned in calling fn, and passed in, so that error location is the caller and not always here...
        let line_number = std::panic::Location::caller().line();
        match e{
            DocumentError::InvalidInput => {self.set_mode(Mode::Warning(WarningKind::InvalidInput));}
            DocumentError::SelectionAtDocBounds |
            DocumentError::NoChangesToUndo |
            DocumentError::NoChangesToRedo => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
            DocumentError::SelectionsError(s) => {
                match s{
                    SelectionsError::ResultsInSameState |
                    SelectionsError::CannotAddSelectionAbove |
                    SelectionsError::CannotAddSelectionBelow => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
                    SelectionsError::MultipleSelections => {self.set_mode(Mode::Warning(WarningKind::MultipleSelections));}
                    SelectionsError::SingleSelection => {self.set_mode(Mode::Warning(WarningKind::SingleSelection));}
                    SelectionsError::NoSearchMatches |
                    SelectionsError::SpansMultipleLines => self.set_mode(Mode::Warning(WarningKind::UnhandledError(format!("{s:#?} at {this_file}::{line_number}. This Error shouldn't be possible here.")))),
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

    pub fn selection_action(&mut self, action: SelectionAction){
        assert!(self.mode == Mode::Insert);
        let result = match action{
            SelectionAction::MoveCursorUp => {self.document.move_cursor_up(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorDown => {self.document.move_cursor_down(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorLeft => {self.document.move_cursor_left(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorRight => {self.document.move_cursor_right(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorWordBoundaryForward => {self.document.move_cursor_word_boundary_forward(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorWordBoundaryBackward => {self.document.move_cursor_word_boundary_backward(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorLineEnd => {self.document.move_cursor_line_end(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorHome => {self.document.move_cursor_home(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorDocumentStart => {self.document.move_cursor_document_start(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorDocumentEnd => {self.document.move_cursor_document_end(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorPageUp => {self.document.move_cursor_page_up(CURSOR_SEMANTICS)}
            SelectionAction::MoveCursorPageDown => {self.document.move_cursor_page_down(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionUp => {self.document.extend_selection_up(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionDown => {self.document.extend_selection_down(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionLeft => {self.document.extend_selection_left(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionRight => {self.document.extend_selection_right(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionWordBoundaryBackward => {self.document.extend_selection_word_boundary_backward(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionWordBoundaryForward => {self.document.extend_selection_word_boundary_forward(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionLineEnd => {self.document.extend_selection_line_end(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionHome => {self.document.extend_selection_home(CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionDocumentStart => {self.document.extend_selection_document_start(CURSOR_SEMANTICS)}
            //SelectionAction::ExtendSelectionDocumentEnd => {self.document.extend_selection_document_end(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionPageUp => {self.document.extend_selection_page_up(CURSOR_SEMANTICS)}
            SelectionAction::ExtendSelectionPageDown => {self.document.extend_selection_page_down(CURSOR_SEMANTICS)}
            SelectionAction::SelectLine => {self.document.select_line(CURSOR_SEMANTICS)}
            SelectionAction::SelectAll => {self.document.select_all(CURSOR_SEMANTICS)}
            SelectionAction::CollapseSelections => {self.document.collapse_selections(CURSOR_SEMANTICS)}
            SelectionAction::ClearNonPrimarySelections => {self.document.clear_non_primary_selections()}
            SelectionAction::AddSelectionAbove => {self.document.add_selection_above(CURSOR_SEMANTICS)}
            SelectionAction::AddSelectionBelow => {self.document.add_selection_below(CURSOR_SEMANTICS)}
            SelectionAction::RemovePrimarySelection => {self.document.remove_primary_selection()}
            SelectionAction::IncrementPrimarySelection => {self.document.increment_primary_selection()}
            SelectionAction::DecrementPrimarySelection => {self.document.decrement_primary_selection()}
        };

        match result{
            Ok(()) => {
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Err(e) => {self.handle_document_error(e);}
        }
    }

    pub fn view_action(&mut self, action: ViewAction){      //TODO: make separate view mode, and call this from there   //TODO: make sure this can still be called from insert, so users can assign a direct keybind if desired
        assert!(/*self.mode == Mode::Insert || */self.mode == Mode::View); //TODO: assert!(self.mode == Mode::View);
        let view = self.document.view();

        let mut should_exit = false;
        let result = match action{
            ViewAction::CenterVerticallyAroundCursor => {
                should_exit = true;
                view.center_vertically_around_cursor(self.document.selections().primary(), self.document.text(), CURSOR_SEMANTICS)
            }
            ViewAction::ScrollUp => {view.scroll_up(VIEW_SCROLL_AMOUNT)}
            ViewAction::ScrollDown => {view.scroll_down(VIEW_SCROLL_AMOUNT, self.document.text())}
            ViewAction::ScrollLeft => {view.scroll_left(VIEW_SCROLL_AMOUNT)}
            ViewAction::ScrollRight => {view.scroll_right(VIEW_SCROLL_AMOUNT, self.document.text())}
        };
        match result{
            Ok(new_view) => {
                *self.document.view_mut() = new_view;
                self.update_ui_data_document();
                if should_exit{self.set_mode(Mode::Insert);}
            }
            Err(e) => {
                match e{
                    ViewError::InvalidInput => {self.set_mode(Mode::Warning(WarningKind::InvalidInput));}
                    ViewError::ResultsInSameState => {if SHOW_SAME_STATE_WARNINGS{self.set_mode(Mode::Warning(WarningKind::SameState));}}
                }
            }
        }
    }

    //TODO: split into util_edit_action and util_selection_action
    pub fn generic_util_action(&mut self, action: UtilAction){
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

        //perform any mode specific follow up actions
        match self.mode{
            Mode::Insert |
            Mode::View |
            Mode::Notify |
            Mode::Warning(_) => {/*do nothing*/}
            Mode::Goto => {
                self.goto_mode_text_validity_check();
            }
            Mode::Find => {
                self.incremental_search();
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Mode::Split => {
                self.incremental_split();
                self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
            }
            Mode::Command => {/*do nothing*/}
        }
    }

    pub fn goto_mode_accept(&mut self){
        assert!(self.mode == Mode::Goto);
        let mut show_warning = false;
        if let Ok(line_number) = self.ui.util_bar.utility_widget.text_box.text.to_string().parse::<usize>(){
            if line_number == 0{show_warning = true;}   //we have no line number 0, so this is invalid
            else{
                let line_number = line_number.saturating_sub(1);    // make line number 0 based for interfacing correctly with backend impl
                match self.document.move_to_line_number(line_number, CURSOR_SEMANTICS){
                    Ok(()) => {self.checked_scroll_and_update(&self.document.selections().primary().clone(), Application::update_ui_data_selections, Application::update_ui_data_selections);}  //TODO: pretty sure one of these should be update_ui_data_document
                    Err(_) => {show_warning = true;}    //TODO: match error and handle
                }
            }
        }else{show_warning = true;}
        if show_warning{self.set_mode(Mode::Warning(WarningKind::InvalidInput));}
        else{self.set_mode(Mode::Insert);}
    }
    // Not entirely sure I want this behavior...
    pub fn goto_mode_selection_action(&mut self, action: SelectionAction){  //TODO: this is pretty slow when user enters a large number into util text box
        assert!(self.mode == Mode::Goto);
        if let Ok(amount) = self.ui.util_bar.utility_widget.text_box.text.to_string().parse::<usize>(){
            self.set_mode(Mode::Insert);    //or else the selection action will panic
            for _ in 0..amount{
                if matches!(self.mode, Mode::Warning(_)){break;}    //trying to speed this up by preventing this from running `amount` times, if there has already been an error
                self.selection_action(action.clone());  //TODO: if this reaches doc boundaries, this will display same state warning. which it technically may not be the same state as when this fn was called...
            }
        }else{
            self.set_mode(Mode::Warning(WarningKind::InvalidInput));
        }
        //also, this doesn't work with goto_mode_text_validity_check
    }
    pub fn goto_mode_text_validity_check(&mut self){
        assert!(self.mode == Mode::Goto);
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

    fn incremental_search(&mut self){   //this def doesn't work correctly with utf-8 yet
        if let Some(selections) = self.ui.util_bar.utility_widget.selections_before_search.clone(){
            if let Ok(selections) = selections.search(&self.ui.util_bar.utility_widget.text_box.text.to_string(), self.document.text()){    //TODO: selection management should be done in edit_core::document.rs
                self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
                *self.document.selections_mut() = selections;
            }else{  //TODO: may want to match on error to make sure we are handling this correctly
                self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
                *self.document.selections_mut() = self.ui.util_bar.utility_widget.selections_before_search.clone().unwrap();
            }
            //TODO: if no selection extended, search whole document
        }
    }
    pub fn restore_selections_and_exit(&mut self){
        self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        *self.document.selections_mut() = self.ui.util_bar.utility_widget.selections_before_search.clone().unwrap();
        self.set_mode(Mode::Insert);
    }

    fn incremental_split(&mut self){
        if let Some(selections) = self.ui.util_bar.utility_widget.selections_before_search.clone(){
            if let Ok(selections) = selections.split(&self.ui.util_bar.utility_widget.text_box.text.to_string(), self.document.text()){ //TODO: selection management should be done in edit_core::document.rs
                self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
                *self.document.selections_mut() = selections;
            }else{  //TODO: may want to match on error to make sure we are handling this correctly
                self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
                *self.document.selections_mut() = self.ui.util_bar.utility_widget.selections_before_search.clone().unwrap();
            }
        }
    }

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
        if warn{self.mode = Mode::Warning(WarningKind::CommandParseFailed);}
        else{self.set_mode(Mode::Insert);}
    }
}

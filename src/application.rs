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
// if an error occurs, display error in error mode util widget
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

use std::path::PathBuf;
use crossterm::event;
use ratatui::{
    prelude::*,
    widgets::*
};
use crate::{
    config::*,
    keybind,
    range::Range,
    buffer::Buffer,
    display_area::{DisplayArea, DisplayAreaError},
    mode_stack::ModeStack,
    selection::{Selection, CursorSemantics},
    selections::{Selections, SelectionsError},
    ui::{UserInterface, util_bar::*},
    history::{Change, ChangeSet, Operation},
};



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
        //TODO: CenterHorizontallyAroundCursor,
        //TODO: AlignWithCursorAtTop,
        //TODO: AlignWithCursorAtBottom,    
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}
pub enum EditAction{
        //TODO: AlignSelectedTextVertically,
    InsertChar(char),
    InsertNewline,
    InsertTab,
    Delete,
        //TODO: DeleteToNextWordBoundary,
        //TODO: DeleteToPrevWordBoundary,
    Backspace,
    Cut,
    Paste,
    Undo,
    Redo,
        //TODO: SwapUp,   (if text selected, swap selected text with line above. if no selection, swap current line with line above)
        //TODO: SwapDown, (if text selected, swap selected text with line below. if no selection, swap current line with line below)
        //TODO: RotateTextInSelections,
    AddSurround(char, char),
}
pub enum SelectionAction{   //TODO?: have (all?) selection actions take an amount, for action repetition. MoveCursorDown(2) would move the cursor down two lines, if possible, or saturate at buffer end otherwise, and error if already at buffer end
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorWordBoundaryForward,  //TODO: this isn't working with count, for some reason. check move_cursor_word_boundary_backward impl to determine cause...
    MoveCursorWordBoundaryBackward, //TODO: this isn't working with count, for some reason. check move_cursor_word_boundary_forward impl to determine cause...
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
    ExtendSelectionWordBoundaryBackward,    //TODO: this isn't working with count, for some reason. check extend_selection_word_boundary_backward impl to determine cause...
    ExtendSelectionWordBoundaryForward,     //TODO: this isn't working with count, for some reason. check extend_selection_word_boundary_forward impl to determine cause...
    ExtendSelectionLineEnd,
    ExtendSelectionHome,
        //TODO: ExtendSelectionBufferStart,
        //TODO: ExtendSelectionBufferEnd,
        //TODO: ExtendSelectionPageUp,
        //TODO: ExtendSelectionPageDown,
    SelectLine,           //TODO: this may benefit from using a count. would the next count # of lines including current
    SelectAll,
    CollapseSelectionToAnchor,
    CollapseSelectionToCursor,
    ClearNonPrimarySelections,
    AddSelectionAbove,    //TODO: this may benefit from using a count. would add count # of selections
    AddSelectionBelow,    //TODO: this may benefit from using a count. would add count # of selections
    RemovePrimarySelection,
    IncrementPrimarySelection,  //TODO: this may benefit from using a count. would increment primary selection index by 'count'
    DecrementPrimarySelection,  //TODO: this may benefit from using a count. would decrement primary selection index by 'count'
    Surround,         //this would not benefit from using a count. use existing selection primitives to select text to surround
    SurroundingPair,  //TODO: this may benefit from using a count. would select the 'count'th surrounding pair
    FlipDirection,
        //TODO: SplitSelectionLines,    //split current selection into a selection for each line. error if single line
}

#[derive(Clone, PartialEq, Debug)]
pub enum Mode{
    /// for editing text and moving/extending selections
    Insert,
    
    /// for display of errors in the use of the editor(such as invalid input)
    /// should block input until mode exited
    /// to be displayed in ERROR_MODE_BACKGROUND_COLOR and ERROR_MODE_FOREGROUND_COLOR
    Error(String),   //maybe same state warnings should be in notify, so they don't block
    
    /// for display of warnings(such as same state)
    /// unhandled keybinds should fall through to Insert mode, clearing util bar
    /// to be displayed in WARNING_MODE_BACKGROUND_COLOR and WARNING_MODE_FOREGROUND_COLOR
    Warning(String), 
    
    /// for display of notifications(such as text copied indicator, or "action performed outside of view" for non-visible actions)
    /// unhandled keybinds should fall through to Insert mode, clearing util bar
    /// to be displayed in NOTIFY_MODE_BACKGROUND_COLOR and NOTIFY_MODE_FOREGROUND_COLOR
    Notify(String),
    
    /// for display of any information(such as resolved command variables)
    /// unhandled keybinds should fall through to Insert mode, clearing util bar
    /// to be displayed in INFO_MODE_BACKGROUND_COLOR and INFO_MODE_FOREGROUND_COLOR
    /// for example, the command: info %{file_name} , should display the file name or None in the util bar
    /// or info date    , should display the current date in the util bar
    Info(String),
    
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



pub enum ApplicationError{
    ReadOnlyBuffer,
    InvalidInput,
    SelectionAtDocBounds,
    NoChangesToUndo,
    NoChangesToRedo,
    SelectionsError(SelectionsError),
}
pub struct Application{
//these will be client constructs when client/server arichitecture impled...
    should_quit: bool,
    mode_stack: ModeStack,
    pub ui: UserInterface, 
    pub buffer_horizontal_start: usize,
    pub buffer_vertical_start: usize,

//these will be server constructs when client/server architecture impled...
    pub config: Config,
    pub buffer: Buffer, 
    pub preserved_selections: Option<Selections>, 
    pub undo_stack: Vec<ChangeSet>,   //maybe have separate buffer and selections undo/redo stacks?...
    pub redo_stack: Vec<ChangeSet>,
    pub selections: Selections,
    pub clipboard: String,
}
impl Application{
    pub fn new(
        config: Config, 
        display_line_numbers_on_startup: bool, 
        display_status_bar_on_startup: bool, 
        buffer_text: &str, 
        file_path: Option<PathBuf>, 
        read_only: bool, 
        terminal: &Terminal<impl Backend>
    ) -> Result<Self, String>{
        let terminal_size = match terminal.size(){
            Ok(size) => size,
            Err(e) => return Err(format!("{}", e))
        };
        let terminal_rect = Rect::new(0, 0, terminal_size.width, terminal_size.height);

        let buffer = Buffer::new(buffer_text, file_path.clone(), read_only);
        let mut instance = Self{
            should_quit: false,
            mode_stack: ModeStack::default(),
        ui: UserInterface::new(terminal_rect),
            config: config.clone(),
            buffer: buffer.clone(),
            preserved_selections: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selections: Selections::new(
                vec![
                    Selection::new_from_range(
                        match config.semantics.clone(){
                            CursorSemantics::Bar => Range::new(0, 0),
                            CursorSemantics::Block => Range::new(0, buffer.next_grapheme_boundary_index(0))
                        },
                        None, 
                        &buffer, 
                        config.semantics.clone())
                ], 
                0, 
                &buffer, 
                config.semantics.clone()
            ),
            buffer_horizontal_start: 0,
            buffer_vertical_start: 0,
            clipboard: String::new()
        };

        instance.setup(display_line_numbers_on_startup, display_status_bar_on_startup);

        Ok(instance)
    }
    fn setup(&mut self, display_line_numbers_on_startup: bool, display_status_bar_on_startup: bool/*TODO:, cursor_line_number: usize, cursor_column_number: usize */){
        self.ui.document_viewport.line_number_widget.show = display_line_numbers_on_startup;
        self.ui.status_bar.show = display_status_bar_on_startup;

        if self.buffer.read_only{
            self.ui.status_bar.read_only_widget.show = true;
            self.ui.status_bar.read_only_widget.text = "ReadOnly".to_string();
        }else{
            self.ui.status_bar.read_only_widget.show = false;
            self.ui.status_bar.read_only_widget.text = String::new();
        }
        
        if self.buffer.file_path.is_some(){
            self.ui.status_bar.file_name_widget.show = true;
            if self.config.use_full_file_path{
                self.ui.status_bar.file_name_widget.text = self.buffer.file_path().unwrap_or_default();
            }else{
                self.ui.status_bar.file_name_widget.text = self.buffer.file_name().unwrap_or_default();
            }
        }else{
            self.ui.status_bar.file_name_widget.show = false;
            self.ui.status_bar.file_name_widget.text = String::new();
        }

        self.update_ui_data_mode();
        
        //self.ui.document_viewport.document_widget.doc_length = self.buffer.len_lines();
        
        //self.ui.update_layouts(&self.mode());
        //crate::ui::update_layouts(self);
        self.update_layouts();

        // prefer this over scroll_and_update, even when response fns are the same, because it saves us from unnecessarily reassigning the view
        self.checked_scroll_and_update(
            &self.selections.primary().clone(), 
            Application::update_ui_data_document, 
            Application::update_ui_data_document
        );
        self.update_ui_data_util_bar(); //needed for util bar cursor to render the first time it is triggered
    }

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), String>{
        //TODO?: run setup() //set inital ui state  //or is this better left being called from new()?
        while !self.should_quit{
            // could we get terminal size here, and store it in Application, so that we don't have to calculate it in update_layouts, or have an explicit set size?
            // i don't think so, because some functions call update layouts in between run cycles, so size may be outdated...

            //derive User Interface from Application state
            self.update_layouts();
            self.render(terminal)?;
            
            //update Application state
            self.handle_event()?;
        }
        Ok(())
    }

    pub fn update_layouts(&mut self){   //-> Result<(), String>{ //to handle terminal.size() error
        //TODO: terminal.size() should be called here, instead of storing terminal_size
        // this will require all calling functions to return a Result. handle changes to action fns before doing this...
        //let terminal_size = match terminal.size(){
        //    Ok(size) => size,
        //    Err(e) => return Err(format!("{}", e))
        //};
        //let terminal_size = Rect::new(0, 0, terminal_size.width, terminal_size.height);
    
        let terminal_rect = self.layout_terminal(self.ui.terminal_size);
        let document_viewport_rect = self.layout_buffer_viewport(terminal_rect[0]);
        let status_bar_rect = self.layout_status_bar(terminal_rect[1]);
        let util_rect = self.layout_util_bar(terminal_rect[2]);
    
        self.ui.document_viewport.line_number_widget.rect = document_viewport_rect[0];
        self.ui.document_viewport.padding.rect = document_viewport_rect[1];
        self.ui.document_viewport.document_widget.rect = document_viewport_rect[2];
            
        self.ui.status_bar.read_only_widget.rect = status_bar_rect[0];
        self.ui.status_bar.padding_1.rect = status_bar_rect[1];
        self.ui.status_bar.file_name_widget.rect = status_bar_rect[2];
        self.ui.status_bar.padding_2.rect = status_bar_rect[3];
        self.ui.status_bar.modified_widget.rect = status_bar_rect[4];
        self.ui.status_bar.selections_widget.rect = status_bar_rect[5];
        self.ui.status_bar.cursor_position_widget.rect = status_bar_rect[6];
        self.ui.status_bar.padding_3.rect = status_bar_rect[7];
        self.ui.status_bar.mode_widget.rect = status_bar_rect[8];
            
        self.ui.util_bar.prompt.rect = util_rect[0];
        self.ui.util_bar.utility_widget.rect = util_rect[1];
            
        self.ui.popups.goto.rect = crate::ui::sized_centered_rect(self.ui.popups.goto.widest_element_len, self.ui.popups.goto.num_elements, self.ui.terminal_size);
        self.ui.popups.command.rect = crate::ui::sized_centered_rect(self.ui.popups.command.widest_element_len, self.ui.popups.command.num_elements, self.ui.terminal_size);
        self.ui.popups.find.rect = crate::ui::sized_centered_rect(self.ui.popups.find.widest_element_len, self.ui.popups.find.num_elements, self.ui.terminal_size);
        self.ui.popups.split.rect = crate::ui::sized_centered_rect(self.ui.popups.split.widest_element_len, self.ui.popups.split.num_elements, self.ui.terminal_size);
        self.ui.popups.error.rect = crate::ui::sized_centered_rect(self.ui.popups.error.widest_element_len, self.ui.popups.error.num_elements, self.ui.terminal_size);
        self.ui.popups.modified_error.rect = crate::ui::sized_centered_rect(self.ui.popups.modified_error.widest_element_len, self.ui.popups.modified_error.num_elements, self.ui.terminal_size);
        self.ui.popups.warning.rect = crate::ui::sized_centered_rect(self.ui.popups.warning.widest_element_len, self.ui.popups.warning.num_elements, self.ui.terminal_size);
        self.ui.popups.notify.rect = crate::ui::sized_centered_rect(self.ui.popups.notify.widest_element_len, self.ui.popups.notify.num_elements, self.ui.terminal_size);
        self.ui.popups.info.rect = crate::ui::sized_centered_rect(self.ui.popups.info.widest_element_len, self.ui.popups.info.num_elements, self.ui.terminal_size);
        self.ui.popups.view.rect = crate::ui::sized_centered_rect(self.ui.popups.view.widest_element_len, self.ui.popups.view.num_elements, self.ui.terminal_size);
        self.ui.popups.object.rect = crate::ui::sized_centered_rect(self.ui.popups.object.widest_element_len, self.ui.popups.object.num_elements, self.ui.terminal_size);
        self.ui.popups.add_surround.rect = crate::ui::sized_centered_rect(self.ui.popups.add_surround.widest_element_len, self.ui.popups.add_surround.num_elements, self.ui.terminal_size);
    }
    fn layout_terminal(&self, terminal_size: Rect) -> std::rc::Rc<[Rect]>{       //TODO: maybe rename layout_terminal_vertical_ui_components
        // layout of the whole terminal screen
        Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(
                vec![
                    //[0]
                    // document + line num rect height
                    Constraint::Min(0),

                    //[1]
                    // status bar rect height
                    Constraint::Length(if self.ui.status_bar.show{1}else{0}),

                    //[2]
                    // util(goto/find/command) bar rect height
                    Constraint::Length(
                        match &self.mode(){
                            Mode::Error(_) | 
                            Mode::Warning(_) | 
                            Mode::Notify(_) | 
                            Mode::Info(_) | 
                            Mode::Command | 
                            Mode::Find | 
                            Mode::Goto | 
                            Mode::Split => 1,
    
                            Mode::Object |
                            Mode::Insert |
                            Mode::View |
                            Mode::AddSurround => if self.ui.status_bar.show{1}else{0}
                        }
                    )
                ]
            )
            .split(terminal_size)
    }
    fn layout_buffer_viewport(&self, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of document + line num rect
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    //[0]
                    // line number rect width
                    Constraint::Length(
                        if self.ui.document_viewport.line_number_widget.show{
                            crate::ui::count_digits(self.buffer.len_lines())
                        }else{0}
                    ),
    
                    //[1]
                    // line number right padding
                    Constraint::Length(
                        if self.ui.document_viewport.line_number_widget.show{
                            1
                        }else{0}
                    ),
    
                    //[2]
                    // document rect width
                    Constraint::Min(5)
                ]
            )
            .split(rect)
    }
    fn layout_status_bar(&self, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of status bar rect (modified_indicator/file_name/cursor_position)
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    //[0]
                    // read_only widget
                    Constraint::Max(
                        self.ui.status_bar.read_only_widget.text.len() as u16
                    ),
    
                    //[1]
                    // padding_1
                    Constraint::Max(
                        if self.buffer.read_only{
                            1
                        }else{0}
                    ),
                    
                    //[2]
                    // file_name widget
                    Constraint::Max(
                        self.ui.status_bar.file_name_widget.text.len() as u16
                    ),
    
                    //[3]
                    // padding_2
                    Constraint::Max(
                        if self.buffer.is_modified(){
                            1
                        }else{0}
                    ),
                    
                    //[4]
                    // modified widget
                    Constraint::Max(
                        self.ui.status_bar.modified_widget.text.len() as u16
                    ),
                    
                    //[5]
                    // selections widget
                    Constraint::Min(0),     //or set selections widget to Max, and surround with 2 padding widgets set to Min(0)?...idk if that will work the same?...
                    
                    //[6]
                    // cursor position indicator width
                    Constraint::Max(
                        self.ui.status_bar.cursor_position_widget.text.len() as u16
                    ),
    
                    //[7]
                    // padding_3
                    Constraint::Max(1),
    
                    //[8]
                    // mode widget
                    Constraint::Max(
                        self.ui.status_bar.mode_widget.text.len() as u16
                    ),
                ]
            )
            .split(rect)
    }
    fn layout_util_bar(&self, rect: Rect) -> std::rc::Rc<[Rect]>{
        use crate::ui::util_bar::{GOTO_PROMPT, FIND_PROMPT, SPLIT_PROMPT, COMMAND_PROMPT};
        // layout of util rect (goto/find/command/save as)
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    //[0]
                    // util bar prompt width
                    Constraint::Length(
                        match self.mode(){
                            Mode::Goto => GOTO_PROMPT.len() as u16,
                            Mode::Find => FIND_PROMPT.len() as u16,
                            Mode::Split => SPLIT_PROMPT.len() as u16,
                            Mode::Command => COMMAND_PROMPT.len() as u16,
                            Mode::Error(_)
                            | Mode::Warning(_)
                            | Mode::Notify(_)
                            | Mode::Info(_)
                            | Mode::Insert
                            | Mode::Object
                            | Mode::View 
                            | Mode::AddSurround => 0
                        }
                    ),

                    //[1]
                    // util bar rect width
                    Constraint::Length(
                        match self.mode(){
                            Mode::Insert
                            | Mode::Object
                            | Mode::View
                            | Mode::Error(_) 
                            | Mode::Warning(_)
                            | Mode::Notify(_)
                            | Mode::Info(_)
                            | Mode::AddSurround => rect.width,
                            Mode::Goto => rect.width - GOTO_PROMPT.len() as u16,
                            Mode::Command => rect.width - COMMAND_PROMPT.len() as u16,
                            Mode::Find => rect.width - FIND_PROMPT.len() as u16,
                            Mode::Split => rect.width - SPLIT_PROMPT.len() as u16,
                        }
                    ),
                    // used to fill in space when other two are 0 length
                    Constraint::Length(0)
                ]
            )
            .split(rect)
    }

    pub fn render(&self, terminal: &mut Terminal<impl Backend>) -> Result<(), String>{
        match terminal.draw(
            |frame| {
                // always render
                render_widget(self.ui.document_viewport.document_widget.text.clone(), self.ui.document_viewport.document_widget.rect, Alignment::Left, false, DOCUMENT_BACKGROUND_COLOR, DOCUMENT_FOREGROUND_COLOR, frame);
                self.render_buffer_highlights(frame.buffer_mut());
                
                // conditionally render
                if self.ui.document_viewport.line_number_widget.show{
                    render_widget(self.ui.document_viewport.line_number_widget.text.clone(), self.ui.document_viewport.line_number_widget.rect, Alignment::Right, false, LINE_NUMBER_BACKGROUND_COLOR, LINE_NUMBER_FOREGROUND_COLOR, frame);
                    render_widget(String::new(), self.ui.document_viewport.padding.rect, Alignment::Center, false, LINE_NUMBER_BACKGROUND_COLOR, LINE_NUMBER_BACKGROUND_COLOR, frame);
                }
                if self.ui.status_bar.show{
                    //instead of read_only_widget.text, we could do: if app.buffer.read_only{"ReadOnly"}else{String::new()}
                    render_widget(self.ui.status_bar.read_only_widget.text.clone(), self.ui.status_bar.read_only_widget.rect, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, READ_ONLY_WIDGET_FOREGROUND_COLOR, frame);
                    render_widget(String::new(), self.ui.status_bar.padding_1.rect, Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, Color::Red, frame);
                    render_widget(self.ui.status_bar.file_name_widget.text.clone(), self.ui.status_bar.file_name_widget.rect, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, FILE_NAME_WIDGET_FOREGROUND_COLOR, frame);
                    render_widget(String::new(), self.ui.status_bar.padding_2.rect, Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, Color::Red, frame);
                    render_widget(self.ui.status_bar.modified_widget.text.clone(), self.ui.status_bar.modified_widget.rect, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, MODIFIED_WIDGET_FOREGROUND_COLOR, frame);
                    render_widget(self.ui.status_bar.selections_widget.text.clone(), self.ui.status_bar.selections_widget.rect, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, SELECTIONS_WIDGET_FOREGROUND_COLOR, frame);
                    render_widget(self.ui.status_bar.cursor_position_widget.text.clone(), self.ui.status_bar.cursor_position_widget.rect, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, CURSOR_POSITION_WIDGET_FOREGROUND_COLOR, frame);
                    render_widget(String::new(), self.ui.status_bar.padding_3.rect, Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, Color::Red, frame);
                    render_widget(self.ui.status_bar.mode_widget.text.clone(), self.ui.status_bar.mode_widget.rect, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, MODE_WIDGET_FOREGROUND_COLOR, frame);
                }
    
                // render according to mode
                match self.mode(){
                    Mode::Insert => {
                        // built in cursor handling. now handling cursor rendering ourselves
                        // frame.set_cursor_position((
                        //     self.document_viewport.document_widget.rect.x + pos.x() as u16,
                        //     self.document_viewport.document_widget.rect.y + pos.y() as u16
                        // ))
                    }
                    Mode::Goto => {
                        render_widget(GOTO_PROMPT.to_string(), self.ui.util_bar.prompt.rect, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, frame);
                        render_widget(self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), self.ui.util_bar.utility_widget.rect, Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, if self.ui.util_bar.utility_widget.text_box.text_is_valid{UTIL_BAR_FOREGROUND_COLOR}else{UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR}, frame);
                        self.render_util_bar_highlights(frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.goto.rect);
                            render_popup(self.ui.popups.goto.text.clone(), self.ui.popups.goto.title.clone(), self.ui.popups.goto.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::Command => {
                        render_widget(COMMAND_PROMPT.to_string(), self.ui.util_bar.prompt.rect, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, frame);
                        render_widget(self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), self.ui.util_bar.utility_widget.rect, Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, frame);
                        self.render_util_bar_highlights(frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.command.rect);
                            render_popup(self.ui.popups.command.text.clone(), self.ui.popups.command.title.clone(), self.ui.popups.command.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::Find => {
                        render_widget(FIND_PROMPT.to_string(), self.ui.util_bar.prompt.rect, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, frame);
                        render_widget(self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), self.ui.util_bar.utility_widget.rect, Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, if self.ui.util_bar.utility_widget.text_box.text_is_valid{UTIL_BAR_FOREGROUND_COLOR}else{UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR}, frame);
                        self.render_util_bar_highlights(frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.find.rect);
                            render_popup(self.ui.popups.find.text.clone(), self.ui.popups.find.title.clone(), self.ui.popups.find.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::Split => {
                        render_widget(SPLIT_PROMPT.to_string(), self.ui.util_bar.prompt.rect, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR, frame);
                        render_widget(self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), self.ui.util_bar.utility_widget.rect, Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, if self.ui.util_bar.utility_widget.text_box.text_is_valid{UTIL_BAR_FOREGROUND_COLOR}else{UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR}, frame);
                        self.render_util_bar_highlights(frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.split.rect);
                            render_popup(self.ui.popups.split.text.clone(), self.ui.popups.split.title.clone(), self.ui.popups.split.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::Error(string) => {
                        render_widget(string.clone(), self.ui.util_bar.utility_widget.rect, Alignment::Center, true, ERROR_BACKGROUND_COLOR, ERROR_FOREGROUND_COLOR, frame);
                        if string == FILE_MODIFIED{
                            if SHOW_CONTEXTUAL_KEYBINDS{
                                frame.render_widget(ratatui::widgets::Clear, self.ui.popups.modified_error.rect);
                                render_popup(self.ui.popups.error.text.clone(), self.ui.popups.error.title.clone(), self.ui.popups.error.rect, Color::Black, Color::Yellow, frame);
                            }
                        }
                        else{
                            if SHOW_CONTEXTUAL_KEYBINDS{
                                frame.render_widget(ratatui::widgets::Clear, self.ui.popups.error.rect);
                                render_popup(self.ui.popups.error.text.clone(), self.ui.popups.error.title.clone(), self.ui.popups.error.rect, Color::Black, Color::Yellow, frame);
                            }
                        }
                    }
                    Mode::Warning(string) => {
                        render_widget(string.clone(), self.ui.util_bar.utility_widget.rect, Alignment::Center, true, WARNING_BACKGROUND_COLOR, WARNING_FOREGROUND_COLOR, frame);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.warning.rect);
                            render_popup(self.ui.popups.warning.text.clone(), self.ui.popups.warning.title.clone(), self.ui.popups.warning.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::Notify(string) => {
                        render_widget(string.clone(), self.ui.util_bar.utility_widget.rect, Alignment::Center, true, NOTIFY_BACKGROUND_COLOR, NOTIFY_FOREGROUND_COLOR, frame);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.notify.rect);
                            render_popup(self.ui.popups.notify.text.clone(), self.ui.popups.notify.title.clone(), self.ui.popups.notify.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::Info(string) => {
                        render_widget(string.clone(), self.ui.util_bar.utility_widget.rect, Alignment::Center, true, INFO_BACKGROUND_COLOR, INFO_FOREGROUND_COLOR, frame);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.info.rect);
                            render_popup(self.ui.popups.info.text.clone(), self.ui.popups.info.title.clone(), self.ui.popups.info.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::View => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.view.rect);
                            render_popup(self.ui.popups.view.text.clone(), self.ui.popups.view.title.clone(), self.ui.popups.view.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::Object => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.object.rect);
                            render_popup(self.ui.popups.object.text.clone(), self.ui.popups.object.title.clone(), self.ui.popups.object.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                    Mode::AddSurround => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.add_surround.rect);
                            render_popup(self.ui.popups.add_surround.text.clone(), self.ui.popups.add_surround.title.clone(), self.ui.popups.add_surround.rect, Color::Black, Color::Yellow, frame);
                        }
                    }
                }
            }
        ){
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{e}"))
        }
    }
    fn render_buffer_highlights(&self, buf: &mut ratatui::prelude::Buffer){
        // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        let area = self.ui.document_viewport.document_widget.rect;
        // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    
        let primary_cursor = self.ui.document_viewport.highlighter.primary_cursor.clone();
        let cursors = self.ui.document_viewport.highlighter.cursors.clone();
        let selections = self.ui.document_viewport.highlighter.selections.clone();
        //
        if self.config.show_cursor_column{
            for y in area.top()..area.height{
                if let Some(primary_cursor_position) = &primary_cursor{//self.primary_cursor.clone(){
                    if let Some(cell) = buf.cell_mut((area.left() + primary_cursor_position.x as u16, y)){
                        cell.set_style(
                            Style::default()
                                .bg(CURSOR_COLUMN_BACKGROUND_COLOR)
                                .fg(CURSOR_COLUMN_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }
        if self.config.show_cursor_line{
            for x in area.left()..(area.width + area.left()){
                if let Some(primary_cursor_position) = &primary_cursor{//self.primary_cursor.clone(){
                    if let Some(cell) = buf.cell_mut((x, area.top() + primary_cursor_position.y as u16)){
                        cell.set_style(
                            Style::default()
                                .bg(CURSOR_LINE_BACKGROUND_COLOR)
                                .fg(CURSOR_LINE_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }
    
        //if let Some(selections) = self.selections{  //selection not rendering properly on last empty line following previous newline, when cursor rendering below is not drawn there. maybe this is correct, because there is technically no content there...
        //if !self.selections.is_empty(){
        if !selections.is_empty(){
            for selection in selections{//&self.selections{  //self.selections.iter(){   //change suggested by clippy lint
                if selection.head().x - selection.anchor().x == 0{continue;}    //should this use start and end instead?
                for col in selection.anchor().x../*=*/selection.head().x{
                    let x_pos = area.left() + (col as u16);
                    //let y_pos = selection.head().y as u16;
                    let y_pos = area.top() + (selection.head().y as u16);
        
                    if let Some(cell) = buf.cell_mut((x_pos, y_pos)){
                        cell.set_style(Style::default()
                            .bg(SELECTION_BACKGROUND_COLOR)
                            .fg(SELECTION_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }
    
        //render cursors for all selections
        //if !self.cursors.is_empty(){
        if !cursors.is_empty(){
            for cursor in cursors{//self.cursors{
                if let Some(cell) = buf.cell_mut((area.left() + (cursor.x as u16), area.top() + (cursor.y as u16))){
                    cell.set_style(Style::default()
                        .bg(CURSOR_BACKGROUND_COLOR)
                        .fg(CURSOR_FOREGROUND_COLOR)
                    );
                }
            }
        }
    
        // render primary cursor
        if let Some(cursor) = &primary_cursor{//self.primary_cursor{
            if let Some(cell) = buf.cell_mut((area.left() + (cursor.x as u16), area.top() + (cursor.y as u16))){
                cell.set_style(Style::default()
                    .bg(PRIMARY_CURSOR_BACKGROUND_COLOR)
                    .fg(PRIMARY_CURSOR_FOREGROUND_COLOR)
                );
            }
        }
    
        //debug //this can help ensure we are using the correct Rect
        //if let Some(cell) = buf.cell_mut((area.left(), area.top())){
        //    cell.set_style(
        //        Style::default()
        //            .bg(ratatui::style::Color::Yellow)
        //    );
        //}
    }
    fn render_util_bar_highlights(&self, buf: &mut ratatui::prelude::Buffer){
        let area = self.ui.util_bar.utility_widget.rect;
    
        let selection = self.ui.util_bar.highlighter.selection.clone();
        let cursor = self.ui.util_bar.highlighter.cursor.clone();
    
        //render selection
        if let Some(selection) = selection{//self.selection{
            if selection.head().x - selection.anchor().x > 0{   //if selection extended
                for col in selection.anchor().x..selection.head().x{
                    let x_pos = area.left() + (col as u16);
                    //let y_pos = area.top();
                    let y_pos = area.top() + (selection.head().y as u16);
                    //assert_eq!(0, y_pos, "util bar text should be guaranteed to be one line");    //this seems to be causing issues when moving from end of line...
        
                    if let Some(cell) = buf.cell_mut((x_pos, y_pos)){
                        cell.set_style(Style::default()
                            .bg(SELECTION_BACKGROUND_COLOR)
                            .fg(SELECTION_FOREGROUND_COLOR)
                        );
                    }
                }
            }
        }
    
        // render cursor
        if let Some(cursor) = cursor{//self.cursor{
            assert_eq!(0, cursor.y, "util bar text should be guaranteed to be one line");
            if let Some(cell) = buf.cell_mut((area.left() + (cursor.x as u16), area.top() + (cursor.y as u16))){
                cell.set_style(Style::default()
                    .bg(PRIMARY_CURSOR_BACKGROUND_COLOR)
                    .fg(PRIMARY_CURSOR_FOREGROUND_COLOR)
                );
            }
        }
    
        //debug //this can help ensure we are using the correct Rect
        //if let Some(cell) = buf.cell_mut((area.left(), area.top())){
        //    cell.set_style(
        //        Style::default()
        //            .bg(ratatui::style::Color::Yellow)
        //    );
        //}
    }

    fn handle_event(&mut self) -> Result<(), String>{
        match event::read(){
            Ok(event) => {
                match event{
                    //TODO: handle_keypress fns could take a mode as context, then mode specific functionality wouldn't need to be in separate fns...
                    //that context could also be used to fill available commands in mode specific popup menus
                    event::Event::Key(key_event) => {
                        match self.mode(){
                            Mode::Insert => {keybind::handle_insert_mode_keypress(self, key_event.code, key_event.modifiers);}
                            Mode::View => {keybind::handle_view_mode_keypress(self, key_event.code, key_event.modifiers);}
                            Mode::Goto => {keybind::handle_goto_mode_keypress(self, key_event.code, key_event.modifiers);}
                            Mode::Find => {keybind::handle_find_mode_keypress(self, key_event.code, key_event.modifiers);}
                            Mode::Command => {keybind::handle_command_mode_keypress(self, key_event.code, key_event.modifiers);}
                            Mode::Error(_) => {keybind::handle_error_mode_keypress(self, key_event.code, key_event.modifiers);}
                            Mode::Warning(_) => {
                                //unhandled keybinds in warning mode fall through to insert mode //TODO: do the same for suggestions mode(not impled yet)
                                keybind::handle_warning_mode_keypress(self, key_event.code, key_event.modifiers);
                            }
                            Mode::Notify(_) => {
                                //unhandled keybinds in notify mode fall through to insert mode //TODO: do the same for suggestions mode(not impled yet)
                                keybind::handle_notify_mode_keypress(self, key_event.code, key_event.modifiers);
                            }
                            Mode::Info(_) => {
                                //unhandled keybinds in info mode fall through to insert mode //TODO: do the same for suggestions mode(not impled yet)
                                keybind::handle_info_mode_keypress(self, key_event.code, key_event.modifiers);
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
                    event::Event::Paste(_) => {self.no_op_event();}
                }

                Ok(())
            }
            Err(e) => Err(format!("{e}"))
        }
    }

    pub fn buffer_display_area(&self) -> DisplayArea{
        DisplayArea::new(
            self.buffer_horizontal_start, 
            self.buffer_vertical_start, 
            self.ui.document_viewport.document_widget.rect.width as usize, 
            self.ui.document_viewport.document_widget.rect.height as usize
        )
    }
    pub fn text_box_display_area(&self) -> DisplayArea{
        DisplayArea::new(
            self.ui.util_bar.utility_widget.text_box.display_area_horizontal_start, 
            self.ui.util_bar.utility_widget.text_box.display_area_vertical_start, 
            self.ui.util_bar.utility_widget.rect.width as usize, 
            self.ui.util_bar.utility_widget.rect.height as usize
        )
    }

    pub fn mode(&self) -> Mode{
        self.mode_stack.top().clone()
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
            Mode::Object | 
            Mode::View | 
            Mode::Error(_) | 
            Mode::Warning(_) | 
            Mode::Notify(_) | 
            Mode::Info(_) | 
            Mode::AddSurround => {/* do nothing */}
            Mode::Insert => {self.handle_notification(INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}
        }

        //remove current mode from stack
        if self.mode_stack.pop().is_err(){
            self.handle_notification(SAME_STATE_DISPLAY_MODE, SAME_STATE);
        }

        //handle exit behavior
        if update_layouts_and_document{
            //self.ui.update_layouts(&self.mode());
            //crate::ui::update_layouts(self);
            self.update_layouts();
            self.update_ui_data_document();
        }
        if clear_util_bar_text{
            self.ui.util_bar.utility_widget.text_box.clear();
            self.update_ui_data_util_bar();
        }
        if clear_saved_selections{
            //self.ui.util_bar.utility_widget.preserved_selections = None;
            self.preserved_selections = None;
        }

        //does this belong here, or in ui.rs?...
        self.update_ui_data_mode();
    }
    pub fn mode_push(&mut self, to_mode: Mode){
        if self.mode() == to_mode{/*do nothing*/}   //don't push mode to stack because we are already there
        else{
            //set any mode specific entry behavior
            let mut save_selections = false;
            let mut update_util_bar = false;
            let mut update_layouts_and_document = false;
            match to_mode{
                Mode::Find | Mode::Split => {
                    save_selections = true;
                    if !self.ui.status_bar.show{ // potential fix for status bar bug in todo.rs
                        update_util_bar = true;
                        update_layouts_and_document = true;
                    }
                }
                Mode::Command | Mode::Goto => {
                    if !self.ui.status_bar.show{ // potential fix for status bar bug in todo.rs
                        update_util_bar = true;
                        update_layouts_and_document = true;
                    }
                }
                Mode::Object | 
                Mode::Insert | 
                Mode::View | 
                Mode::Error(_) | 
                Mode::Warning(_) | 
                Mode::Notify(_) | 
                Mode::Info(_) | 
                Mode::AddSurround => {/* do nothing */}
            }

            //add mode to top of stack
            self.mode_stack.push(to_mode);

            //handle entry behavior
            if save_selections{
                //self.ui.util_bar.utility_widget.preserved_selections = Some(self.selections.clone());
                self.preserved_selections = Some(self.selections.clone());
            }
            if update_layouts_and_document{
                //self.ui.update_layouts(&self.mode());
                //crate::ui::update_layouts(self);
                self.update_layouts();
                self.update_ui_data_document();
            }
            if update_util_bar{
                self.update_ui_data_util_bar();
            }

            //does this belong here, or in ui.rs?...
            self.update_ui_data_mode();
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
        let buffer = &self.buffer;
        
        self.ui.document_viewport.document_widget.text = self.buffer_display_area().text(buffer);
        self.ui.document_viewport.line_number_widget.text = self.buffer_display_area().line_numbers(buffer);
        self.update_ui_data_selections();
        //TODO?: this may be better to have in the main loop, in case the file is modified underneath us while the buffer is open...
        if self.buffer.is_modified(){
            self.ui.status_bar.modified_widget.show = true;
            self.ui.status_bar.modified_widget.text = "[Modified]".to_string();
        }else{
            self.ui.status_bar.modified_widget.show = false;
            self.ui.status_bar.modified_widget.text = String::new();
        }
    }
    /// Set only data related to selections in document viewport UI.
    pub fn update_ui_data_selections(&mut self){
        let buffer = &self.buffer;
        let selections = &self.selections;
        
        self.ui.document_viewport.highlighter.primary_cursor = self.buffer_display_area().primary_cursor_position(buffer, selections, self.config.semantics.clone());
        self.ui.document_viewport.highlighter.cursors = self.buffer_display_area().cursor_positions(buffer, selections, self.config.semantics.clone());
        self.ui.document_viewport.highlighter.selections = self.buffer_display_area().selections(selections, buffer);
        self.ui.status_bar.selections_widget.text = format!("selections: {}/{}", selections.primary_selection_index + 1, selections.count());
        let cursor_position = selections.primary().selection_to_selection2d(buffer, self.config.semantics.clone()).head().clone();
        self.ui.status_bar.cursor_position_widget.text = format!("cursor: {}:{}", cursor_position.y + 1, cursor_position.x + 1)
    }
    pub fn update_ui_data_mode(&mut self){
        //does this belong here, or in ui.rs?...
        self.ui.status_bar.mode_widget.text = match self.mode(){
            Mode::AddSurround => format!("AddSurround: {:#?}", self.mode_stack.len()),
            Mode::Command => format!("Command: {:#?}", self.mode_stack.len()),
            Mode::Error(_) => format!("Error: {:#?}", self.mode_stack.len()),
            Mode::Find => format!("Find: {:#?}", self.mode_stack.len()),
            Mode::Goto => format!("Goto: {:#?}", self.mode_stack.len()),
            Mode::Info(_) => format!("Info: {:#?}", self.mode_stack.len()),
            Mode::Insert => format!("Insert: {:#?}", self.mode_stack.len()),
            Mode::Notify(_) => format!("Notify: {:#?}", self.mode_stack.len()),
            Mode::Object => format!("Object: {:#?}", self.mode_stack.len()),
            Mode::Split => format!("Split: {:#?}", self.mode_stack.len()),
            Mode::View => format!("View: {:#?}", self.mode_stack.len()),
            Mode::Warning(_) => format!("Warning: {:#?}", self.mode_stack.len()),
        };
    }
    pub fn update_ui_data_util_bar(&mut self){
        let text_box = &self.ui.util_bar.utility_widget.text_box;
        let text_box_display_area = self.text_box_display_area();
        if text_box_display_area.should_scroll(&text_box.selection, &text_box.buffer, self.config.semantics.clone()){
            let DisplayArea{horizontal_start, vertical_start, width: _width, height: _height} = text_box_display_area.scroll_following_cursor(&text_box.selection, &text_box.buffer, self.config.semantics.clone());
            self.ui.util_bar.utility_widget.text_box.display_area_horizontal_start = horizontal_start;
            self.ui.util_bar.utility_widget.text_box.display_area_vertical_start = vertical_start;
        }//else{/*keep current view*/}

        let text_box = &self.ui.util_bar.utility_widget.text_box;
        let text_box_display_area = self.text_box_display_area();
        let selections = Selections::new(
            vec![text_box.selection.clone()], 0, &text_box.buffer, self.config.semantics.clone()
        );
        self.ui.util_bar.highlighter.selection = text_box_display_area.selections(&selections, &text_box.buffer).first().cloned();
        self.ui.util_bar.highlighter.cursor = text_box_display_area.primary_cursor_position(&text_box.buffer, &selections, self.config.semantics.clone());
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
        let buffer = &self.buffer;
        if self.buffer_display_area().should_scroll(cursor_to_follow, buffer, self.config.semantics.clone()){
            let DisplayArea{horizontal_start, vertical_start, width: _width, height: _height} = self.buffer_display_area().scroll_following_cursor(cursor_to_follow, buffer, self.config.semantics.clone());
            self.buffer_horizontal_start = horizontal_start;
            self.buffer_vertical_start = vertical_start;
            scroll_response_fn(self);
        }else{
            non_scroll_response_fn(self);
        }
    }

    //TODO: think of a better name for this...
    pub fn handle_notification(&mut self, display_mode: DisplayMode, message: &'static str){
        match display_mode{
            DisplayMode::Error => {self.mode_push(Mode::Error(message.to_string()))}
            DisplayMode::Warning => {self.mode_push(Mode::Warning(message.to_string()));}
            DisplayMode::Notify => {self.mode_push(Mode::Notify(message.to_string()));}
            DisplayMode::Info => {self.mode_push(Mode::Info(message.to_string()));}
            DisplayMode::Ignore => {/* do nothing */}
        }
    }

    pub fn no_op_keypress(&mut self){
        self.handle_notification(UNHANDLED_KEYPRESS_DISPLAY_MODE, UNHANDLED_KEYPRESS);
    }
    pub fn no_op_event(&mut self){
        self.handle_notification(UNHANDLED_EVENT_DISPLAY_MODE, UNHANDLED_EVENT);
    }
    pub fn resize(&mut self, x: u16, y: u16){
        self.ui.set_terminal_size(x, y);
        //self.ui.update_layouts(&self.mode());
        //crate::ui::update_layouts(self);
        self.update_layouts();
        self.update_ui_data_util_bar(); //TODO: can this be called later in fn impl?
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
            self.selection_action(&SelectionAction::ClearNonPrimarySelections, 1);
        }
        else if self.selections.primary().is_extended(){
            self.selection_action(&SelectionAction::CollapseSelectionToCursor, 1);
        }
        else{
            self.handle_notification(SAME_STATE_DISPLAY_MODE, SAME_STATE);
        }
    }

    pub fn quit(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        if self.buffer.is_modified(){
            self.handle_notification(DisplayMode::Error, FILE_MODIFIED);
        }
        else{self.should_quit = true;}
    }
    pub fn quit_ignoring_changes(&mut self){
        assert!(self.mode() == Mode::Error(FILE_MODIFIED.to_string()) || self.mode() == Mode::Command);
        self.should_quit = true;
    }

    pub fn save(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        if self.buffer.file_path.is_none(){self.handle_notification(DisplayMode::Error, "cannot save unnamed buffer");}
        else if self.buffer.read_only{self.handle_notification(READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER);}
        else{
            match crate::utilities::save::application_impl(self){
                Ok(()) => {self.update_ui_data_document();}
                Err(_) => {     //this could maybe benefit from passing the io error up to this fn...
                    self.handle_notification(FILE_SAVE_FAILED_DISPLAY_MODE, FILE_SAVE_FAILED);
                }
            }
        }
    }
    fn handle_application_error(&mut self, e: ApplicationError){
        //let this_file = std::panic::Location::caller().file();  //actually, these should prob be assigned in calling fn, and passed in, so that error location is the caller and not always here...
        //let line_number = std::panic::Location::caller().line();
        match e{
            ApplicationError::ReadOnlyBuffer => {self.handle_notification(READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER);}
            ApplicationError::InvalidInput => {self.handle_notification(INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}
            ApplicationError::SelectionAtDocBounds |
            ApplicationError::NoChangesToUndo |
            ApplicationError::NoChangesToRedo => {self.handle_notification(SAME_STATE_DISPLAY_MODE, SAME_STATE);}
            ApplicationError::SelectionsError(s) => {
                match s{
                    SelectionsError::ResultsInSameState |
                    SelectionsError::CannotAddSelectionAbove |
                    SelectionsError::CannotAddSelectionBelow => {self.handle_notification(SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                    SelectionsError::MultipleSelections => {self.handle_notification(MULTIPLE_SELECTIONS_DISPLAY_MODE, MULTIPLE_SELECTIONS);}
                    SelectionsError::SingleSelection => {self.handle_notification(SINGLE_SELECTION_DISPLAY_MODE, SINGLE_SELECTION);}
                    SelectionsError::NoSearchMatches |
                    SelectionsError::SpansMultipleLines => self.handle_notification(SPANS_MULTIPLE_LINES_DISPLAY_MODE, SPANS_MULTIPLE_LINES),
                }
            }
        }
    }
    pub fn copy(&mut self){
        assert!(self.mode() == Mode::Insert);
        match crate::utilities::copy::application_impl(self){
            Ok(()) => {
                self.handle_notification(COPIED_TEXT_DISPLAY_MODE, COPIED_TEXT);
                self.update_ui_data_document(); //TODO: is this really needed for something?...
            }
            Err(e) => {
                self.handle_application_error(e);
            }
        }
    }
    //TODO: impl application_impl here, instead of in utilities...
    pub fn edit_action(&mut self, action: &EditAction){
        use crate::utilities::*;
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::AddSurround);
        
        if self.buffer.read_only{self.handle_notification(READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER);}
        else{
            //let len = self.buffer.len_lines();
            let result = match action{
                EditAction::InsertChar(c) => insert_string::application_impl(self, &c.to_string(), self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                EditAction::InsertNewline => insert_string::application_impl(self, "\n", self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                EditAction::InsertTab => insert_string::application_impl(self, "\t", self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                EditAction::Delete => delete::application_impl(self, self.config.semantics.clone()),
                EditAction::Backspace => backspace::application_impl(self, self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                EditAction::Cut => cut::application_impl(self, self.config.semantics.clone()),
                EditAction::Paste => paste::application_impl(self, self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                EditAction::Undo => undo::application_impl(self, self.config.semantics.clone()),   // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
                EditAction::Redo => redo::application_impl(self, self.config.semantics.clone()),
                EditAction::AddSurround(l, t) => add_surrounding_pair::application_impl(self, *l, *t, self.config.semantics.clone()),
            };
            match result{
                Ok(()) => {
                    if self.mode() != Mode::Insert{self.mode_pop();}
                    
                    self.checked_scroll_and_update(
                        &self.selections.primary().clone(), 
                        Application::update_ui_data_document, 
                        Application::update_ui_data_document
                    );
                    //if len != self.buffer.len_lines(){self.ui.document_viewport.document_widget.doc_length = self.buffer.len_lines();}

                    // check if any selection is outside of view
                    let mut selection_out_of_view = false;
                    for selection in self.selections.iter(){
                        if self.buffer_display_area().should_scroll(selection, &self.buffer, self.config.semantics.clone()){
                            selection_out_of_view = true;
                        }
                    }
                    if selection_out_of_view{
                        self.handle_notification(EDIT_ACTION_DISPLAY_MODE, EDIT_ACTION_OUT_OF_VIEW);
                    }
                    //
                }
                Err(e) => {
                    self.handle_application_error(e);
                }
            }
        }
    }

    pub fn selection_action(&mut self, action: &SelectionAction, count: usize){
        use crate::utilities::*;
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Object);
        enum SelectionToFollow{Primary,First,Last}
        
        let (result, selection_to_follow) = match action{
            SelectionAction::MoveCursorUp => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), move_cursor_up::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorDown => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), move_cursor_down::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorLeft => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), move_cursor_left::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorRight => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), move_cursor_right::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorWordBoundaryForward => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), move_cursor_word_boundary_forward::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorWordBoundaryBackward => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), move_cursor_word_boundary_backward::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorLineEnd => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), move_cursor_line_end::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorHome => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), move_cursor_home::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorBufferStart => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), move_cursor_buffer_start::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorBufferEnd => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), move_cursor_buffer_end::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorPageUp => {(self.selections.move_selection(count, &self.buffer, Some(&self.buffer_display_area()), self.config.semantics.clone(), move_cursor_page_up::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::MoveCursorPageDown => {(self.selections.move_selection(count, &self.buffer, Some(&self.buffer_display_area()), self.config.semantics.clone(), move_cursor_page_down::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ExtendSelectionUp => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), extend_selection_up::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ExtendSelectionDown => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), extend_selection_down::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ExtendSelectionLeft => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), extend_selection_left::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ExtendSelectionRight => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), extend_selection_right::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ExtendSelectionWordBoundaryBackward => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), extend_selection_word_boundary_backward::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ExtendSelectionWordBoundaryForward => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), extend_selection_word_boundary_forward::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ExtendSelectionLineEnd => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), extend_selection_line_end::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ExtendSelectionHome => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), extend_selection_home::selection_impl), SelectionToFollow::Primary)}
                //SelectionAction::ExtendSelectionDocumentStart => {self.document.extend_selection_document_start(CURSOR_SEMANTICS)}
                //SelectionAction::ExtendSelectionDocumentEnd => {self.document.extend_selection_document_end(CURSOR_SEMANTICS)}
                //SelectionAction::ExtendSelectionPageUp => {self.document.extend_selection_page_up(CURSOR_SEMANTICS)}
                //SelectionAction::ExtendSelectionPageDown => {self.document.extend_selection_page_down(CURSOR_SEMANTICS)}
            SelectionAction::SelectLine => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), select_line::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::SelectAll => {(self.selections.move_cursor_clearing_non_primary(&self.buffer, self.config.semantics.clone(), select_all::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::CollapseSelectionToAnchor => {(self.selections.move_cursor_non_overlapping(&self.buffer, self.config.semantics.clone(), collapse_selections_to_anchor::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::CollapseSelectionToCursor => {(self.selections.move_cursor_non_overlapping(&self.buffer, self.config.semantics.clone(), collapse_selections_to_cursor::selection_impl), SelectionToFollow::Primary)}
            SelectionAction::ClearNonPrimarySelections => {(clear_non_primary_selections::selections_impl(&self.selections), SelectionToFollow::Primary)}
            SelectionAction::AddSelectionAbove => {(add_selection_above::selections_impl(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::First)}
            SelectionAction::AddSelectionBelow => {(add_selection_below::selections_impl(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::Last)}
            SelectionAction::RemovePrimarySelection => {(remove_primary_selection::selections_impl(&self.selections), SelectionToFollow::Primary)}
            SelectionAction::IncrementPrimarySelection => {(increment_primary_selection::selections_impl(&self.selections), SelectionToFollow::Primary)}
            SelectionAction::DecrementPrimarySelection => {(decrement_primary_selection::selections_impl(&self.selections), SelectionToFollow::Primary)}
            SelectionAction::Surround => {(surround::selections_impl(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::Primary)},
            SelectionAction::FlipDirection => {(self.selections.move_cursor_non_overlapping(&self.buffer, self.config.semantics.clone(), flip_direction::selection_impl), SelectionToFollow::Primary)},
        
                //These may technically be distinct from the other selection actions, because they could be called from object mode, and would need to pop the mode stack after calling...
                //TODO: SelectionAction::Word => {self.document.word()}
                //TODO: SelectionAction::Sentence => {self.document.sentence()}
                //TODO: SelectionAction::Paragraph => {self.document.paragraph()}
            SelectionAction::SurroundingPair => {(nearest_surrounding_pair::selections_impl(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::Primary)}  //TODO: rename SurroundingBracketPair
                //TODO: SelectionAction::QuotePair => {self.document.nearest_quote_pair()}                      //TODO: rename SurroundingQuotePair
                //TODO: SelectionAction::ExclusiveSurroundingPair => {self.document.exclusive_surrounding_pair()}
                //TODO: SelectionAction::InclusiveSurroundingPair => {self.document.inclusive_surrounding_pair()}
        };

        match result{
            Ok(new_selections) => {
                self.selections = new_selections;

                //maybe.    so far, only needed for selection actions called from object mode
                if self.mode() != Mode::Insert{self.mode_pop();}
                //
                
                let primary_selection = self.selections.primary().clone();
                let first_selection = self.selections.first().clone();
                let last_selection = self.selections.last().clone();
                self.checked_scroll_and_update(
                    match selection_to_follow{
                        SelectionToFollow::Primary => &primary_selection,
                        SelectionToFollow::First => &first_selection,
                        SelectionToFollow::Last => &last_selection,
                    },
                    Application::update_ui_data_document, 
                    Application::update_ui_data_selections
                );

                // check if any selection is outside of view
                let mut selection_out_of_view = false;
                for selection in self.selections.iter(){
                    if self.buffer_display_area().should_scroll(selection, &self.buffer, self.config.semantics.clone()){
                        selection_out_of_view = true;
                    }
                }
                if selection_out_of_view{
                    self.handle_notification(SELECTION_ACTION_DISPLAY_MODE, SELECTION_ACTION_OUT_OF_VIEW);
                }
                //
            }
            Err(e) => {self.handle_application_error(ApplicationError::SelectionsError(e));}
        }
    }

    pub fn view_action(&mut self, action: &ViewAction){ //TODO: make sure this can still be called from insert, so users can assign a direct keybind if desired
        use crate::utilities::*;
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::View);

        let mut should_exit = false;
        let result = match action{
            ViewAction::CenterVerticallyAroundCursor => {
                should_exit = true;
                center_view_vertically_around_cursor::view_impl(&self.buffer_display_area(), self.selections.primary(), &self.buffer, self.config.semantics.clone())
            }
            ViewAction::ScrollUp => {scroll_view_up::view_impl(&self.buffer_display_area(), self.config.view_scroll_amount)}
            ViewAction::ScrollDown => {scroll_view_down::view_impl(&self.buffer_display_area(), self.config.view_scroll_amount, &self.buffer)}
            ViewAction::ScrollLeft => {scroll_view_left::view_impl(&self.buffer_display_area(), self.config.view_scroll_amount)}
            ViewAction::ScrollRight => {scroll_view_right::view_impl(&self.buffer_display_area(), self.config.view_scroll_amount, &self.buffer)}
        };

        match result{
            Ok(view) => {
                let DisplayArea{horizontal_start, vertical_start, width: _width, height: _height} = view;
                self.buffer_horizontal_start = horizontal_start;
                self.buffer_vertical_start = vertical_start;

                self.update_ui_data_document();
                if self.mode() != Mode::Insert && should_exit{self.mode_pop();}
            }
            Err(e) => {
                match e{
                    DisplayAreaError::InvalidInput => {self.handle_application_error(ApplicationError::InvalidInput);}
                    DisplayAreaError::ResultsInSameState => {self.handle_application_error(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
                }
            }
        }
    }

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
            Mode::Error(_) |
            Mode::Warning(_) |
            Mode::Notify(_) |
            Mode::Info(_) |
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

    //TODO: entering current line number should be a same state warning, not invalid input error
    //TODO: entering a very large number switches util bar text color to the valid state instead of the error state for some reason
    pub fn goto_mode_accept(&mut self){
        assert!(self.mode() == Mode::Goto);
        let mut show_warning = false;
        if let Ok(line_number) = self.ui.util_bar.utility_widget.text_box.buffer./*inner.*/to_string().parse::<usize>(){
            if line_number == 0{show_warning = true;}   //we have no line number 0, so this is invalid
            else{
                let line_number = line_number.saturating_sub(1);    // make line number 0 based for interfacing correctly with backend impl
                match crate::utilities::move_to_line_number::application_impl(self, line_number, self.config.semantics.clone()){
                    Ok(()) => {self.checked_scroll_and_update(&self.selections.primary().clone(), Application::update_ui_data_selections, Application::update_ui_data_selections);}  //TODO: pretty sure one of these should be update_ui_data_document
                    Err(_) => {show_warning = true;}    //TODO: match error and handle
                }
            }
        }else{show_warning = true;}
        if show_warning{self.handle_notification(INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}
        else{self.mode_pop()}
    }
    //TODO: add go to matching surrounding char(curly, square, paren, single quote, double quote, etc)
    //TODO: can this be accomplished in edit_core instead?...
    pub fn goto_mode_selection_action(&mut self, action: &SelectionAction){  //TODO: this is pretty slow when user enters a large number into util text box
        assert!(self.mode() == Mode::Goto);
        if let Ok(count) = self.ui.util_bar.utility_widget.text_box.buffer./*inner.*/to_string().parse::<usize>(){
            self.mode_pop();
            assert!(self.mode() == Mode::Insert);
            self.selection_action(action, count);
        }else{self.handle_notification(INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}  //TODO: this may benefit from a specific error, maybe stating why the input is invalid...empty/non number input string...//"action requires non-empty, numeric input string"
        //also, this doesn't work with goto_mode_text_validity_check
    }
    pub fn goto_mode_text_validity_check(&mut self){
        assert!(self.mode() == Mode::Goto);
        // run text validity check
        let mut is_numeric = true;
        for grapheme in self.ui.util_bar.utility_widget.text_box.buffer.inner.chars(){ // .graphemes(true)?
            if !grapheme.is_ascii_digit(){is_numeric = false;}
        }
        let exceeds_doc_length = match self.ui.util_bar.utility_widget.text_box.buffer./*inner.*/to_string().parse::<usize>(){
            Ok(line_number) => {line_number > self.buffer.len_lines()}
            Err(_) => false //TODO: very large numeric input strings aren't parseable to usize, thus set exceeds_doc_length to false...
        };
        self.ui.util_bar.utility_widget.text_box.text_is_valid = is_numeric && !exceeds_doc_length;
    }

    pub fn restore_selections_and_exit(&mut self){
        self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
        self.selections = self.preserved_selections.clone().unwrap();//self.ui.util_bar.utility_widget.preserved_selections.clone().unwrap();    //shouldn't be called unless this value is Some()
        self.checked_scroll_and_update(&self.selections.primary().clone(), Application::update_ui_data_document, Application::update_ui_data_selections);
        self.mode_pop();
    }
    fn incremental_search(&mut self){   //this def doesn't work correctly with utf-8 yet
        let preserved_selections = self.preserved_selections.clone();//self.ui.util_bar.utility_widget.preserved_selections.clone();
        let search_text = self.ui.util_bar.utility_widget.text_box.buffer./*inner.*/to_string().clone();
        match preserved_selections{
            Some(preserved_selections) => {
                match crate::utilities::incremental_search_in_selection::application_impl(self, &search_text, &preserved_selections, self.config.semantics.clone()){
                    Ok(()) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = true;}
                    Err(_) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = false;}
                }
            }
            None => {/* maybe error?... */unreachable!()}
        }
    }
    fn incremental_split(&mut self){
        let preserved_selections = self.preserved_selections.clone();//self.ui.util_bar.utility_widget.preserved_selections.clone();
        let split_text = self.ui.util_bar.utility_widget.text_box.buffer./*inner.*/to_string().clone();
        match preserved_selections{
            Some(preserved_selections) => {
                match crate::utilities::incremental_split_in_selection::application_impl(self, &split_text, &preserved_selections, self.config.semantics.clone()){
                    Ok(()) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = true;}
                    Err(_) => {self.ui.util_bar.utility_widget.text_box.text_is_valid = false;}
                }
            }
            None => {/* maybe error?... */unreachable!()}
        }
    }

    pub fn toggle_line_numbers(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        self.ui.document_viewport.line_number_widget.show = !self.ui.document_viewport.line_number_widget.show;
                
        //self.ui.update_layouts(&self.mode());
        //crate::ui::update_layouts(self);
        self.update_layouts();
        self.update_ui_data_document();
    }
    pub fn toggle_status_bar(&mut self){
        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
        self.ui.status_bar.show = !self.ui.status_bar.show;
                
        //self.ui.update_layouts(&self.mode());
        //crate::ui::update_layouts(self);
        self.update_layouts();
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
        match self.ui.util_bar.utility_widget.text_box.buffer./*inner.*/to_string().as_str(){
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
            //write buffer contents to file //should this optionally take a filepath to save to? then we don't need to implement save as    //would have to split util bar text on ' ' into separate args
            "write" | "w" => {
                self.save();
            }
            _ => {warn = true;}
        }
        if warn{self.handle_notification(COMMAND_PARSE_FAILED_DISPLAY_MODE, COMMAND_PARSE_FAILED);}
        else{self.mode_pop()}
    }


    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_replace(
        buffer: &mut Buffer, 
        replacement_text: &str, 
        selection: &mut Selection, 
        semantics: CursorSemantics
    ) -> Change{ //TODO: Error if replacement_text is empty(or if selection empty? is this possible?)
        let old_selection = selection.clone();
        let delete_change = Application::apply_delete(buffer, selection, semantics.clone());
        let replaced_text = if let Operation::Insert{inserted_text} = delete_change.inverse(){inserted_text}else{unreachable!();};  // inverse of delete change should always be insert
        let _ = Application::apply_insert(buffer, replacement_text, selection, semantics.clone());   //intentionally discard returned Change

        Change::new(
            Operation::Replace{replacement_text: replacement_text.to_string()}, 
            old_selection, 
            selection.clone(), 
            Operation::Replace{replacement_text: replaced_text}
        )
    }
    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_insert(
        buffer: &mut Buffer, 
        string: &str, 
        selection: &mut Selection, 
        semantics: CursorSemantics
    ) -> Change{    //TODO: Error if string is empty
        let old_selection = selection.clone();
        buffer.insert(selection.cursor(buffer, semantics.clone()), string);
        for _ in 0..string.len(){
            if let Ok(new_selection) = crate::utilities::move_cursor_right::selection_impl(selection, 1, buffer, None, semantics.clone()){
                *selection = new_selection;
            }
        }

        Change::new(
            Operation::Insert{inserted_text: string.to_string()}, 
            old_selection, 
            selection.clone(), 
            Operation::Delete
        )
    }
    // TODO: test. should test rope is edited correctly and selection is moved correctly, not necessarily the returned change. behavior, not impl
    pub fn apply_delete(
        buffer: &mut Buffer, 
        selection: &mut Selection, 
        semantics: CursorSemantics
    ) -> Change{  //TODO: Error if cursor and anchor at end of text
        use std::cmp::Ordering;
        
        let old_selection = selection.clone();
        let original_text = buffer.clone();

        let (start, end, new_cursor) = match selection.cursor(buffer, semantics.clone()).cmp(&selection.anchor()){
            Ordering::Less => {(selection.head(), selection.anchor(), selection.cursor(buffer, semantics.clone()))}
            Ordering::Greater => {
                match semantics{
                    CursorSemantics::Bar => {(selection.anchor(), selection.head(), selection.anchor())}
                    CursorSemantics::Block => {
                        if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){(selection.anchor(), selection.cursor(buffer, semantics.clone()), selection.anchor())}
                        else{(selection.anchor(), selection.head(), selection.anchor())}
                    }
                }
            }
            Ordering::Equal => {
                if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){ //do nothing    //or preferrably return error   //could have condition check in calling fn
                    return Change::new(
                        Operation::Delete, 
                        old_selection, 
                        selection.clone(), 
                        Operation::Insert{inserted_text: String::new()}
                    );   //change suggested by clippy lint
                }
                
                match semantics.clone(){
                    CursorSemantics::Bar => {(selection.head(), selection.head().saturating_add(1), selection.anchor())}
                    CursorSemantics::Block => {(selection.anchor(), selection.head(), selection.anchor())}
                }
            }
        };

        let change_text = original_text.slice(start, end);
        buffer.remove(start..end);
        if let Ok(new_selection) = selection.put_cursor(new_cursor, &original_text, crate::selection::Movement::Move, semantics, true){
            *selection = new_selection;
        }

        Change::new(
            Operation::Delete, 
            old_selection, 
            selection.clone(), 
            Operation::Insert{inserted_text: change_text.to_string()}
        )
    }
}

fn render_widget(text: String, area: Rect, alignment: Alignment, bold: bool, background_color: Color, foreground_color: Color, frame: &mut Frame<'_>){
    frame.render_widget(
        if bold{
            Paragraph::new(text)
            .style(
                Style::default()
                    .bg(background_color)
                    .fg(foreground_color)
            )
            .alignment(alignment)
            .bold()
        }else{
            Paragraph::new(text)
            .style(
                Style::default()
                    .bg(background_color)
                    .fg(foreground_color)
            )
            .alignment(alignment)
        }, 
        area
    );
}
fn render_popup(text: String, title: String, area: Rect, background_color: Color, foreground_color: Color, frame: &mut Frame<'_>){
    frame.render_widget(
        Paragraph::new(text)
            .block(ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::all())
                .title(title))
            .style(
                Style::new()
                    .bg(background_color)
                    .fg(foreground_color)
            ),
        area
    );
}

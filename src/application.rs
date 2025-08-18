//goal features:
//  edit is a file server. serves readable files for exposure of internal data, and writeable files for integration with external utilities(for example, integrate with plan9 style plumber via plumbing rules)
//  controllable through a custom command language, with command extensibility, executable from within the text buffer
//  enable commands to be associated with event hooks, to enable synchronous integration(file read/write would be for asynchronous?...)
//  enable ui customization with custom layouts/widgets + content
//  modal agnostic
//  reduce the set of built in features to only those that cannot be built using external utilities or from combining existing commands
//  always give user some visual response to input(this may not be possible with only a text buffer ui widget)


//simplest impl would be just a text buffer, and the ability to eval commands from within buffer, and assign commands to keybinds


//integrate with external plumb utility
    //%sh{cat /mnt/edit/$pid/selection/content | plumb}     //send content of primary selection to plumb utility    //response behavior is determined by predefined plumb rules
    //%sh{cat $selection | plumb}


//TODO: figure out how to launch another terminal session, start another edit session, and pass it text via stdin
    //let _ = std::process::Command::new("alacritty")
    //                    .args(&["-e", "bash", "-c", "<program to run>"])
    //                    .spawn()
    //                    .expect("Failed to launch Alacritty");

//TODO: if error message displayed in scratch buffer, select filename and error location, trigger plumb command(acme mouse right click).
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

// could certain actions be accomplished with a built in command?
// add-highlighter  //line/column/rope offset/range based?...    //pre virtual text or post?     //only single cell or vec of cells?...
    //let user decide which coordinate scheme to use, but it should always resolve to a rope offset
    //this will be pre virtual text, and accounted for in edit core/server before conversion to display coordinates
    //all buffer highlights should be listed in offset coordinates and served at buffer/highlights
    //visible buffer highlights should be listed in display coordinates and served at display_area/highlights
    //highlighters may need a group parameter, so that a group can be cleared, if needed, and the rest left alone
// add-virtual-text //line/column or rope offset?...    //should this have an associated color for highlighting?...
// add-command <name> <command>     //add-comand open_terminal "alacritty &"    //if command ends in '&', spawn instead of status/output
// add-keybind <scope> <mode> <keybind> <command>   //does this need to be handled separately from other commands, since keybinds are a frontend only concept?...
// add-fold     //line or selection_range     //pre virtual text/highlighting or post?
// menu     //for contextual popup menus
// prompt   //for util text box
// command aliases could be accomplished by defining a new command  //add-command <alias> <aliased-command>
// search <regex>   //non interactive search within selections  //could interactive search be accomplished by integrating external utility instead of being built in?...
// split <regex>    //non interactive split within selections   //could interactive split be accomplished by integrating external utility instead of being built in?...

// client could tokenize command string, and maybe have separate client specific commands, which the client would resolve before sending
// command tokens to server
// ex: add-keybind
// ex: %sh{idk_bar -display !val{mode}}     //mode is expanded in client, shell command expanded and called in server
// menu and prompt would have to be intercepted by the client side parser as well
// maybe we wouldn't need separate %val{} and !val{}. client can just determine what is it's responsibility to expand, and do so...

//maybe add Mode::User(mode_name) to let user add custom modes...

//would it be possible to allow layout(and contents) to be defined using commands?
// add-widget text --start (0, 0) --dimensions (10, 20) --content %val{line_numbers} --bg Rgb(0, 0, 0) --fg(255, 255, 255) --name line_number_widget
// add-sub-widget scrollbar --parent file_text_buffer (+whatever other info would be needed)
//how could this be made to change dynamically(like during resize)

//Config could be populated from an rc(run command) file, instead of deserializing from some config format
//the rc file would contain a list of whitespace separated commands to run(from top to bottom) at startup, to set up necessary data structures
//such as default keybinds:
//bind <mode> <keybind> <command>   //command could be a built-in, or one defined earlier in the rc...
    //it may be a good idea to allow for comments in the rc file. how could this be accomplished
//or to set up options:
//set-option <option> <value>   //set-option use_full_file_path false


//9p file system
// command file     //on write, edit performs commands
//%sh{ cat date > $buffer }      //write date to buffer file, which inserts date's output at every selection(replacing selection content if extended)
// write to /mnt/edit/id/buffer would insert/replace at/in place of the currently selected text for all selections
// write to /mnt/edit/id/selections/selection_num would insert/replace at/in place of the selected text for that selection only
// /mnt/edit/id/selections/leading/selection_num, /mnt/edit/id/selections/primary, /mnt/edit/id/selections/trailing/selection_num if using alternate selections impl
// selection_num files would only be served if that selection number exists


//notification modes could be set through a command
//echo --error <message>
//echo --warn <message>
//echo --notify <message>
//echo --info <message>     //same as echo <message>

use std::path::PathBuf;
use crossterm::event;
use ratatui::{
    prelude::*,
    widgets::*
};
use crate::{
    config::*,
    mode::Mode,
    mode_stack::StackMember,
    action::{Action, EditorAction, SelectionAction, EditAction, ViewAction, UtilAction},
    range::Range,
    buffer::Buffer,
    display_area::{DisplayArea, DisplayAreaError},
    mode_stack::ModeStack,
    selection::{Selection, CursorSemantics},
    selections::{Selections, SelectionsError},
    ui::{UserInterface, util_bar::*},
    history::ChangeSet,
};

//TODO: maybe Mode, ModeStack, and Action + related actually do belong in this file...

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
    config: Config,
    pub buffer: Buffer, 
    preserved_selections: Option<Selections>, 
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
            ui: UserInterface::new(terminal_rect, &config.keybinds),
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
                            CursorSemantics::Block => Range::new(0, buffer.next_grapheme_char_index(0))
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
            clipboard: String::new(),
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
        }//else{    //this should already be default
        //    self.ui.status_bar.read_only_widget.show = false;
        //    self.ui.status_bar.read_only_widget.text = String::new();
        //}
        
        if self.buffer.file_path.is_some(){
            self.ui.status_bar.file_name_widget.show = true;
            if self.config.use_full_file_path{
                self.ui.status_bar.file_name_widget.text = self.buffer.file_path().unwrap_or_default();
            }else{
                self.ui.status_bar.file_name_widget.text = self.buffer.file_name().unwrap_or_default();
            }
        }//else{    //this should already be default
        //    self.ui.status_bar.file_name_widget.show = false;
        //    self.ui.status_bar.file_name_widget.text = String::new();
        //}

        self.update_ui_data_mode();
        self.update_layouts();
        self.checked_scroll_and_update(
            &self.selections.primary().clone(), 
            Application::update_ui_data_document, 
            Application::update_ui_data_document
        );
        self.update_ui_data_util_bar(); //needed for util bar cursor to render the first time it is triggered   //TODO?: does this belong before update_layouts()?...
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
    pub fn mode(&self) -> Mode{self.mode_stack.top().mode.clone()}

    /// Set all data related to document viewport UI.
    fn update_ui_data_document(&mut self){
        self.ui.document_viewport.document_widget.text = self.buffer_display_area().text(&self.buffer);
        self.ui.document_viewport.line_number_widget.text = self.buffer_display_area().line_numbers(&self.buffer);
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
    fn update_ui_data_selections(&mut self){
        self.ui.document_viewport.highlighter.primary_cursor = self.buffer_display_area().primary_cursor_position(&self.buffer, &self.selections, self.config.semantics.clone());
        self.ui.document_viewport.highlighter.cursors = self.buffer_display_area().cursor_positions(&self.buffer, &self.selections, self.config.semantics.clone());
        self.ui.document_viewport.highlighter.selections = self.buffer_display_area().selections(&self.selections, &self.buffer);
        self.ui.status_bar.selections_widget.text = format!("selections: {}/{}", &self.selections.primary_selection_index + 1, &self.selections.count());
        let cursor_position = &self.selections.primary().selection_to_selection2d(&self.buffer, self.config.semantics.clone()).head().clone();
        self.ui.status_bar.cursor_position_widget.text = format!("cursor: {}:{}", cursor_position.y + 1, cursor_position.x + 1)
    }
    fn update_ui_data_mode(&mut self){self.ui.status_bar.mode_widget.text = format!("{:?}: {:#?}", self.mode(), self.mode_stack.len());}
    /// set data related to util bar UI.
    fn update_ui_data_util_bar(&mut self){
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
    fn checked_scroll_and_update<F, A>(&mut self, cursor_to_follow: &Selection, scroll_response_fn: F, non_scroll_response_fn: A)
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

    pub fn update_layouts(&mut self){   //-> Result<(), String>{ //to handle terminal.size() error
        fn layout_terminal(app: &Application, terminal_size: Rect) -> std::rc::Rc<[Rect]>{       //TODO: maybe rename layout_terminal_vertical_ui_components
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
                        Constraint::Length(if app.ui.status_bar.show{1}else{0}),

                        //[2]
                        // util(goto/find/command) bar rect height
                        Constraint::Length(
                            match &app.mode(){
                                Mode::Error | 
                                Mode::Warning | 
                                Mode::Notify | 
                                Mode::Info | 
                                Mode::Command | 
                                Mode::Find | 
                                Mode::Goto | 
                                Mode::Split => 1,
                            
                                Mode::Object |
                                Mode::Insert |
                                Mode::View |
                                Mode::AddSurround => if app.ui.status_bar.show{1}else{0}
                            }
                        )
                    ]
                )
                .split(terminal_size)
        }
        fn layout_buffer_viewport(app: &Application, rect: Rect) -> std::rc::Rc<[Rect]>{
            fn count_digits(mut n: usize) -> u16{
                if n == 0{return 1;}
                let mut count = 0;
                while n > 0{
                    count += 1;
                    n /= 10;
                }
                count
            }
            // layout of document + line num rect
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    vec![
                        //[0]
                        // line number rect width
                        Constraint::Length(
                            if app.ui.document_viewport.line_number_widget.show{
                                /*crate::ui::*/count_digits(app.buffer.len_lines())
                            }else{0}
                        ),
                    
                        //[1]
                        // line number right padding
                        Constraint::Length(
                            if app.ui.document_viewport.line_number_widget.show{
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
        fn layout_status_bar(app: &Application, rect: Rect) -> std::rc::Rc<[Rect]>{
            // layout of status bar rect (modified_indicator/file_name/cursor_position)
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    vec![
                        //[0]
                        // read_only widget
                        Constraint::Max(
                            app.ui.status_bar.read_only_widget.text.len() as u16
                        ),
                    
                        //[1]
                        // padding_1
                        Constraint::Max(
                            if app.buffer.read_only{
                                1
                            }else{0}
                        ),

                        //[2]
                        // file_name widget
                        Constraint::Max(
                            app.ui.status_bar.file_name_widget.text.len() as u16
                        ),
                    
                        //[3]
                        // padding_2
                        Constraint::Max(
                            if app.buffer.is_modified(){
                                1
                            }else{0}
                        ),

                        //[4]
                        // modified widget
                        Constraint::Max(
                            app.ui.status_bar.modified_widget.text.len() as u16
                        ),

                        //[5]
                        // selections widget
                        Constraint::Min(0),     //or set selections widget to Max, and surround with 2 padding widgets set to Min(0)?...idk if that will work the same?...

                        //[6]
                        // cursor position indicator width
                        Constraint::Max(
                            app.ui.status_bar.cursor_position_widget.text.len() as u16
                        ),
                    
                        //[7]
                        // padding_3
                        Constraint::Max(1),
                    
                        //[8]
                        // mode widget
                        Constraint::Max(
                            app.ui.status_bar.mode_widget.text.len() as u16
                        ),
                    ]
                )
                .split(rect)
        }
        fn layout_util_bar(app: &Application, rect: Rect) -> std::rc::Rc<[Rect]>{
            use crate::ui::util_bar::{GOTO_PROMPT, FIND_PROMPT, SPLIT_PROMPT, COMMAND_PROMPT};
            // layout of util rect (goto/find/command/save as)
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    vec![
                        //[0]
                        // util bar prompt width
                        Constraint::Length(
                            match app.mode(){
                                Mode::Goto => GOTO_PROMPT.len() as u16,
                                Mode::Find => FIND_PROMPT.len() as u16,
                                Mode::Split => SPLIT_PROMPT.len() as u16,
                                Mode::Command => COMMAND_PROMPT.len() as u16,
                                Mode::Error
                                | Mode::Warning
                                | Mode::Notify
                                | Mode::Info
                                | Mode::Insert
                                | Mode::Object
                                | Mode::View 
                                | Mode::AddSurround => 0
                            }
                        ),

                        //[1]
                        // util bar rect width
                        Constraint::Length(
                            match app.mode(){
                                Mode::Insert
                                | Mode::Object
                                | Mode::View
                                | Mode::Error
                                | Mode::Warning
                                | Mode::Notify
                                | Mode::Info
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
        fn sized_centered_rect(x: u16, y: u16, r: Rect) -> Rect{
            let padding_height = r.height.saturating_sub(y) / 2;
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(padding_height.saturating_sub(1)),
                        Constraint::Length(y),
                        Constraint::Length(padding_height.saturating_sub(1)),
                    ]
                    .as_ref()
                )
                .split(r);
            
            let padding_width = r.width.saturating_sub(x) / 2;
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(padding_width.saturating_sub(1)),
                        Constraint::Length(x),
                        Constraint::Length(padding_width.saturating_sub(1)),
                    ]
                )
                .split(popup_layout[1])[1]
        }
        //TODO: terminal.size() should be called here, instead of storing terminal_size
        // this will require all calling functions to return a Result. handle changes to action fns before doing this...
        //let terminal_size = match terminal.size(){
        //    Ok(size) => size,
        //    Err(e) => return Err(format!("{}", e))
        //};
        //let terminal_size = Rect::new(0, 0, terminal_size.width, terminal_size.height);
    
        let terminal_rect = layout_terminal(self, self.ui.terminal_size);
        let document_viewport_rect = layout_buffer_viewport(self, terminal_rect[0]);
        let status_bar_rect = layout_status_bar(self, terminal_rect[1]);
        let util_rect = layout_util_bar(self, terminal_rect[2]);
    
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
            
        self.ui.popups.goto.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.goto.widest_element_len, self.ui.popups.goto.num_elements, self.ui.terminal_size);
        self.ui.popups.command.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.command.widest_element_len, self.ui.popups.command.num_elements, self.ui.terminal_size);
        self.ui.popups.find.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.find.widest_element_len, self.ui.popups.find.num_elements, self.ui.terminal_size);
        self.ui.popups.split.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.split.widest_element_len, self.ui.popups.split.num_elements, self.ui.terminal_size);
        self.ui.popups.error.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.error.widest_element_len, self.ui.popups.error.num_elements, self.ui.terminal_size);
        self.ui.popups.modified_error.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.modified_error.widest_element_len, self.ui.popups.modified_error.num_elements, self.ui.terminal_size);
        self.ui.popups.warning.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.warning.widest_element_len, self.ui.popups.warning.num_elements, self.ui.terminal_size);
        self.ui.popups.notify.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.notify.widest_element_len, self.ui.popups.notify.num_elements, self.ui.terminal_size);
        self.ui.popups.info.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.info.widest_element_len, self.ui.popups.info.num_elements, self.ui.terminal_size);
        self.ui.popups.view.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.view.widest_element_len, self.ui.popups.view.num_elements, self.ui.terminal_size);
        self.ui.popups.object.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.object.widest_element_len, self.ui.popups.object.num_elements, self.ui.terminal_size);
        self.ui.popups.add_surround.rect = /*crate::ui::*/sized_centered_rect(self.ui.popups.add_surround.widest_element_len, self.ui.popups.add_surround.num_elements, self.ui.terminal_size);
    }
    
    //TODO: error/warning/notify/info mode(or maybe all modes) popup titles should include the mode stack count, so repeated modes can be seen if status bar hidden
    pub fn render(&self, terminal: &mut Terminal<impl Backend>) -> Result<(), String>{
        fn generate_widget(text: &str, alignment: Alignment, bold: bool, background_color: Color, foreground_color: Color) -> ratatui::widgets::Paragraph<'_>{
            if bold{Paragraph::new(text).style(Style::default().bg(background_color).fg(foreground_color)).alignment(alignment).bold()}
            else{Paragraph::new(text).style(Style::default().bg(background_color).fg(foreground_color)).alignment(alignment)}
        }
        fn render_buffer_highlights(app: &Application, buf: &mut ratatui::prelude::Buffer){
            // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
            let area = app.ui.document_viewport.document_widget.rect;
            // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
            
            if app.config.show_cursor_column{
                for y in area.top()..area.height{
                    if let Some(primary_cursor_position) = &app.ui.document_viewport.highlighter.primary_cursor{
                        if let Some(cell) = buf.cell_mut((area.left() + primary_cursor_position.x as u16, y)){
                            cell.set_style(Style::default().bg(CURSOR_COLUMN_BACKGROUND_COLOR).fg(CURSOR_COLUMN_FOREGROUND_COLOR));
                        }
                    }
                }
            }
            if app.config.show_cursor_line{
                for x in area.left()..(area.width + area.left()){
                    if let Some(primary_cursor_position) = &app.ui.document_viewport.highlighter.primary_cursor{
                        if let Some(cell) = buf.cell_mut((x, area.top() + primary_cursor_position.y as u16)){
                            cell.set_style(Style::default().bg(CURSOR_LINE_BACKGROUND_COLOR).fg(CURSOR_LINE_FOREGROUND_COLOR));
                        }
                    }
                }
            }
        
            if !app.ui.document_viewport.highlighter.selections.is_empty(){
                for selection in &app.ui.document_viewport.highlighter.selections{
                    if selection.head().x - selection.anchor().x == 0{continue;}    //should this use start and end instead?
                    for col in selection.anchor().x../*=*/selection.head().x{
                        let x_pos = area.left() + (col as u16);
                        let y_pos = area.top() + (selection.head().y as u16);
                    
                        if let Some(cell) = buf.cell_mut((x_pos, y_pos)){
                            cell.set_style(Style::default().bg(SELECTION_BACKGROUND_COLOR).fg(SELECTION_FOREGROUND_COLOR));
                        }
                    }
                }
            }
        
            //render cursors for all selections
            if !app.ui.document_viewport.highlighter.cursors.is_empty(){
                for cursor in &app.ui.document_viewport.highlighter.cursors{
                    if let Some(cell) = buf.cell_mut((area.left() + (cursor.x as u16), area.top() + (cursor.y as u16))){
                        cell.set_style(Style::default().bg(CURSOR_BACKGROUND_COLOR).fg(CURSOR_FOREGROUND_COLOR));
                    }
                }
            }
        
            // render primary cursor
            if let Some(cursor) = &app.ui.document_viewport.highlighter.primary_cursor{
                if let Some(cell) = buf.cell_mut((area.left() + (cursor.x as u16), area.top() + (cursor.y as u16))){
                    cell.set_style(Style::default().bg(PRIMARY_CURSOR_BACKGROUND_COLOR).fg(PRIMARY_CURSOR_FOREGROUND_COLOR));
                }
            }
        
            //debug //this can help ensure we are using the correct Rect
            //if let Some(cell) = buf.cell_mut((area.left(), area.top())){
            //    cell.set_style(Style::default().bg(ratatui::style::Color::Yellow));
            //}
        }
        fn render_util_bar_highlights(app: &Application, buf: &mut ratatui::prelude::Buffer){
            let area = app.ui.util_bar.utility_widget.rect;
            
            //render selection
            if let Some(selection) = &app.ui.util_bar.highlighter.selection{
                if selection.head().x - selection.anchor().x > 0{   //if selection extended
                    for col in selection.anchor().x..selection.head().x{
                        let x_pos = area.left() + (col as u16);
                        let y_pos = area.top() + (selection.head().y as u16);
                        //assert_eq!(0, y_pos, "util bar text should be guaranteed to be one line");    //this seems to be causing issues when moving from end of line...        
                        if let Some(cell) = buf.cell_mut((x_pos, y_pos)){
                            cell.set_style(Style::default().bg(SELECTION_BACKGROUND_COLOR).fg(SELECTION_FOREGROUND_COLOR));
                        }
                    }
                }
            }
        
            // render cursor
            if let Some(cursor) = &app.ui.util_bar.highlighter.cursor{
                assert_eq!(0, cursor.y, "util bar text should be guaranteed to be one line");
                if let Some(cell) = buf.cell_mut((area.left() + (cursor.x as u16), area.top() + (cursor.y as u16))){
                    cell.set_style(Style::default().bg(PRIMARY_CURSOR_BACKGROUND_COLOR).fg(PRIMARY_CURSOR_FOREGROUND_COLOR));
                }
            }
        
            //debug //this can help ensure we are using the correct Rect
            //if let Some(cell) = buf.cell_mut((area.left(), area.top())){
            //    cell.set_style(Style::default().bg(ratatui::style::Color::Yellow));
            //}
        }
        fn generate_popup<'a>(text: &'a str, title: &'a str, background_color: Color, foreground_color: Color) -> Paragraph<'a>{
            Paragraph::new(text)
                .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::all()).title(title))
                .style(Style::new().bg(background_color).fg(foreground_color))
        }
        match terminal.draw(
            |frame| {
                // always render
                frame.render_widget(generate_widget(&self.ui.document_viewport.document_widget.text, Alignment::Left, false, DOCUMENT_BACKGROUND_COLOR, DOCUMENT_FOREGROUND_COLOR), self.ui.document_viewport.document_widget.rect);
                render_buffer_highlights(self, frame.buffer_mut());
                
                // conditionally render
                if self.ui.document_viewport.line_number_widget.show{
                    frame.render_widget(generate_widget(&self.ui.document_viewport.line_number_widget.text, Alignment::Right, false, LINE_NUMBER_BACKGROUND_COLOR, LINE_NUMBER_FOREGROUND_COLOR), self.ui.document_viewport.line_number_widget.rect);
                    frame.render_widget(generate_widget("", Alignment::Center, false, LINE_NUMBER_BACKGROUND_COLOR, LINE_NUMBER_BACKGROUND_COLOR), self.ui.document_viewport.padding.rect);
                }
                if self.ui.status_bar.show{
                    //instead of read_only_widget.text, we could do: if app.buffer.read_only{"ReadOnly"}else{String::new()}
                    frame.render_widget(generate_widget(&self.ui.status_bar.read_only_widget.text, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, READ_ONLY_WIDGET_FOREGROUND_COLOR), self.ui.status_bar.read_only_widget.rect);
                    frame.render_widget(generate_widget("", Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, Color::Red), self.ui.status_bar.padding_1.rect);
                    frame.render_widget(generate_widget(&self.ui.status_bar.file_name_widget.text, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, FILE_NAME_WIDGET_FOREGROUND_COLOR), self.ui.status_bar.file_name_widget.rect);
                    frame.render_widget(generate_widget("", Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, Color::Red), self.ui.status_bar.padding_2.rect);
                    frame.render_widget(generate_widget(&self.ui.status_bar.modified_widget.text, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, MODIFIED_WIDGET_FOREGROUND_COLOR), self.ui.status_bar.modified_widget.rect);
                    frame.render_widget(generate_widget(&self.ui.status_bar.selections_widget.text, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, SELECTIONS_WIDGET_FOREGROUND_COLOR), self.ui.status_bar.selections_widget.rect);
                    frame.render_widget(generate_widget(&self.ui.status_bar.cursor_position_widget.text, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, CURSOR_POSITION_WIDGET_FOREGROUND_COLOR), self.ui.status_bar.cursor_position_widget.rect);
                    frame.render_widget(generate_widget("", Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, Color::Red), self.ui.status_bar.padding_3.rect);
                    frame.render_widget(generate_widget(&self.ui.status_bar.mode_widget.text, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, MODE_WIDGET_FOREGROUND_COLOR), self.ui.status_bar.mode_widget.rect);
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
                        frame.render_widget(generate_widget(GOTO_PROMPT, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.prompt.rect);
                        frame.render_widget(generate_widget(&self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, if self.ui.util_bar.utility_widget.text_box.text_is_valid{UTIL_BAR_FOREGROUND_COLOR}else{UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR}), self.ui.util_bar.utility_widget.rect);
                        render_util_bar_highlights(self, frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.goto.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.goto.text, &self.ui.popups.goto.title, Color::Black, Color::Yellow), self.ui.popups.goto.rect);
                        }
                    }
                    Mode::Command => {
                        frame.render_widget(generate_widget(COMMAND_PROMPT, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.prompt.rect);
                        frame.render_widget(generate_widget(&self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        render_util_bar_highlights(self, frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.command.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.command.text, &self.ui.popups.command.title, Color::Black, Color::Yellow), self.ui.popups.command.rect);
                        }
                    }
                    Mode::Find => {
                        frame.render_widget(generate_widget(FIND_PROMPT, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.prompt.rect);
                        frame.render_widget(generate_widget(&self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, if self.ui.util_bar.utility_widget.text_box.text_is_valid{UTIL_BAR_FOREGROUND_COLOR}else{UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR}), self.ui.util_bar.utility_widget.rect);
                        render_util_bar_highlights(self, frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.find.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.find.text, &self.ui.popups.find.title, Color::Black, Color::Yellow), self.ui.popups.find.rect);
                        }
                    }
                    Mode::Split => {
                        frame.render_widget(generate_widget(SPLIT_PROMPT, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.prompt.rect);
                        frame.render_widget(generate_widget(&self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, if self.ui.util_bar.utility_widget.text_box.text_is_valid{UTIL_BAR_FOREGROUND_COLOR}else{UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR}), self.ui.util_bar.utility_widget.rect);
                        render_util_bar_highlights(self, frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.split.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.split.text, &self.ui.popups.split.title, Color::Black, Color::Yellow), self.ui.popups.split.rect);
                        }
                    }
                    Mode::Error => {
                        frame.render_widget(generate_widget(&self.mode_stack.top().text.expect("text being Some should be guaranteed in Error mode"), Alignment::Center, true, ERROR_BACKGROUND_COLOR, ERROR_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        //if &self.mode_stack.top().text.expect("text being Some should be guaranteed in Error mode") == FILE_MODIFIED{
                        if self.mode_stack.top().text == Some(FILE_MODIFIED.to_string()){
                            if SHOW_CONTEXTUAL_KEYBINDS{
                                frame.render_widget(ratatui::widgets::Clear, self.ui.popups.modified_error.rect);
                                frame.render_widget(generate_popup(&self.ui.popups.modified_error.text, &self.ui.popups.modified_error.title, Color::Black, Color::Yellow), self.ui.popups.modified_error.rect);
                            }
                        }
                        else{
                            if SHOW_CONTEXTUAL_KEYBINDS{
                                frame.render_widget(ratatui::widgets::Clear, self.ui.popups.error.rect);
                                frame.render_widget(generate_popup(&self.ui.popups.error.text, &self.ui.popups.error.title, Color::Black, Color::Yellow), self.ui.popups.error.rect);
                            }
                        }
                    }
                    Mode::Warning => {
                        frame.render_widget(generate_widget(&self.mode_stack.top().text.expect("text being Some should be guaranteed in Warning mode"), Alignment::Center, true, WARNING_BACKGROUND_COLOR, WARNING_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.warning.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.warning.text, &self.ui.popups.warning.title, Color::Black, Color::Yellow), self.ui.popups.warning.rect);
                        }
                    }
                    Mode::Notify => {
                        frame.render_widget(generate_widget(&self.mode_stack.top().text.expect("text being Some should be guaranteed in Notify mode"), Alignment::Center, true, NOTIFY_BACKGROUND_COLOR, NOTIFY_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.notify.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.notify.text, &self.ui.popups.notify.title, Color::Black, Color::Yellow), self.ui.popups.notify.rect);
                        }
                    }
                    Mode::Info => {
                        frame.render_widget(generate_widget(&self.mode_stack.top().text.expect("text being Some should be guaranteed in Info mode"), Alignment::Center, true, INFO_BACKGROUND_COLOR, INFO_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.info.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.info.text, &self.ui.popups.info.title, Color::Black, Color::Yellow), self.ui.popups.info.rect);
                        }
                    }
                    Mode::View => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.view.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.view.text, &self.ui.popups.view.title, Color::Black, Color::Yellow), self.ui.popups.view.rect);
                        }
                    }
                    Mode::Object => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.object.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.object.text, &self.ui.popups.object.title, Color::Black, Color::Yellow), self.ui.popups.object.rect);
                        }
                    }
                    Mode::AddSurround => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.add_surround.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.add_surround.text, &self.ui.popups.add_surround.title, Color::Black, Color::Yellow), self.ui.popups.add_surround.rect);
                        }
                    }
                }
            }
        ){
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{e}"))
        }
    }

    fn handle_event(&mut self) -> Result<(), String>{
        // This is needed because generic keypresses cannot be inserted into keybind hashmap
        fn handle_char_insert(mode: Mode, key_event: crossterm::event::KeyEvent) -> Action{
            use crossterm::event::{KeyCode, KeyModifiers};
            match (key_event.code, key_event.modifiers){
                (KeyCode::Char(c), KeyModifiers::SHIFT) if matches!(mode, Mode::Insert)                             => Action::EditAction(EditAction::InsertChar(c)),
                (KeyCode::Char(c), KeyModifiers::SHIFT) if matches!(mode, Mode::Find | Mode::Split | Mode::Command) => Action::UtilAction(UtilAction::InsertChar(c)),
                (KeyCode::Char(c), KeyModifiers::NONE)  if matches!(mode, Mode::Insert)                             => Action::EditAction(EditAction::InsertChar(c)),
                (KeyCode::Char(c), KeyModifiers::NONE)  if matches!(mode, Mode::Goto) && c.is_numeric()             => Action::UtilAction(UtilAction::InsertChar(c)),
                (KeyCode::Char(c), KeyModifiers::NONE)  if matches!(mode, Mode::Find | Mode::Split | Mode::Command) => Action::UtilAction(UtilAction::InsertChar(c)),
                _ => Action::EditorAction(EditorAction::NoOpKeypress)
            }
        }
        match event::read(){
            Ok(event) => {
                self.action(
                    match event{
                        event::Event::Key(key_event) => {
                            match self.config.keybinds.get(&(self.mode(), key_event)).cloned(){
                                Some(action) => action,
                                None => {
                                    match self.mode(){
                                        //unbound key presses for these modes fall through to insert mode
                                        Mode::Warning | Mode::Notify | Mode::Info => {
                                            //spoofing our mode as insert, to handle fall through
                                            match self.config.keybinds.get(&(Mode::Insert, key_event)).cloned(){
                                                //some actions may need to be modified to pop until insert mode, because we aren't actually in insert yet
                                                //the application will panic/misbehave if action does not handle this...
                                                Some(insert_action) => insert_action,
                                                None => handle_char_insert(Mode::Insert, key_event)
                                            }
                                        }
                                        _ => handle_char_insert(self.mode(), key_event)
                                    }
                                }
                            }
                        },
                        event::Event::Mouse(_mouse_event) => Action::EditorAction(EditorAction::NoOpEvent),
                        event::Event::Resize(width, height) => Action::EditorAction(EditorAction::Resize(width, height)),
                        event::Event::FocusLost => Action::EditorAction(EditorAction::NoOpEvent), //maybe quit displaying cursor(s)/selection(s)?...
                        event::Event::FocusGained => Action::EditorAction(EditorAction::NoOpEvent),   //display cursor(s)/selection(s)?...
                        event::Event::Paste(_) => Action::EditorAction(EditorAction::NoOpEvent)
                    }
                );
                Ok(())
            }
            Err(e) => Err(format!("{e}"))
        }
    }

    pub fn action(&mut self, action: Action){
        //impl helper functions here to manage scope of exposure
        //fn esc_handle(app: &mut Application){
        //    assert!(app.mode() == Mode::Insert);
        //    //TODO: if lsp suggestions displaying(currently unimplemented), exit that display   //lsp suggestions could be a separate mode with keybind fallthrough to insert...
        //    /*else */if app.selections.count() > 1{app.action(Action::SelectionAction(SelectionAction::ClearNonPrimarySelections, 1));}
        //    else if app.selections.primary().is_extended(){app.action(Action::SelectionAction(SelectionAction::CollapseSelectionToCursor, 1));}
        //    else{handle_notification(app, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
        //}
        fn pop_to_insert(app: &mut Application){    //helper function for insert fallthrough
            //pop until insert mode, because of fallthrough
            while app.mode() != Mode::Insert{app.action(Action::EditorAction(EditorAction::ModePop));}
        }
        fn handle_message(app: &mut Application, display_mode: DisplayMode, message: &/*'static */str){
            match display_mode{
                DisplayMode::Error => app.action(Action::EditorAction(EditorAction::ModePush(StackMember{mode: Mode::Error, text: Some(message.to_string())}))),
                DisplayMode::Warning => app.action(Action::EditorAction(EditorAction::ModePush(StackMember{mode: Mode::Warning, text: Some(message.to_string())}))),
                DisplayMode::Notify => app.action(Action::EditorAction(EditorAction::ModePush(StackMember{mode: Mode::Notify, text: Some(message.to_string())}))),
                DisplayMode::Info => app.action(Action::EditorAction(EditorAction::ModePush(StackMember{mode: Mode::Info, text: Some(message.to_string())}))),
                DisplayMode::Ignore => {/* do nothing */}
            }
        }
        fn handle_application_error(app: &mut Application, e: ApplicationError){
            //let this_file = std::panic::Location::caller().file();  //actually, these should prob be assigned in calling fn, and passed in, so that error location is the caller and not always here...
            //let line_number = std::panic::Location::caller().line();
            match e{
                ApplicationError::ReadOnlyBuffer => {handle_message(app, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER);}
                ApplicationError::InvalidInput => {handle_message(app, INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}
                ApplicationError::SelectionAtDocBounds |
                ApplicationError::NoChangesToUndo |
                ApplicationError::NoChangesToRedo => {handle_message(app, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                ApplicationError::SelectionsError(s) => {
                    match s{
                        SelectionsError::ResultsInSameState |
                        SelectionsError::CannotAddSelectionAbove |
                        SelectionsError::CannotAddSelectionBelow => {handle_message(app, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                        SelectionsError::MultipleSelections => {handle_message(app, MULTIPLE_SELECTIONS_DISPLAY_MODE, MULTIPLE_SELECTIONS);}
                        SelectionsError::SingleSelection => {handle_message(app, SINGLE_SELECTION_DISPLAY_MODE, SINGLE_SELECTION);}
                        SelectionsError::NoSearchMatches |
                        SelectionsError::SpansMultipleLines => handle_message(app, SPANS_MULTIPLE_LINES_DISPLAY_MODE, SPANS_MULTIPLE_LINES),
                    }
                }
            }
        }
        //at the extreme, i think every action could end up being a command
        //in that sense, the editor is just a command parser, with command specific response behavior
        fn parse_command(app: &mut Application, command_string: String) -> Result<(), ()>{
            //TODO: split commands on any '\n' or ';' outside of quote strings
            //TODO: loop through each separate command string, and perform
            //TODO: consider how to handle a failed command in a list of commands. should we just error on first failed command?...
            let command: Vec<&str> = command_string.split(' ').collect();
            match /*command_string.as_str()*/*command.first().unwrap(){    //maybe command.get(0)...
                //TODO: this should be a user defined command instead of built in
                //push-command <name_string> <command_string>
                //push-command "open new alacritty window" %sh{alacritty msg create-window}
                //push-command term "open new alacritty window"  //this is effectively an alias
                //push-command t term                            //this is effectively an alias
                //pop-command <name_string> //remove a user defined command
                //UserCommand{
                //  name: String,
                //  aliases: Option<Vec<String>>,
                //  command_body: String
                //}
                //and would have to match on user defined commands
                //user defined commands may need to be quoted "if spaces are used"...
                "term" | "t" => app.action(Action::EditorAction(EditorAction::OpenNewTerminalWindow)),
                "toggle_line_numbers" | "ln" => app.action(Action::EditorAction(EditorAction::ToggleLineNumbers)),  //these will prob end up using set-option command...
                "toggle_status_bar" | "sb" => app.action(Action::EditorAction(EditorAction::ToggleStatusBar)),      //these will prob end up using set-option command...
                "quit" | "q" => app.action(Action::EditorAction(EditorAction::Quit)),
                "quit!" | "q!" => app.action(Action::EditorAction(EditorAction::QuitIgnoringChanges)),
                //write buffer contents to file //should this optionally take a filepath to save to? then we don't need to implement save as    //would have to split util bar text on ' ' into separate args
                "write" | "w" => app.action(Action::EditorAction(EditorAction::Save)),
                "search" => {
                    match crate::utilities::incremental_search_in_selection::selections_impl(
                        &app.selections, 
                        command.get(1).unwrap(),
                        &app.buffer, 
                        app.config.semantics.clone()
                    ){
                        Ok(new_selections) => {
                            app.selections = new_selections;
                            app.checked_scroll_and_update(
                                &app.selections.primary().clone(), 
                                Application::update_ui_data_document, 
                                Application::update_ui_data_selections
                            );
                        }
                        Err(_) => {
                            //self.selections = selections_before_search.clone();
                            handle_message(app, DisplayMode::Error, "not matching regex");
                        }
                    }
                }
                //"\"idk some shit\"" => handle_message(app, DisplayMode::Error, "idk some shit"),  //commands with whitespace can be handled this way
                //add_command <command_name> <command>
                //"add_command" => {/* match positional args and, if command name available, insert into command list */}
                //remove_command <command_name>
                //add_keybind <mode> <keybind> <command>
                "add_keybind" => {
                    let mode = Mode::Insert;    //get mode from positional args
                    let keycode = crossterm::event::KeyCode::Char('w'); //get mode from positional args
                    let modifiers = crossterm::event::KeyModifiers::CONTROL;    //get mode from positional args
                    let key_event = crossterm::event::KeyEvent::new(keycode, modifiers);
                    let _command = "idk some shit".to_string();  //get mode from positional args
                    if app.config.keybinds.contains_key(&(mode, key_event)){
                        //error
                    }else{
                        //app.config.keybinds.insert((mode, key_event), Action::EditorAction(EditorAction::EvalCommand(command)));
                        handle_message(app, DisplayMode::Info, "keybind added");
                    }
                }
                //remove_keybind <keybind>
                //add_option <name> <type>
                //remove_option <name>
                //set_option <name> <value>
                //add_hook
                //remove hook
                _ => {
                    //TODO: check if command_string matches user defined command
                    return Err(());
                }
            }
            Ok(())
        }
        match action{
            Action::EditorAction(editor_action) => {
                match editor_action{
                    EditorAction::ModePop => {
                        fn perform_shared_behavior(app: &mut Application){
                            //update layouts and document
                            app.update_layouts();
                            app.update_ui_data_document();
                            // clear util bar text
                            app.ui.util_bar.utility_widget.text_box.clear();
                            app.update_ui_data_util_bar();
                        }
                        //remove current mode from stack
                        if let Ok(StackMember{mode: popped_mode, text: popped_text}) = self.mode_stack.pop(){
                            if popped_mode == self.mode() && popped_text == self.mode_stack.top().text{
                                //continue popping until self.mode() is something else (this would clean up repeated error messages/etc.)
                                self.action(Action::EditorAction(EditorAction::ModePop));
                                return; //only the final ModePop should run any follow up code
                            }
                            match popped_mode{
                                Mode::Command | Mode::Goto => {perform_shared_behavior(self);}
                                Mode::Find | Mode::Split => {
                                    perform_shared_behavior(self);
                                    self.preserved_selections = None;   //clear saved selections
                                }
                                Mode::Object | Mode::View | Mode::Error | Mode::Warning | Mode::Notify | Mode::Info | Mode::AddSurround | 
                                Mode::Insert => {/* do nothing */}  //could early return here, if we didn't need to update mode data
                            }
                            //does this belong here, or in ui.rs?...    //by calling here, we only perform this calculation as needed, not on every editor run cycle
                            self.update_ui_data_mode();
                        }else{handle_message(self, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                    }
                    EditorAction::ModePush(stack_member) => {
                        fn perform_shared_behavior(app: &mut Application){
                            //update layouts and document
                            app.update_layouts();
                            app.update_ui_data_document();
                            //update util bar
                            app.update_ui_data_util_bar();
                        }
                        let to_mode = stack_member.mode.clone();
                        match to_mode{
                            Mode::Find | Mode::Split => {
                                pop_to_insert(self);
                                self.mode_stack.push(stack_member);
                                self.preserved_selections = Some(self.selections.clone());  //save selections
                                if !self.ui.status_bar.show{ // potential fix for status bar bug in todo.rs
                                    perform_shared_behavior(self);
                                }
                            }
                            Mode::Command | Mode::Goto => {
                                pop_to_insert(self);
                                self.mode_stack.push(stack_member);
                                if !self.ui.status_bar.show{ // potential fix for status bar bug in todo.rs
                                    perform_shared_behavior(self);
                                }
                            }
                            Mode::Object | Mode::AddSurround | Mode::View => {
                                pop_to_insert(self);
                                self.mode_stack.push(stack_member);
                            }
                            Mode::Error | Mode::Warning | Mode::Notify | Mode::Info => {self.mode_stack.push(stack_member);}
                            Mode::Insert => {unreachable!()}    //should always pop to Insert, never push to Insert
                        }
                        //does this belong here, or in ui.rs?...    //by calling here, we only perform this calculation as needed, not on every editor run cycle
                        self.update_ui_data_mode();
                    }
                    EditorAction::Resize(width, height) => {
                        self.ui.set_terminal_size(width, height);
                        self.update_layouts();
                        self.update_ui_data_util_bar(); //TODO: can this be called later in fn impl?
                        // scrolling so cursor is in a reasonable place, and updating so any ui changes render correctly
                        self.checked_scroll_and_update(
                            &self.selections.primary().clone(),
                            Application::update_ui_data_document, 
                            Application::update_ui_data_document
                        );
                    }
                    EditorAction::NoOpKeypress => {handle_message(self, UNHANDLED_KEYPRESS_DISPLAY_MODE, UNHANDLED_KEYPRESS);}
                    EditorAction::NoOpEvent => {handle_message(self, UNHANDLED_EVENT_DISPLAY_MODE, UNHANDLED_EVENT);}
                    EditorAction::Quit => {
                        //possible modes are Insert and Command + any mode with fallthrough to insert
                        assert!(matches!(self.mode(), Mode::Insert | Mode::Command | Mode::Error | Mode::Warning | Mode::Notify | Mode::Info));
                        if self.buffer.is_modified(){
                            if self.mode() == Mode::Error && self.mode_stack.top().text.unwrap() == FILE_MODIFIED{
                                self.action(Action::EditorAction(EditorAction::QuitIgnoringChanges));
                            }
                            else{
                                handle_message(self, DisplayMode::Error, FILE_MODIFIED);
                            }
                        }
                        else{
                            if self.mode() == Mode::Error{self.action(Action::EditorAction(EditorAction::NoOpKeypress));}
                            else{self.should_quit = true;}
                        }
                    }
                    EditorAction::QuitIgnoringChanges => {
                        assert!(
                            self.mode() == Mode::Error && self.mode_stack.top().text.unwrap() == FILE_MODIFIED ||
                            self.mode() == Mode::Command ||
                            self.mode() == Mode::Insert
                        );
                        self.should_quit = true;
                    }
                    EditorAction::Save => {
                        //possible modes are Insert and Command + any mode with fallthrough to insert
                        assert!(matches!(self.mode(), Mode::Insert | Mode::Command | Mode::Warning | Mode::Notify | Mode::Info));
                        if self.buffer.file_path.is_none(){
                            pop_to_insert(self);
                            handle_message(self, DisplayMode::Error, "cannot save unnamed buffer");
                        }
                        else if self.buffer.read_only{
                            pop_to_insert(self);
                            handle_message(self, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER);
                        }
                        else{
                            match crate::utilities::save::application_impl(self){
                                Ok(()) => {
                                    pop_to_insert(self);
                                    self.update_ui_data_document();
                                }
                                //this could maybe benefit from passing the io error up to this fn...
                                Err(_) => {handle_message(self, FILE_SAVE_FAILED_DISPLAY_MODE, FILE_SAVE_FAILED);}
                            }
                        }
                    }
                    EditorAction::Copy => {
                        //possible modes are Insert + any mode with fallthrough to insert
                        assert!(matches!(self.mode(), Mode::Insert | Mode::Warning | Mode::Notify | Mode::Info));
                        match crate::utilities::copy::application_impl(self){
                            Ok(()) => {
                                pop_to_insert(self);
                                handle_message(self, COPIED_TEXT_DISPLAY_MODE, COPIED_TEXT);
                                self.update_ui_data_document(); //TODO: is this really needed for something?...
                            }
                            Err(e) => {handle_application_error(self, e);}
                        }
                    }
                    //TODO: remove this in favor of a user defined command
                    EditorAction::OpenNewTerminalWindow => {
                        assert!(matches!(self.mode(), Mode::Insert | Mode::Command | Mode::Warning | Mode::Notify | Mode::Info));
                        //if matches!(self.mode(), Mode::Warning | Mode::Notify | Mode::Info){pop_to_insert(self);}   //handle insert fallthrough
                        let result = std::process::Command::new("alacritty")     //TODO: have user define TERMINAL const in config.rs   //or check env vars for $TERM?
                            //.arg("msg")     // these extra commands just make new instances use the same backend(daemon?)
                            //.arg("create-window")
                            //.current_dir(std::env::current_dir().unwrap())    //not needed here, because term spawned here defaults to this directory, but good to know
                            .spawn();
                            //.expect("failed to spawn new terminal at current directory");
                        if let Err(e) = result{handle_message(self, DisplayMode::Error, &format!("{e}"));}
                    }
                    EditorAction::ToggleLineNumbers => {
                        //TODO: this may need to handle insert fallthrough
                        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
                        self.ui.document_viewport.line_number_widget.show = !self.ui.document_viewport.line_number_widget.show;
                        self.update_layouts();
                        self.update_ui_data_document();
                    }
                    EditorAction::ToggleStatusBar => {
                        //TODO: this may need to handle insert fallthrough
                        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
                        self.ui.status_bar.show = !self.ui.status_bar.show;
                        self.update_layouts();
                        self.update_ui_data_document();
                    }
                    //could become a command: eval_command %val{selection}
                    EditorAction::EvaluateSelectionAsCommand => {
                        if self.mode() != Mode::Insert{pop_to_insert(self);}    //handle insert fallthrough
                        //TODO: figure out best way to handle multiple selections...
                        if self.selections.count() > 1{
                            handle_application_error(self, ApplicationError::SelectionsError(SelectionsError::MultipleSelections));
                        }else{
                            //if parse_command(self, self.selections.primary().to_string(&self.buffer)).is_err(){
                            if Result::is_err(&parse_command(self, self.selections.primary().to_string(&self.buffer))){
                                handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, COMMAND_PARSE_FAILED);
                            }
                        }
                    }
                    //this, in combination with copy, is the keyboard centric version of plan9's acme's 2-1 mouse chording
                    //evaluate_command %val{clipboard}
                    EditorAction::EvaluateClipboardAsCommand => {
                        if self.mode() != Mode::Insert{pop_to_insert(self);}    //handle insert fallthrough
                        //if parse_command(self, self.clipboard.clone()).is_err(){
                        if Result::is_err(&parse_command(self, self.clipboard.clone())){
                            handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, COMMAND_PARSE_FAILED);
                        }
                    }
                }
            }
            Action::SelectionAction(selection_action, count) => {
                use crate::utilities::*;
                //possible modes are Insert and Object + any mode with fallthrough to insert
                assert!(matches!(self.mode(), Mode::Insert | Mode::Object | Mode::Warning | Mode::Notify | Mode::Info));
                enum SelectionToFollow{Primary,First,Last}

                let (result, selection_to_follow) = match selection_action{
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
                    SelectionAction::ExtendSelectionBufferStart => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), extend_selection_buffer_start::selection_impl), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionBufferEnd => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), extend_selection_buffer_end::selection_impl), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionPageUp => {(self.selections.move_selection(count, &self.buffer, Some(&self.buffer_display_area()), self.config.semantics.clone(), extend_selection_page_up::selection_impl), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionPageDown => {(self.selections.move_selection(count, &self.buffer, Some(&self.buffer_display_area()), self.config.semantics.clone(), extend_selection_page_down::selection_impl), SelectionToFollow::Primary)}                    
                    SelectionAction::SelectLine => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), select_line::selection_impl), SelectionToFollow::Primary)}
                    SelectionAction::SelectAll => {(self.selections.move_cursor_clearing_non_primary(&self.buffer, self.config.semantics.clone(), select_all::selection_impl), SelectionToFollow::Primary)}
                    //TODO: bug fix: if selection extended vertically(up/page up/maybe others), then collapsed to anchor, the resultant cursor is 1 grapheme right from where it should be
                    //TODO: this also happens if selection extended backwards horizontally
                    SelectionAction::CollapseSelectionToAnchor => {(self.selections.move_cursor_non_overlapping(&self.buffer, self.config.semantics.clone(), collapse_selections_to_anchor::selection_impl), SelectionToFollow::Primary)}
                    SelectionAction::CollapseSelectionToCursor => {(self.selections.move_cursor_non_overlapping(&self.buffer, self.config.semantics.clone(), collapse_selections_to_cursor::selection_impl), SelectionToFollow::Primary)}
                    SelectionAction::ClearNonPrimarySelections => {(clear_non_primary_selections::selections_impl(&self.selections), SelectionToFollow::Primary)}
                    SelectionAction::AddSelectionAbove => {(add_selection_above::selections_impl(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::First)}
                    SelectionAction::AddSelectionBelow => {(add_selection_below::selections_impl(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::Last)}
                    SelectionAction::RemovePrimarySelection => {(remove_primary_selection::selections_impl(&self.selections), SelectionToFollow::Primary)}
                    SelectionAction::IncrementPrimarySelection => {(increment_primary_selection::selections_impl(&self.selections), SelectionToFollow::Primary)}
                    SelectionAction::DecrementPrimarySelection => {(decrement_primary_selection::selections_impl(&self.selections), SelectionToFollow::Primary)}
                    SelectionAction::Surround => {(surround::selections_impl(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::Primary)},
                    //TODO: FlipDirection should update stored line position, so that a subsequent vertical move reflects the current cursor position
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
                    
                        pop_to_insert(self);

                        let primary_selection = &self.selections.primary().clone();
                        let first_selection = &self.selections.first().clone();
                        let last_selection = &self.selections.last().clone();
                        self.checked_scroll_and_update(
                            match selection_to_follow{
                                SelectionToFollow::Primary => primary_selection,
                                SelectionToFollow::First => first_selection,
                                SelectionToFollow::Last => last_selection,
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
                            handle_message(self, SELECTION_ACTION_DISPLAY_MODE, SELECTION_ACTION_OUT_OF_VIEW);
                        }
                        //
                    }
                    Err(e) => {handle_application_error(self, ApplicationError::SelectionsError(e));}
                }
            }
            Action::EditAction(edit_action) => {
                //TODO: impl application_impl here, instead of in utilities...
                use crate::utilities::*;
                //possible modes are Insert and AddSurround + any mode with fallthrough to insert
                assert!(matches!(self.mode(), Mode::Insert | Mode::AddSurround | Mode::Warning | Mode::Notify | Mode::Info));

                if self.buffer.read_only{handle_message(self, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER);}
                else{
                    let result = match edit_action{
                        EditAction::InsertChar(c) => insert_string::application_impl(self, &c.to_string(), self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::InsertNewline => insert_string::application_impl(self, "\n", self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::InsertTab => insert_string::application_impl(self, "\t", self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::Delete => delete::application_impl(self, self.config.semantics.clone()),
                        EditAction::Backspace => backspace::application_impl(self, self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::Cut => cut::application_impl(self, self.config.semantics.clone()),
                        EditAction::Paste => paste::application_impl(self, self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::Undo => undo::application_impl(self, self.config.semantics.clone()),   // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
                        EditAction::Redo => redo::application_impl(self, self.config.semantics.clone()),
                        EditAction::AddSurround(l, t) => add_surrounding_pair::application_impl(self, l, t, self.config.semantics.clone()),
                    };
                    match result{
                        Ok(()) => {
                            pop_to_insert(self);
                            self.checked_scroll_and_update(
                                &self.selections.primary().clone(), 
                                Application::update_ui_data_document, 
                                Application::update_ui_data_document
                            );
                            // check if any selection is outside of view
                            let mut selection_out_of_view = false;
                            for selection in self.selections.iter(){
                                if self.buffer_display_area().should_scroll(selection, &self.buffer, self.config.semantics.clone()){
                                    selection_out_of_view = true;
                                }
                            }
                            if selection_out_of_view{
                                handle_message(self, EDIT_ACTION_DISPLAY_MODE, EDIT_ACTION_OUT_OF_VIEW);
                            }
                            //
                        }
                        Err(e) => {handle_application_error(self, e);}
                    }
                }
            }
            Action::ViewAction(view_action) => {
                use crate::utilities::*;
                //possible modes are Insert and View + any mode with fallthrough to insert
                assert!(matches!(self.mode(), Mode::Insert | Mode::View | Mode::Warning | Mode::Notify | Mode::Info));
                let mut should_exit = false;
                let result = match view_action{
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
                        if self.mode() != Mode::View && self.mode() != Mode::Insert{pop_to_insert(self);}
                        let DisplayArea{horizontal_start, vertical_start, width: _width, height: _height} = view;
                        self.buffer_horizontal_start = horizontal_start;
                        self.buffer_vertical_start = vertical_start;
                    
                        self.update_ui_data_document();
                        if self.mode() == Mode::View && should_exit{self.action(Action::EditorAction(EditorAction::ModePop));}
                    }
                    Err(e) => {
                        match e{
                            DisplayAreaError::InvalidInput => {handle_application_error(self, ApplicationError::InvalidInput);}
                            DisplayAreaError::ResultsInSameState => {handle_application_error(self, ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
                        }
                    }
                }
            }
            Action::UtilAction(util_action) => {
                let text_box = &mut self.ui.util_bar.utility_widget.text_box;
                let mut perform_follow_up_behavior = true;
                match util_action{
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
                    UtilAction::Cut => {
                        self.clipboard = text_box.buffer.slice(text_box.selection.range.start, text_box.selection.range.end).to_string();
                        text_box.delete();
                    }
                    UtilAction::Copy => {self.clipboard = text_box.buffer.slice(text_box.selection.range.start, text_box.selection.range.end).to_string();}
                    UtilAction::Paste => {
                        if text_box.selection.is_extended(){
                            text_box.buffer.apply_replace(&self.clipboard, &mut text_box.selection, CURSOR_SEMANTICS);
                        }else{
                            text_box.buffer.apply_insert(&self.clipboard, &mut text_box.selection, CURSOR_SEMANTICS);
                        }
                    }
                    UtilAction::Accept => {
                        match self.mode(){
                            Mode::Goto => { //TODO: entering a very large number switches util bar text color to the valid state instead of the error state for some reason
                                //parse 1-based line number
                                if let Ok(line_number) = self.ui.util_bar.utility_widget.text_box.buffer.to_string().parse::<usize>(){
                                    if line_number > 0{
                                        // make line number 0 based for interfacing correctly with backend impl
                                        let line_number = line_number.saturating_sub(1);
                                        if line_number < self.buffer.len_lines(){
                                            if self.selections.count() > 1{
                                                if let Ok(new_selections) = crate::utilities::clear_non_primary_selections::selections_impl(&self.selections){self.selections = new_selections;}    //intentionally ignoring any errors
                                            }
                                            match crate::utilities::move_to_line_number::selection_impl(self.selections.primary(), line_number, &self.buffer, crate::selection::Movement::Move, self.config.semantics.clone()){
                                                Ok(new_selection) => {
                                                    *self.selections.primary_mut() = new_selection;
                                                    self.checked_scroll_and_update(
                                                        &self.selections.primary().clone(), 
                                                        Application::update_ui_data_selections, 
                                                        Application::update_ui_data_selections
                                                    ); //TODO: pretty sure one of these should be update_ui_data_document
                                                    self.action(Action::EditorAction(EditorAction::ModePop));
                                                    // center view vertically around new primary, if possible
                                                    if let Ok(new_view) = crate::utilities::center_view_vertically_around_cursor::view_impl(&self.buffer_display_area(), self.selections.primary(), &self.buffer, self.config.semantics.clone()){
                                                        self.buffer_horizontal_start = new_view.horizontal_start;
                                                        self.buffer_vertical_start = new_view.vertical_start;
                                                        self.update_ui_data_document();
                                                    }
                                                    //
                                                }
                                                Err(_) => {handle_message(self, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                                            }
                                        }
                                        //line number >= len lines
                                        else{handle_message(self, INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}
                                    }
                                    //line number <= 0
                                    else{handle_message(self, INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}
                                }
                                //line number not parseable
                                else{handle_message(self, INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}
                            }
                            Mode::Command => {
                                //if parse_command(self, self.ui.util_bar.utility_widget.text_box.buffer.to_string()).is_ok(){
                                if Result::is_ok(&parse_command(self, self.ui.util_bar.utility_widget.text_box.buffer.to_string())){
                                    //only checking command mode because parsed resultant fn may need to enter error/warning/notify/info mode, and user should see that
                                    if self.mode() == Mode::Command{pop_to_insert(self);}
                                }else{handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, COMMAND_PARSE_FAILED);}
                            }
                            Mode::Find | Mode::Split => self.action(Action::EditorAction(EditorAction::ModePop)),
                            Mode::AddSurround | Mode::Insert | Mode::Object | Mode::View | Mode::Error | Mode::Warning | Mode::Notify | Mode::Info => {unreachable!()}
                        }
                        perform_follow_up_behavior = false;
                    }
                    UtilAction::Exit => {
                        match self.mode(){
                            Mode::Find | Mode::Split => {
                                self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
                                self.selections = self.preserved_selections.clone().unwrap();   //shouldn't be called unless this value is Some()
                                self.checked_scroll_and_update(
                                    &self.selections.primary().clone(), 
                                    Application::update_ui_data_document, 
                                    Application::update_ui_data_selections
                                );
                                self.action(Action::EditorAction(EditorAction::ModePop));
                            }
                            _ => {self.action(Action::EditorAction(EditorAction::ModePop));}
                        }
                        perform_follow_up_behavior = false;
                    }
                    UtilAction::GotoModeSelectionAction(selection_action) => {
                        //TODO?: add go to matching surrounding char(curly, square, paren, single quote, double quote, etc)?
                        assert!(self.mode() == Mode::Goto);
                        if let Ok(count) = self.ui.util_bar.utility_widget.text_box.buffer.to_string().parse::<usize>(){
                            self.action(Action::EditorAction(EditorAction::ModePop));
                            assert!(self.mode() == Mode::Insert);
                            self.action(Action::SelectionAction(selection_action, count));
                        }else{handle_message(self, INVALID_INPUT_DISPLAY_MODE, INVALID_INPUT);}  //TODO: this may benefit from a specific error, maybe stating why the input is invalid...empty/non number input string...//"action requires non-empty, numeric input string"
                        //also, this doesn't work with goto_mode_text_validity_check
                        perform_follow_up_behavior = false;
                    }
                }
                if perform_follow_up_behavior{
                    self.update_ui_data_util_bar();
                    //perform any mode specific follow up actions   //shouldn't need to call these if action was a selection action instead of an edit action
                    match self.mode(){
                        Mode::Object |
                        Mode::Insert |
                        Mode::View |
                        Mode::Error |
                        Mode::Warning |
                        Mode::Notify |
                        Mode::Info |
                        Mode::AddSurround => {/*do nothing*/}
                        Mode::Goto => {
                            // run text validity check
                            let mut is_numeric = true;
                            for char in self.ui.util_bar.utility_widget.text_box.buffer.chars(){
                                if !char.is_ascii_digit(){is_numeric = false;}
                            }
                            let exceeds_doc_length = match self.ui.util_bar.utility_widget.text_box.buffer.to_string().parse::<usize>(){
                                Ok(line_number) => {line_number > self.buffer.len_lines()}
                                Err(_) => false //TODO: very large numeric input strings aren't parseable to usize, thus set exceeds_doc_length to false...
                            };
                            self.ui.util_bar.utility_widget.text_box.text_is_valid = is_numeric && !exceeds_doc_length;
                        }
                        Mode::Find => {
                            match &self.preserved_selections{
                                Some(selections_before_search) => {
                                    match crate::utilities::incremental_search_in_selection::selections_impl(
                                        selections_before_search, 
                                        &self.ui.util_bar.utility_widget.text_box.buffer.to_string(),
                                        &self.buffer, 
                                        self.config.semantics.clone()
                                    ){
                                        Ok(new_selections) => {
                                            self.selections = new_selections;
                                            self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
                                        }
                                        Err(_) => {
                                            self.selections = selections_before_search.clone();
                                            self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
                                        }
                                    }
                                }
                                None => {/* maybe error?... */unreachable!()}
                            }
                            self.checked_scroll_and_update(
                                &self.selections.primary().clone(), 
                                Application::update_ui_data_document, 
                                Application::update_ui_data_selections
                            );
                        }
                        Mode::Split => {
                            match &self.preserved_selections{
                                Some(selections_before_split) => {
                                    match crate::utilities::incremental_split_in_selection::selections_impl(
                                        selections_before_split, 
                                        &self.ui.util_bar.utility_widget.text_box.buffer.to_string(),
                                        &self.buffer, 
                                        self.config.semantics.clone()
                                    ){
                                        Ok(new_selections) => {
                                            self.selections = new_selections;
                                            self.ui.util_bar.utility_widget.text_box.text_is_valid = true;
                                        }
                                        Err(_) => {
                                            self.selections = selections_before_split.clone();
                                            self.ui.util_bar.utility_widget.text_box.text_is_valid = false;
                                        }
                                    }
                                }
                                None => {/* maybe error?... */unreachable!()}
                            }
                            self.checked_scroll_and_update(
                                &self.selections.primary().clone(), 
                                Application::update_ui_data_document, 
                                Application::update_ui_data_selections
                            );
                        }
                        Mode::Command => {/*do nothing*/}
                    }
                }
            }
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), String>{
        while !self.should_quit{
            //derive User Interface from Application state
            self.update_layouts();  //TODO: does update_layouts always need to be called, or can this be called only from actions that require it?...
            self.render(terminal)?;            
            //update Application state
            self.handle_event()?;
        }
        Ok(())
    }
}

//TODO: serve events to events file for interested external programs to read. 
//certain events may require blocking until all readers send some sort of acknowledge indication 
//to let edit know to continue with it's usual behavior, or a reader may modify whatever it needs 
//via filesystem interface, then send some finished indication.
//only after edit has received a continue or finished indication, should it unblock
//TODO: research how acme handles events and coordinating response behavior

//TODO: research how acme uses Send + Win to allow a buffer to be used as an interactive command line interface

use std::{io::Write, path::{Path, PathBuf}};
use ratatui::{
    prelude::*,
    widgets::*
};
use crate::{
    config::*,
    mode::Mode,
    action::{Action, EditorAction, SelectionAction, EditAction, ViewAction, UtilAction},
    mode_stack::ModeStack,
    ui::{UserInterface, util_bar::*},
    range::Range,
    buffer::Buffer,
    display_area::{self, DisplayArea, DisplayAreaError},
    selection::{self, Selection, CursorSemantics},
    selections::{self, Selections, SelectionsError},
    history::ChangeSet,
};



pub enum WindowEvent{
    Resize{width: u16, height: u16},
    FocusLost,
    FocusGained,
}
pub enum Event{ //TODO: need to disambiguate these events from events in 9p served events file  //System/Editor, Input/Output, External/Internal, ...
    KeyboardInput(crossterm::event::KeyEvent),
    MouseInput(crossterm::event::MouseEvent),
    NineP(serve9p::file_system::FsRequest),
    Window(WindowEvent),
    //Tick(timed_event_kind),   //maybe for cursor blink or similar...
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
    should_quit: bool,
    mode_stack: ModeStack,
    pub ui: UserInterface, 
    pub buffer_horizontal_start: usize,
    pub buffer_vertical_start: usize,
    config: Config,
    pub buffer: Buffer, 
    preserved_selections: Option<Selections>, 
    pub undo_stack: Vec<ChangeSet>,   //maybe have separate buffer and selections undo/redo stacks?...
    pub redo_stack: Vec<ChangeSet>,
    pub selections: Selections,
    pub clipboard: String,
}
impl Application{
    pub fn new(config: Config, buffer_text: &str, file_path: Option<PathBuf>, read_only: bool, terminal: &Terminal<impl Backend>) -> Result<Self, String>{
        let terminal_size = match terminal.size(){
            Ok(size) => size,
            Err(e) => return Err(format!("{}", e))
        };
        let terminal_rect = Rect::new(0, 0, terminal_size.width, terminal_size.height);

        let buffer = Buffer::new(buffer_text, file_path, read_only);
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
                        config.semantics.clone()
                    )
                ], 
                0, 
                &buffer, 
                config.semantics.clone()
            ),
            buffer_horizontal_start: 0,
            buffer_vertical_start: 0,
            clipboard: String::new(),
        };

        instance.setup();

        Ok(instance)
    }
    //start_selection: StartSelection{Point{line: u16, column: u16}, Regex{regex: String}}
    fn setup(&mut self/*TODO:, cursor_line_number: usize, cursor_column_number: usize */){
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
        self.layout();
        self.checked_scroll_and_update(
            &self.selections.primary.clone(),
            Application::update_ui_data_document, 
            Application::update_ui_data_document    //should this be update_selections?...
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
    pub fn mode(&self) -> Mode{self.mode_stack.top().clone()}

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
        self.ui.status_bar.selections_widget.text = format!("selections: {}/{}", &self.selections.primary_selection_index() + 1, &self.selections.count());
        let cursor_position = &self.selections.primary.selection_to_selection2d(&self.buffer, self.config.semantics.clone()).head().clone();
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

    pub fn layout(&mut self){   //-> Result<(), String>{ //to handle terminal.size() error
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
        fn layout_status_bar(rect: Rect) -> std::rc::Rc<[Rect]>{
            // layout of status bar rect (modified_indicator/file_name/cursor_position)
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    vec![
                        //[0]
                        // first third
                        Constraint::Ratio(1, 3),

                        //[1]
                        // middle third
                        Constraint::Ratio(1, 3),

                        //[2]
                        // last third
                        Constraint::Ratio(1, 3)
                    ]
                )
                .split(rect)
        }
        fn layout_status_bar_first_third(app: &Application, rect: Rect) -> std::rc::Rc<[Rect]>{
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
                        // padding_3
                        Constraint::Min(0)
                    ]
                )
                .split(rect)
        }
        fn layout_status_bar_middle_third(rect: Rect) -> std::rc::Rc<[Rect]>{
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    vec![
                        //[0]
                        // selections widget
                        Constraint::Min(0),     //or set selections widget to Max, and surround with 2 padding widgets set to Min(0)?...idk if that will work the same?...
                    ]
                )
                .split(rect)
        }
        fn layout_status_bar_last_third(app: &Application, rect: Rect) -> std::rc::Rc<[Rect]>{
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    vec![
                        //[0]
                        // padding_4
                        Constraint::Min(0),

                        //[1]
                        // cursor position indicator width
                        Constraint::Max(
                            app.ui.status_bar.cursor_position_widget.text.len() as u16
                        ),
                    
                        //[2]
                        // padding_3
                        Constraint::Max(1),
                    
                        //[3]
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
        let status_bar_rect = layout_status_bar(terminal_rect[1]);
        let util_rect = layout_util_bar(self, terminal_rect[2]);

        let status_bar_first_third_rect = layout_status_bar_first_third(self, status_bar_rect[0]);
        let status_bar_middle_third_rect = layout_status_bar_middle_third(status_bar_rect[1]);
        let status_bar_last_third_rect = layout_status_bar_last_third(self, status_bar_rect[2]);
    
        self.ui.document_viewport.line_number_widget.rect = document_viewport_rect[0];
        self.ui.document_viewport.padding.rect = document_viewport_rect[1];
        self.ui.document_viewport.document_widget.rect = document_viewport_rect[2];
            
        self.ui.status_bar.read_only_widget.rect =       status_bar_first_third_rect[0];//status_bar_rect[0];
        self.ui.status_bar.padding_1.rect =              status_bar_first_third_rect[1];//status_bar_rect[1];
        self.ui.status_bar.file_name_widget.rect =       status_bar_first_third_rect[2];//status_bar_rect[2];
        self.ui.status_bar.padding_2.rect =              status_bar_first_third_rect[3];//status_bar_rect[3];
        self.ui.status_bar.modified_widget.rect =        status_bar_first_third_rect[4];//status_bar_rect[4];
        self.ui.status_bar.padding_3.rect =              status_bar_first_third_rect[5];
        
        self.ui.status_bar.selections_widget.rect =      status_bar_middle_third_rect[0];//status_bar_rect[5];
        
        self.ui.status_bar.padding_4.rect =              status_bar_last_third_rect[0];
        self.ui.status_bar.cursor_position_widget.rect = status_bar_last_third_rect[1];//status_bar_rect[6];
        self.ui.status_bar.padding_5.rect =              status_bar_last_third_rect[2];//status_bar_rect[7];
        self.ui.status_bar.mode_widget.rect =            status_bar_last_third_rect[3];//status_bar_rect[8];
            
        self.ui.util_bar.prompt.rect = util_rect[0];
        self.ui.util_bar.utility_widget.rect = util_rect[1];
            
        self.ui.popups.goto.rect = sized_centered_rect(self.ui.popups.goto.widest_element_len, self.ui.popups.goto.num_elements, self.ui.terminal_size);
        self.ui.popups.command.rect = sized_centered_rect(self.ui.popups.command.widest_element_len, self.ui.popups.command.num_elements, self.ui.terminal_size);
        self.ui.popups.find.rect = sized_centered_rect(self.ui.popups.find.widest_element_len, self.ui.popups.find.num_elements, self.ui.terminal_size);
        self.ui.popups.split.rect = sized_centered_rect(self.ui.popups.split.widest_element_len, self.ui.popups.split.num_elements, self.ui.terminal_size);
        self.ui.popups.error.rect = sized_centered_rect(self.ui.popups.error.widest_element_len, self.ui.popups.error.num_elements, self.ui.terminal_size);
        self.ui.popups.modified_error.rect = sized_centered_rect(self.ui.popups.modified_error.widest_element_len, self.ui.popups.modified_error.num_elements, self.ui.terminal_size);
        self.ui.popups.warning.rect = sized_centered_rect(self.ui.popups.warning.widest_element_len, self.ui.popups.warning.num_elements, self.ui.terminal_size);
        self.ui.popups.notify.rect = sized_centered_rect(self.ui.popups.notify.widest_element_len, self.ui.popups.notify.num_elements, self.ui.terminal_size);
        self.ui.popups.info.rect = sized_centered_rect(self.ui.popups.info.widest_element_len, self.ui.popups.info.num_elements, self.ui.terminal_size);
        self.ui.popups.view.rect = sized_centered_rect(self.ui.popups.view.widest_element_len, self.ui.popups.view.num_elements, self.ui.terminal_size);
        self.ui.popups.object.rect = sized_centered_rect(self.ui.popups.object.widest_element_len, self.ui.popups.object.num_elements, self.ui.terminal_size);
        self.ui.popups.add_surround.rect = sized_centered_rect(self.ui.popups.add_surround.widest_element_len, self.ui.popups.add_surround.num_elements, self.ui.terminal_size);
    }
    
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
                    frame.render_widget(generate_widget("", Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, STATUS_BAR_BACKGROUND_COLOR), self.ui.status_bar.padding_3.rect);
                    
                    frame.render_widget(generate_widget(&self.ui.status_bar.selections_widget.text, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, SELECTIONS_WIDGET_FOREGROUND_COLOR), self.ui.status_bar.selections_widget.rect);
                    
                    frame.render_widget(generate_widget("", Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, STATUS_BAR_BACKGROUND_COLOR), self.ui.status_bar.padding_4.rect);
                    frame.render_widget(generate_widget(&self.ui.status_bar.cursor_position_widget.text, Alignment::Center, true, STATUS_BAR_BACKGROUND_COLOR, CURSOR_POSITION_WIDGET_FOREGROUND_COLOR), self.ui.status_bar.cursor_position_widget.rect);
                    frame.render_widget(generate_widget("", Alignment::Center, false, STATUS_BAR_BACKGROUND_COLOR, Color::Red), self.ui.status_bar.padding_5.rect);
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
                            frame.render_widget(generate_popup(&self.ui.popups.goto.text, &format!("{}: {}", self.ui.popups.goto.title, self.mode_stack.len())/*&self.ui.popups.goto.title*/, Color::Black, Color::Yellow), self.ui.popups.goto.rect);
                        }
                    }
                    Mode::Command => {
                        frame.render_widget(generate_widget(COMMAND_PROMPT, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.prompt.rect);
                        frame.render_widget(generate_widget(&self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        render_util_bar_highlights(self, frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.command.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.command.text, &format!("{}: {}", self.ui.popups.command.title, self.mode_stack.len())/*&self.ui.popups.command.title*/, Color::Black, Color::Yellow), self.ui.popups.command.rect);
                        }
                    }
                    Mode::Find => {
                        frame.render_widget(generate_widget(FIND_PROMPT, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.prompt.rect);
                        frame.render_widget(generate_widget(&self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, if self.ui.util_bar.utility_widget.text_box.text_is_valid{UTIL_BAR_FOREGROUND_COLOR}else{UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR}), self.ui.util_bar.utility_widget.rect);
                        render_util_bar_highlights(self, frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.find.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.find.text, &format!("{}: {}", self.ui.popups.find.title, self.mode_stack.len())/*&self.ui.popups.find.title*/, Color::Black, Color::Yellow), self.ui.popups.find.rect);
                        }
                    }
                    Mode::Split => {
                        frame.render_widget(generate_widget(SPLIT_PROMPT, Alignment::Center, false, UTIL_BAR_BACKGROUND_COLOR, UTIL_BAR_FOREGROUND_COLOR), self.ui.util_bar.prompt.rect);
                        frame.render_widget(generate_widget(&self.text_box_display_area().text(&self.ui.util_bar.utility_widget.text_box.buffer), Alignment::Left, false, UTIL_BAR_BACKGROUND_COLOR, if self.ui.util_bar.utility_widget.text_box.text_is_valid{UTIL_BAR_FOREGROUND_COLOR}else{UTIL_BAR_INVALID_TEXT_FOREGROUND_COLOR}), self.ui.util_bar.utility_widget.rect);
                        render_util_bar_highlights(self, frame.buffer_mut());
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.split.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.split.text, &format!("{}: {}", self.ui.popups.split.title, self.mode_stack.len())/*&self.ui.popups.split.title*/, Color::Black, Color::Yellow), self.ui.popups.split.rect);
                        }
                    }
                    Mode::Error => {
                        frame.render_widget(generate_widget(&self.mode_stack.top_message().expect("text being Some should be guaranteed in Error mode"), Alignment::Center, true, ERROR_BACKGROUND_COLOR, ERROR_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        //if &self.mode_stack.top().text.expect("text being Some should be guaranteed in Error mode") == FILE_MODIFIED{
                        if self.mode_stack.top_message() == Some(FILE_MODIFIED.to_string()){
                            if SHOW_CONTEXTUAL_KEYBINDS{
                                frame.render_widget(ratatui::widgets::Clear, self.ui.popups.modified_error.rect);
                                frame.render_widget(generate_popup(&self.ui.popups.modified_error.text, &format!("{}: {}", self.ui.popups.modified_error.title, self.mode_stack.len())/*&self.ui.popups.modified_error.title*/, Color::Black, Color::Yellow), self.ui.popups.modified_error.rect);
                            }
                        }
                        else{
                            if SHOW_CONTEXTUAL_KEYBINDS{
                                frame.render_widget(ratatui::widgets::Clear, self.ui.popups.error.rect);
                                frame.render_widget(generate_popup(&self.ui.popups.error.text, &format!("{}: {}", self.ui.popups.error.title, self.mode_stack.len())/*&self.ui.popups.error.title*/, Color::Black, Color::Yellow), self.ui.popups.error.rect);
                            }
                        }
                    }
                    Mode::Warning => {
                        frame.render_widget(generate_widget(&self.mode_stack.top_message().expect("text being Some should be guaranteed in Warning mode"), Alignment::Center, true, WARNING_BACKGROUND_COLOR, WARNING_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.warning.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.warning.text, &format!("{}: {}", self.ui.popups.warning.title, self.mode_stack.len())/*&self.ui.popups.warning.title*/, Color::Black, Color::Yellow), self.ui.popups.warning.rect);
                        }
                    }
                    Mode::Notify => {
                        frame.render_widget(generate_widget(&self.mode_stack.top_message().expect("text being Some should be guaranteed in Notify mode"), Alignment::Center, true, NOTIFY_BACKGROUND_COLOR, NOTIFY_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.notify.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.notify.text, &format!("{}: {}", self.ui.popups.notify.title, self.mode_stack.len())/*&self.ui.popups.notify.title*/, Color::Black, Color::Yellow), self.ui.popups.notify.rect);
                        }
                    }
                    Mode::Info => {
                        frame.render_widget(generate_widget(&self.mode_stack.top_message().expect("text being Some should be guaranteed in Info mode"), Alignment::Center, true, INFO_BACKGROUND_COLOR, INFO_FOREGROUND_COLOR), self.ui.util_bar.utility_widget.rect);
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.info.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.info.text, &format!("{}: {}", self.ui.popups.info.title, self.mode_stack.len())/*&self.ui.popups.info.title*/, Color::Black, Color::Yellow), self.ui.popups.info.rect);
                        }
                    }
                    Mode::View => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.view.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.view.text, &format!("{}: {}", self.ui.popups.view.title, self.mode_stack.len())/*&self.ui.popups.view.title*/, Color::Black, Color::Yellow), self.ui.popups.view.rect);
                        }
                    }
                    Mode::Object => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.object.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.object.text, &format!("{}: {}", self.ui.popups.object.title, self.mode_stack.len())/*&self.ui.popups.object.title*/, Color::Black, Color::Yellow), self.ui.popups.object.rect);
                        }
                    }
                    Mode::AddSurround => {
                        if SHOW_CONTEXTUAL_KEYBINDS{
                            frame.render_widget(ratatui::widgets::Clear, self.ui.popups.add_surround.rect);
                            frame.render_widget(generate_popup(&self.ui.popups.add_surround.text, &format!("{}: {}", self.ui.popups.add_surround.title, self.mode_stack.len())/*&self.ui.popups.add_surround.title*/, Color::Black, Color::Yellow), self.ui.popups.add_surround.rect);
                        }
                    }
                }
            }
        ){
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{e}"))
        }
    }

/*
    Mouse behavior                                      keyboard equivalent
    standard behavior:
    Button one -> select text                           normal selection movement/extension behavior
        double click -> select word under cursor        normal selection movement/extension behavior
            if at line start/end, select whole line
    Button two -> execute text                          execute selection or word(if non extended selection)
    Button three -> plumb                               plumb selection or word(if non extended selection)
        if plumb fails, search for next occurrance          if plumb fails, search buffer for all occurrances of text
        of text

    for each button
    click + drag - select text                          normal movement/extension behavior

    combination uses:
    left + middle                                       save command string to clipboard
        select text with left, middle click command     select text
        sends selected text through command             execute clipboard as command, with selected text as command input

    left + right                                        select text as normal
        select text with left, right click look         trigger look(whole buffer, not search within selection)
        looks for selected text in buffer

    chords:
    after selecting with button one and while still holding button one down (these chords also word with text selected by double-clicking, 
        the double-click expansion happens when the second click starts, not when it ends)
        - clicking button two cuts
        - clicking button three pastes (can be reverted by clicking button two immediately afterwards)
        - to copy, click button two immediately followed by button three
    while holding down button 2 on text to be executed as a command, clicking button 1 appends the text last pointed by button 1 as a
        distinct final argument. (for example, to search for literal text one may execute Look text with button 2 or instead point at
        text with button 1 in any window, release button 1, then execute Look, clicking button 1 while 2 is held down)
*/

    //TODO: have handle_event return Result<Option<Action>, String>, instead of triggering the action directly
    //Option<Action> because the next step may need to update the same variable if subsequent actions need to be performed
    //though this should always return Some(Action), unless an error is encountered in event reading...
    //alternatively, the next step could do: let action = Some(self.handle_event()?);
    fn handle_event(&mut self, event_rx: &std::sync::mpsc::Receiver<Event>) -> Result<(), String>{
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
        //match event_rx.recv(){  //or maybe try_recv() and match error?...
        match event_rx.recv_timeout(std::time::Duration::from_millis(30)){  //still hanging up randomly on quit...
            Ok(event) => {
                match event{
                    Event::KeyboardInput(key_event) => {
                        self.update(
                            match self.config.keybinds.get(&(self.mode(), key_event)).cloned(){
                                Some(action) => action,
                                None => {
                                    match self.mode(){
                                        //unbound key presses for these modes fall through to insert mode
                                        Mode::Warning | Mode::Notify | Mode::Info => {
                                            //spoofing our mode as insert, to handle fall through
                                            match self.config.keybinds.get(&(Mode::Insert, key_event)).cloned(){
                                                //some actions may need to be modified to pop until insert mode, 
                                                //because we aren't actually in insert yet
                                                //the application will panic/misbehave if action does not handle this...
                                                Some(insert_action) => insert_action,
                                                None => handle_char_insert(Mode::Insert, key_event)
                                            }
                                        }
                                        _ => handle_char_insert(self.mode(), key_event)
                                    }
                                }
                            }
                        );
                    },
                    //TODO: figure out how to add mouse events to config.keybinds
                    //TODO: figure out how to accomplish acme style mouse chords
                    Event::MouseInput(mouse_event) => {//self.action(Action::EditorAction(EditorAction::NoOpEvent)),
                        let crossterm::event::MouseEvent{kind, column, row, modifiers} = mouse_event;
                        use crossterm::event::{MouseEventKind, KeyModifiers, MouseButton};
                        match kind{
                            MouseEventKind::Down(mouse_button) => {
                                if modifiers == KeyModifiers::NONE{
                                    if mouse_button == MouseButton::Left{
                                        //TODO: need to impl display coords to buffer coords
                                    }else{}
                                }
                                else if modifiers == KeyModifiers::CONTROL{
                                    if mouse_button == MouseButton::Left{
                                        //TODO: add cursor at click location
                                    }else{}
                                }
                                else{}
                            }
                            MouseEventKind::Up(mouse_button) => {

                            }
                            MouseEventKind::ScrollDown => {
                                if modifiers == KeyModifiers::NONE{
                                    self.update(Action::ViewAction(ViewAction::ScrollDown));
                                }
                                else if modifiers == KeyModifiers::CONTROL{
                                    self.update(Action::ViewAction(ViewAction::ScrollRight));
                                }
                                else{}
                            }
                            MouseEventKind::ScrollUp => {
                                if modifiers == KeyModifiers::NONE{
                                    self.update(Action::ViewAction(ViewAction::ScrollUp));
                                }
                                else if modifiers == KeyModifiers::CONTROL{
                                    self.update(Action::ViewAction(ViewAction::ScrollLeft));
                                }
                                else{}
                            }
                            MouseEventKind::Drag(mouse_button) => {}
                            MouseEventKind::Moved => {}
                        }
                    }
                    Event::Window(window_event) => {
                        match window_event{
                            WindowEvent::Resize { width, height } => {
                                self.ui.set_terminal_size(width, height);
                                self.layout();
                                self.update_ui_data_util_bar(); //TODO: can this be called later in fn impl?
                                // scrolling so cursor is in a reasonable place, and updating so any ui changes render correctly
                                self.checked_scroll_and_update(
                                    &self.selections.primary.clone(),
                                    Application::update_ui_data_document, 
                                    Application::update_ui_data_document
                                );
                            }
                            WindowEvent::FocusLost => {self.update(Action::EditorAction(EditorAction::NoOpEvent))}  //maybe quit displaying cursor(s)/selection(s)?...
                            WindowEvent::FocusGained => {self.update(Action::EditorAction(EditorAction::NoOpEvent))}    //display cursor(s)/selection(s)?...
                        }
                    }
                    Event::NineP(fs_request) => {}
                }
                Ok(())
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {Ok(())}
            Err(e) => Err(format!("{e}"))
        }
    }

    //TODO?: should each action result in a new app state, or an error state?... can these be made more purely functional?...
    pub fn update(&mut self, action: Action){
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
            while app.mode() != Mode::Insert{app.update(Action::EditorAction(EditorAction::ModePop));}
        }
        match action{
            Action::EditorAction(editor_action) => {
                match editor_action{
                    EditorAction::ModePop => {
                        fn perform_shared_behavior(app: &mut Application){
                            //update layouts and document
                            app.layout();
                            app.update_ui_data_document();
                            // clear util bar text
                            app.ui.util_bar.utility_widget.text_box.clear();
                            app.update_ui_data_util_bar();
                        }
                        //remove current mode from stack
                        if let Ok((popped_mode, popped_text)) = self.mode_stack.pop(){
                            if popped_mode == self.mode() && popped_text == self.mode_stack.top_message(){
                                //continue popping until self.mode() is something else (this would clean up repeated error messages/etc.)
                                self.update(Action::EditorAction(EditorAction::ModePop));
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
                    EditorAction::ModePush(to_mode, message) => {
                        fn perform_shared_behavior(app: &mut Application){
                            //update layouts and document
                            app.layout();
                            app.update_ui_data_document();
                            //update util bar
                            app.update_ui_data_util_bar();
                        }
                        //let to_mode = stack_member.mode.clone();
                        match to_mode{
                            Mode::Find | Mode::Split => {
                                pop_to_insert(self);
                                self.mode_stack.push(to_mode, message);
                                self.preserved_selections = Some(self.selections.clone());  //save selections
                                if !self.ui.status_bar.show{ // potential fix for status bar bug in todo.rs
                                    perform_shared_behavior(self);
                                }
                            }
                            Mode::Command | Mode::Goto => {
                                pop_to_insert(self);
                                self.mode_stack.push(to_mode, message);
                                if !self.ui.status_bar.show{ // potential fix for status bar bug in todo.rs
                                    perform_shared_behavior(self);
                                }
                            }
                            Mode::Object | Mode::AddSurround | Mode::View => {
                                pop_to_insert(self);
                                self.mode_stack.push(to_mode, message);
                            }
                            Mode::Error | Mode::Warning | Mode::Notify | Mode::Info => {self.mode_stack.push(to_mode, message);}
                            Mode::Insert => {unreachable!()}    //should always pop to Insert, never push to Insert
                        }
                        //does this belong here, or in ui.rs?...    //by calling here, we only perform this calculation as needed, not on every editor run cycle
                        self.update_ui_data_mode();
                    }
                    EditorAction::NoOpKeypress => {handle_message(self, UNHANDLED_KEYPRESS_DISPLAY_MODE, UNHANDLED_KEYPRESS);}
                    EditorAction::NoOpEvent => {handle_message(self, UNHANDLED_EVENT_DISPLAY_MODE, UNHANDLED_EVENT);}
                    EditorAction::Quit => {
                        //possible modes are Insert and Command + any mode with fallthrough to insert
                        assert!(matches!(self.mode(), Mode::Insert | Mode::Command | Mode::Error | Mode::Warning | Mode::Notify | Mode::Info));
                        if self.buffer.is_modified(){
                            if self.mode() == Mode::Error && self.mode_stack.top_message().unwrap() == FILE_MODIFIED{
                                self.update(Action::EditorAction(EditorAction::QuitIgnoringChanges));
                            }
                            else{
                                handle_message(self, DisplayMode::Error, FILE_MODIFIED);
                            }
                        }
                        else{
                            if self.mode() == Mode::Error{self.update(Action::EditorAction(EditorAction::NoOpKeypress));}
                            else{self.should_quit = true;}
                        }
                    }
                    EditorAction::QuitIgnoringChanges => {
                        assert!(
                            self.mode() == Mode::Error && self.mode_stack.top_message().unwrap() == FILE_MODIFIED ||
                            self.mode() == Mode::Command ||
                            self.mode() == Mode::Insert
                        );
                        self.should_quit = true;
                    }
                    EditorAction::Save => {
                        //possible modes are Insert and Command + any mode with fallthrough to insert
                        assert!(matches!(self.mode(), Mode::Insert | Mode::Command | Mode::Warning | Mode::Notify | Mode::Info));
                        //if self.buffer.file_path.is_none(){
                        //    pop_to_insert(self);
                        //    handle_message(self, DisplayMode::Error, "cannot save unnamed buffer");
                        //}
                        //else if self.buffer.read_only{
                        //    pop_to_insert(self);
                        //    handle_message(self, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER);
                        //}
                        //else{
                        //    match crate::utilities::save::application_impl(self){
                        //        Ok(()) => {
                        //            pop_to_insert(self);
                        //            self.update_ui_data_document();
                        //        }
                        //        //this could maybe benefit from passing the io error up to this fn...
                        //        Err(_) => {handle_message(self, FILE_SAVE_FAILED_DISPLAY_MODE, FILE_SAVE_FAILED);}
                        //    }
                        //}
                        match save(self){
                            Ok(()) => {
                                pop_to_insert(self);
                                self.update_ui_data_document();
                            }
                            Err(e) => {
                                pop_to_insert(self);
                                handle_message(self, 
                                    match e.as_str(){
                                        READ_ONLY_BUFFER => READ_ONLY_BUFFER_DISPLAY_MODE,
                                        SAME_STATE => SAME_STATE_DISPLAY_MODE,
                                        _ => DisplayMode::Error
                                    }, 
                                    &e
                                );
                            }
                        }
                    }
                    EditorAction::Copy => {
                        //possible modes are Insert + any mode with fallthrough to insert
                        assert!(matches!(self.mode(), Mode::Insert | Mode::Warning | Mode::Notify | Mode::Info));
                        match copy(self){
                            Ok(()) => {
                                pop_to_insert(self);
                                handle_message(self, COPIED_TEXT_DISPLAY_MODE, COPIED_TEXT);
                                self.update_ui_data_document(); //TODO: is this really needed for something?...
                            }
                            Err(e) => {handle_application_error(self, e);}
                        }
                    }
                    //TODO: remove this in favor of a user defined command
                    //EditorAction::OpenNewTerminalWindow => {
                    //    assert!(matches!(self.mode(), Mode::Insert | Mode::Command | Mode::Warning | Mode::Notify | Mode::Info));
                    //    //if matches!(self.mode(), Mode::Warning | Mode::Notify | Mode::Info){pop_to_insert(self);}   //handle insert fallthrough
                    //    if self.mode() != Mode::Insert{pop_to_insert(self);}
                    //    let result = std::process::Command::new("alacritty")     //TODO: have user define TERMINAL const in config.rs   //or check env vars for $TERM?
                    //        //.arg("msg")     // these extra commands just make new instances use the same backend(daemon?)
                    //        //.arg("create-window")
                    //        //.current_dir(std::env::current_dir().unwrap())    //not needed here, because term spawned here defaults to this directory, but good to know
                    //        .spawn();
                    //        //.expect("failed to spawn new terminal at current directory");
                    //    if let Err(e) = result{handle_message(self, DisplayMode::Error, &format!("{e}"));}
                    //}
                    EditorAction::ToggleLineNumbers => {
                        //TODO: this may need to handle insert fallthrough
                        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
                        self.ui.document_viewport.line_number_widget.show = !self.ui.document_viewport.line_number_widget.show;
                        self.layout();
                        self.update_ui_data_document();
                    }
                    EditorAction::ToggleStatusBar => {
                        //TODO: this may need to handle insert fallthrough
                        assert!(self.mode() == Mode::Insert || self.mode() == Mode::Command);
                        self.ui.status_bar.show = !self.ui.status_bar.show;
                        self.layout();
                        self.update_ui_data_document();
                    }
                    //could become a command: evaluate_command %val{selection}
                    EditorAction::EvaluateSelectionAsCommand => {
                        if self.mode() != Mode::Insert{pop_to_insert(self);}    //handle insert fallthrough
                        //TODO: figure out best way to handle multiple selections...
                        if self.selections.count() > 1{
                            handle_application_error(self, ApplicationError::SelectionsError(SelectionsError::MultipleSelections));
                        }else{
                            let execute_result = execute_command(self, &self.selections.primary.to_string(&self.buffer));
                            if Result::is_err(&execute_result){
                                let error = Result::unwrap_err(execute_result);
                                handle_message(self, DisplayMode::Error, &error);
                            }
                        }
                    }
                    //this, in combination with copy, is the keyboard centric version of plan9's acme's 2-1 mouse chording
                    EditorAction::EvaluateClipboardAsCommand => {
                        if self.mode() != Mode::Insert{pop_to_insert(self);}    //handle insert fallthrough
                        let execute_result = execute_command(self, &self.clipboard.clone());
                        if Result::is_err(&execute_result){
                            let error = Result::unwrap_err(execute_result);
                            handle_message(self, DisplayMode::Error, &error);
                        }
                    }
                    EditorAction::EvaluateSelectionAsLookObject => {
                        if self.mode() != Mode::Insert{pop_to_insert(self);}    //handle insert fallthrough
                        //TODO: figure out best way to handle multiple selections...
                        if self.selections.count() > 1{
                            handle_application_error(self, ApplicationError::SelectionsError(SelectionsError::MultipleSelections));
                        }else{
                            //expand selection, if not extended
                            //try interpret as file, if looks like file, plumb
                            //try search in document
                                //should we only select next occurrance(like acme), or select all occurrances(this seems more our style...)
                            //warn/error if all else fails
                            //handle_message(self, DisplayMode::Error, "Look unimplemented");

                            let current_primary = &self.selections.primary;
                            let input = &self.selections.primary.to_string(&self.buffer);
                            let input = input.trim();   //handle calling with '\n' or ' '. should not be necessary when .is_extended() checked...
                            match search(input, &self.buffer, self.config.semantics.clone()){
                                Err(error) => handle_application_error(self, ApplicationError::SelectionsError(error)),
                                Ok(mut new_selections) => {
                                    //figure out new primary selection here, so that we don't pollute search fn with the idea that this needs to always happen
                                    //for example, we wouldn't want this when we copy the command "search idk" to clipboard, delete, and then evaluate it
                                    //there is prob a more efficient way to accomplish this
                                    for (i, new_selection) in new_selections.clone().iter().enumerate(){
                                        if new_selection.range == current_primary.range{
                                            new_selections = Selections::new(new_selections.flatten(), i, &self.buffer, self.config.semantics.clone());
                                        }
                                    }
                                    //TODO: this is failing to trigger because stored line offset in new_selections is None.
                                    //when we transition to sum_tree style buffer, stored_line_offset will not be part of Selection,
                                    //and instead be part of DisplayMap, so this should be resolved
                                    if new_selections == self.selections{handle_application_error(self, ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
                                    else{
                                        self.selections = new_selections;
                                        self.checked_scroll_and_update(
                                            &self.selections.primary.clone(), 
                                            Application::update_ui_data_document, 
                                            Application::update_ui_data_selections
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Action::SelectionAction(selection_action, count) => {
                //use crate::utilities::*;
                //possible modes are Insert and Object + any mode with fallthrough to insert
                assert!(matches!(self.mode(), Mode::Insert | Mode::Object | Mode::Warning | Mode::Notify | Mode::Info));
                enum SelectionToFollow{Primary,First,Last}

                let (result, selection_to_follow) = match selection_action{
                    SelectionAction::MoveCursorUp => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::move_cursor_up), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorDown => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::move_cursor_down), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorLeft => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::move_cursor_left), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorRight => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::move_cursor_right), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorWordBoundaryForward => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::move_cursor_word_boundary_forward), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorWordBoundaryBackward => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::move_cursor_word_boundary_backward), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorLineEnd => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::move_cursor_line_end), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorHome => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::move_cursor_home), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorBufferStart => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::move_cursor_buffer_start), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorBufferEnd => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::move_cursor_buffer_end), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorPageUp => {(self.selections.move_selection(count, &self.buffer, Some(&self.buffer_display_area()), self.config.semantics.clone(), selection::move_cursor_page_up), SelectionToFollow::Primary)}
                    SelectionAction::MoveCursorPageDown => {(self.selections.move_selection(count, &self.buffer, Some(&self.buffer_display_area()), self.config.semantics.clone(), selection::move_cursor_page_down), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionUp => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::extend_selection_up), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionDown => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::extend_selection_down), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionLeft => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::extend_selection_left), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionRight => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::extend_selection_right), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionWordBoundaryBackward => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::extend_selection_word_boundary_backward), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionWordBoundaryForward => {(self.selections.move_selection(count, &self.buffer, None, self.config.semantics.clone(), selection::extend_selection_word_boundary_forward), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionLineEnd => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::extend_selection_line_end), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionHome => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::extend_selection_home), SelectionToFollow::Primary)}                    
                    SelectionAction::ExtendSelectionBufferStart => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::extend_selection_buffer_start), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionBufferEnd => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::extend_selection_buffer_end), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionPageUp => {(self.selections.move_selection(count, &self.buffer, Some(&self.buffer_display_area()), self.config.semantics.clone(), selection::extend_selection_page_up), SelectionToFollow::Primary)}
                    SelectionAction::ExtendSelectionPageDown => {(self.selections.move_selection(count, &self.buffer, Some(&self.buffer_display_area()), self.config.semantics.clone(), selection::extend_selection_page_down), SelectionToFollow::Primary)}                    
                    SelectionAction::SelectLine => {(self.selections.move_cursor_potentially_overlapping(&self.buffer, self.config.semantics.clone(), selection::select_line), SelectionToFollow::Primary)}
                    SelectionAction::SelectAll => {(self.selections.move_cursor_clearing_non_primary(&self.buffer, self.config.semantics.clone(), selection::select_all), SelectionToFollow::Primary)}
                    SelectionAction::CollapseSelectionToAnchor => {(self.selections.move_cursor_non_overlapping(&self.buffer, self.config.semantics.clone(), selection::collapse_selection_to_anchor), SelectionToFollow::Primary)}
                    SelectionAction::CollapseSelectionToCursor => {(self.selections.move_cursor_non_overlapping(&self.buffer, self.config.semantics.clone(), selection::collapse_selection_to_cursor), SelectionToFollow::Primary)}
                    SelectionAction::ClearNonPrimarySelections => {(selections::clear_non_primary_selections(&self.selections), SelectionToFollow::Primary)}
                    SelectionAction::AddSelectionAbove => {(selections::add_selection_above(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::First)}
                    SelectionAction::AddSelectionBelow => {(selections::add_selection_below(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::Last)}
                    SelectionAction::RemovePrimarySelection => {(selections::remove_primary_selection(&self.selections), SelectionToFollow::Primary)}
                    SelectionAction::IncrementPrimarySelection => {(selections::increment_primary_selection(&self.selections), SelectionToFollow::Primary)}
                    SelectionAction::DecrementPrimarySelection => {(selections::decrement_primary_selection(&self.selections), SelectionToFollow::Primary)}
                    SelectionAction::Surround => {(selections::surround(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::Primary)},
                    SelectionAction::FlipDirection => {(self.selections.move_cursor_non_overlapping(&self.buffer, self.config.semantics.clone(), selection::flip_direction), SelectionToFollow::Primary)},
                
                        //These may technically be distinct from the other selection actions, because they could be called from object mode, and would need to pop the mode stack after calling...
                        //TODO: SelectionAction::Word => {self.document.word()}
                        //TODO: SelectionAction::Sentence => {self.document.sentence()}
                        //TODO: SelectionAction::Paragraph => {self.document.paragraph()}
                    SelectionAction::SurroundingPair => {(selections::nearest_surrounding_pair(&self.selections, &self.buffer, self.config.semantics.clone()), SelectionToFollow::Primary)}  //TODO: rename SurroundingBracketPair
                        //TODO: SelectionAction::QuotePair => {self.document.nearest_quote_pair()}                      //TODO: rename SurroundingQuotePair
                        //TODO: SelectionAction::ExclusiveSurroundingPair => {self.document.exclusive_surrounding_pair()}
                        //TODO: SelectionAction::InclusiveSurroundingPair => {self.document.inclusive_surrounding_pair()}
                };
            
                match result{
                    Ok(new_selections) => {
                        self.selections = new_selections;
                    
                        //pop_to_insert(self);  //testing to see if this increments SELECTION_ACTION_DISPLAY_MODE if selection_out_of_view
                        fn same_mode(mode: Mode, display_mode: DisplayMode) -> bool{
                            //if mode == Mode::Error && display_mode == DisplayMode::Error{true}
                            //else if mode == Mode::Warning && display_mode == DisplayMode::Warning{true}
                            //else if mode == Mode::Notify && display_mode == DisplayMode::Notify{true}
                            //else if mode == Mode::Info && display_mode == DisplayMode::Info{true}
                            //else{false}
                            (mode == Mode::Error && display_mode == DisplayMode::Error) ||
                            (mode == Mode::Warning && display_mode == DisplayMode::Warning) ||
                            (mode == Mode::Notify && display_mode == DisplayMode::Notify) ||
                            (mode == Mode::Info && display_mode == DisplayMode::Info)
                        }
                        if same_mode(self.mode(), SELECTION_ACTION_DISPLAY_MODE) 
                        && self.mode_stack.top_message() == Some(SELECTION_ACTION_OUT_OF_VIEW.to_string()){
                            /* retain mode as SELECTION_ACTION_DISPLAY_MODE */
                        }else{
                            pop_to_insert(self);
                        }
                        //
                        let primary_selection = &self.selections.primary.clone();
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
                            //testing to see if this increments SELECTION_ACTION_DISPLAY_MODE if selection_out_of_view
                            else{
                                pop_to_insert(self);
                            }
                            //
                        //
                    }
                    Err(e) => {handle_application_error(self, ApplicationError::SelectionsError(e));}
                }
            }
            Action::EditAction(edit_action) => {
                //possible modes are Insert and AddSurround + any mode with fallthrough to insert
                assert!(matches!(self.mode(), Mode::Command | Mode::Insert | Mode::AddSurround | Mode::Warning | Mode::Notify | Mode::Info));

                if self.buffer.read_only{handle_message(self, READ_ONLY_BUFFER_DISPLAY_MODE, READ_ONLY_BUFFER);}
                else{
                    let result = match edit_action{
                        EditAction::InsertChar(c) => insert_string(self, &c.to_string(), self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::InsertNewline => insert_string(self, "\n", self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::InsertTab => insert_string(self, "\t", self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::Delete => delete(self, self.config.semantics.clone()),
                        EditAction::Backspace => backspace(self, self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::Cut => cut(self, self.config.semantics.clone()),
                        EditAction::Paste => paste(self, self.config.use_hard_tab, self.config.tab_width, self.config.semantics.clone()),
                        EditAction::Undo => undo(self, self.config.semantics.clone()),   // TODO: undo takes a long time to undo when whole text deleted. see if this can be improved
                        EditAction::Redo => redo(self, self.config.semantics.clone()),
                        EditAction::AddSurround(l, t) => add_surrounding_pair(self, l, t, self.config.semantics.clone()),
                    };
                    match result{
                        Ok(()) => {
                            //pop_to_insert(self);  //testing to see if this increments EDIT_ACTION_DISPLAY_MODE if selection_out_of_view
                            fn same_mode(mode: Mode, display_mode: DisplayMode) -> bool{
                                //if mode == Mode::Error && display_mode == DisplayMode::Error{true}
                                //else if mode == Mode::Warning && display_mode == DisplayMode::Warning{true}
                                //else if mode == Mode::Notify && display_mode == DisplayMode::Notify{true}
                                //else if mode == Mode::Info && display_mode == DisplayMode::Info{true}
                                //else{false}
                                (mode == Mode::Error && display_mode == DisplayMode::Error) ||
                                (mode == Mode::Warning && display_mode == DisplayMode::Warning) ||
                                (mode == Mode::Notify && display_mode == DisplayMode::Notify) ||
                                (mode == Mode::Info && display_mode == DisplayMode::Info)
                            }
                            if same_mode(self.mode(), EDIT_ACTION_DISPLAY_MODE) 
                            && self.mode_stack.top_message() == Some(EDIT_ACTION_OUT_OF_VIEW.to_string()){
                                /* retain mode as EDIT_ACTION_DISPLAY_MODE */
                            }else{
                                pop_to_insert(self);
                            }
                            //
                            self.checked_scroll_and_update(
                                &self.selections.primary.clone(), 
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
                                //testing to see if this increments EDIT_ACTION_DISPLAY_MODE if selection_out_of_view
                                else{
                                    pop_to_insert(self);
                                }
                                //
                            //
                        }
                        Err(e) => {handle_application_error(self, e);}
                    }
                }
            }
            Action::ViewAction(view_action) => {
                //use crate::utilities::*;
                //possible modes are Insert and View + any mode with fallthrough to insert
                assert!(matches!(self.mode(), Mode::Insert | Mode::View | Mode::Warning | Mode::Notify | Mode::Info));
                let mut should_exit = false;
                let result = match view_action{
                    ViewAction::CenterVerticallyAroundCursor => {
                        should_exit = true;
                        display_area::center_view_vertically_around_cursor(&self.buffer_display_area(), &self.selections.primary, &self.buffer, self.config.semantics.clone())
                    }
                    ViewAction::ScrollUp => {
                        display_area::scroll_view_up(&self.buffer_display_area(), self.config.view_scroll_amount)
                    }
                    ViewAction::ScrollDown => {
                        display_area::scroll_view_down(&self.buffer_display_area(), self.config.view_scroll_amount, &self.buffer)
                    }
                    ViewAction::ScrollLeft => {
                        display_area::scroll_view_left(&self.buffer_display_area(), self.config.view_scroll_amount)
                    }
                    ViewAction::ScrollRight => {
                        display_area::scroll_view_right(&self.buffer_display_area(), self.config.view_scroll_amount, &self.buffer)
                    }
                };
                match result{
                    Ok(view) => {
                        if self.mode() != Mode::View && self.mode() != Mode::Insert{pop_to_insert(self);}
                        let DisplayArea{horizontal_start, vertical_start, width: _width, height: _height} = view;
                        self.buffer_horizontal_start = horizontal_start;
                        self.buffer_vertical_start = vertical_start;
                    
                        self.update_ui_data_document();
                        if self.mode() == Mode::View && should_exit{self.update(Action::EditorAction(EditorAction::ModePop));}
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
                            text_box.buffer.apply_replace(&self.clipboard, &mut text_box.selection, self.config.semantics.clone());
                        }else{
                            text_box.buffer.apply_insert(&self.clipboard, &mut text_box.selection, self.config.semantics.clone());
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
                                                //if let Ok(new_selections) = crate::utilities::clear_non_primary_selections::selections_impl(&self.selections){self.selections = new_selections;}    //intentionally ignoring any errors
                                                if let Ok(new_selections) = selections::clear_non_primary_selections(&self.selections){self.selections = new_selections;}    //intentionally ignoring any errors
                                            }
                                            //match crate::utilities::move_to_line_number::selection_impl(self.selections.primary(), line_number, &self.buffer, crate::selection::Movement::Move, self.config.semantics.clone()){
                                            match selection::move_to_line_number(&self.selections.primary, line_number, &self.buffer, selection::Movement::Move, self.config.semantics.clone()){
                                                Ok(new_selection) => {
                                                    //*self.selections.primary_mut() = new_selection;
                                                    self.selections.primary = new_selection;
                                                    self.checked_scroll_and_update(
                                                        &self.selections.primary.clone(), 
                                                        Application::update_ui_data_selections, 
                                                        Application::update_ui_data_selections
                                                    ); //TODO: pretty sure one of these should be update_ui_data_document
                                                    self.update(Action::EditorAction(EditorAction::ModePop));
                                                    // center view vertically around new primary, if possible
                                                    //if let Ok(new_view) = crate::utilities::center_view_vertically_around_cursor::view_impl(&self.buffer_display_area(), self.selections.primary(), &self.buffer, self.config.semantics.clone()){
                                                    if let Ok(new_view) = display_area::center_view_vertically_around_cursor(&self.buffer_display_area(), &self.selections.primary, &self.buffer, self.config.semantics.clone()){
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
                                let execute_result = execute_command(
                                    self, 
                                    &self.ui.util_bar.utility_widget.text_box.buffer.to_string()
                                );
                                if Result::is_ok(&execute_result){
                                    //only checking command mode because parsed resultant fn may need to enter error/warning/notify/info mode, and user should see that
                                    if self.mode() == Mode::Command{pop_to_insert(self);}
                                }else{
                                    let error = Result::unwrap_err(execute_result);
                                    handle_message(self, DisplayMode::Error, &error);
                                }
                            }
                            //Mode::Find | Mode::Split => self.update(Action::EditorAction(EditorAction::ModePop)),
                            Mode::Find | Mode::Split => {
                                if self.ui.util_bar.utility_widget.text_box.text_is_valid{
                                    self.update(Action::EditorAction(EditorAction::ModePop));
                                }else{
                                    handle_message(self, DisplayMode::Error, "invalid regex");
                                }
                            }
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
                                    &self.selections.primary.clone(), 
                                    Application::update_ui_data_document, 
                                    Application::update_ui_data_selections
                                );
                                self.update(Action::EditorAction(EditorAction::ModePop));
                            }
                            _ => {self.update(Action::EditorAction(EditorAction::ModePop));}
                        }
                        perform_follow_up_behavior = false;
                    }
                    UtilAction::GotoModeSelectionAction(selection_action) => {
                        //TODO?: add go to matching surrounding char(curly, square, paren, single quote, double quote, etc)?
                        assert!(self.mode() == Mode::Goto);
                        if let Ok(count) = self.ui.util_bar.utility_widget.text_box.buffer.to_string().parse::<usize>(){
                            self.update(Action::EditorAction(EditorAction::ModePop));
                            assert!(self.mode() == Mode::Insert);
                            self.update(Action::SelectionAction(selection_action, count));
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
                                    match search_selection(
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
                                &self.selections.primary.clone(), 
                                Application::update_ui_data_document, 
                                Application::update_ui_data_selections
                            );
                        }
                        Mode::Split => {
                            match &self.preserved_selections{
                                Some(selections_before_split) => {
                                    match split_selection(
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
                                &self.selections.primary.clone(), 
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

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>, event_rx: std::sync::mpsc::Receiver<Event>) -> Result<(), String>{
        //eval command from start file, if it exists
        #[cfg(not(test))] match std::fs::metadata(Path::new(START_FILE)){
            Err(_) => {/*path does not exist on file system*/}
            Ok(metadata) => {
                if metadata.is_file(){
                    match std::fs::read_to_string(START_FILE){
                        Err(e) => handle_message(self, DisplayMode::Error, &e.to_string()),
                        Ok(content) => {
                            if let Err(e) = execute_command(self, &content){
                                handle_message(self, DisplayMode::Error, &format!("start file execution failed with error: {e}"));
                            }
                        }
                    }
                }else{/*path is not a file*/}
            }
        }

        //TODO?: maybe handle input/9p threads here?...

        while !self.should_quit{
            //derive User Interface from Application state
            self.layout();  //TODO: does update_layouts always need to be called, or can this be called only from actions that require it?...
            self.render(terminal)?;            
            
            //update Application state
            self.handle_event(&event_rx)?;  //maybe create self.actions: Vec<Action>, and push to this

            //if input_thread_handle.is_finished(){
            //  match input_thread_handle.join(){
            //      Err(e) => {}
            //      Ok(_) => {}
            //  }
            //}

            //for action in self.actions{
                /*self.update()*/               //can push more actions to self.actions
            //}
        }
        Ok(())
    }
}

fn handle_message(app: &mut Application, display_mode: DisplayMode, message: &/*'static */str){ //-> Action
    match display_mode{
        DisplayMode::Error => app.update(Action::EditorAction(EditorAction::ModePush(Mode::Error, Some(message.to_string())))),
        DisplayMode::Warning => app.update(Action::EditorAction(EditorAction::ModePush(Mode::Warning, Some(message.to_string())))),
        DisplayMode::Notify => app.update(Action::EditorAction(EditorAction::ModePush(Mode::Notify, Some(message.to_string())))),
        DisplayMode::Info => app.update(Action::EditorAction(EditorAction::ModePush(Mode::Info, Some(message.to_string())))),
        DisplayMode::Ignore => {/* do nothing */}
    }
}
fn handle_application_error(app: &mut Application, e: ApplicationError){    //-> Action
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
                SelectionsError::NoSearchMatches => handle_message(app, NO_SEARCH_MATCH_DISPLAY_MODE, NO_SEARCH_MATCH),
                SelectionsError::SpansMultipleLines => handle_message(app, SPANS_MULTIPLE_LINES_DISPLAY_MODE, SPANS_MULTIPLE_LINES),
            }
        }
    }
}

/*
    built-in commands vs external programs
    store binaries for editor specific external programs in some associated directory (/edit/bin/<program>)
    append associated directory to PATH in subshell environments (Execute), so subshell can call them directly, without polluting normal use environment
    maybe we would need to write keypresses to an event file, 
        and wait for all subscribers to either write back(indicating unused), then we can handle it as usual
        or consume the keypress and indicate consumed somehow...

    even commands like save could be external programs
        //map keybind to a shell command
        ctrl-s, Mode::Insert -> Action::EditorAction(EditorAction::ExecuteShellCommand("save"))
        //external program impl
            // echo -n /edit/$EDIT_INSTANCE_ID/buffer/content > $EDIT_FILE_PATH

    //should mode logic be external?
    //how would other programs, such as save, query mode state from external mode handling program?...


    rendering:
        ideally, all external gui programs would render themselves, and the window manager would handle placing associated programs together visually
        but for now, i think we will need to implement some way for us to render their visual data by assigning some render area, 
        piping keyboard/mouse events in that area, then displaying their output in that area...
*/

//TODO: replace old kakoune style command handling with acme style command handling
//      behavior not supported by acme should be labeled (not acme)
//  left click -> select text
//  middle click -> execute command
//  right click -> search(Look)(if text looks like a file(foo.c:123), it will send that to plumber, and not search for it within buffer)
//  <command> [argument_1] [argument_2] [etc...]
//  Edit ,s/old/new/g                           - substitute(s) all(g) occurrences of "old" with "new" in whole buffer(,)
//  Get                                         - reload buffer from file on filesystem (what about temp buffers? maybe just ignore? or show diagnostic message "cannot reload a temporary buffer")
//  Look text                                   - equivalent to right click
//  Pipe sort                                   - sends the selected text through "sort" and replaces it with output (prob save this to clipboard, select text, then eval clipboard. not execute this directly...)
//  |sort
//  >command                                    - sends the selected text to command's stdin, but output does not replace selected text (output goes to new window?...)   (apparently equivalent to Pipe > command)
//  <command                                    - runs command with no stdin, replaces selected text with command's output, or inserts output at cursor, if no selection    (apparently equivalent to Pipe < command)
//  !command                                    - runs command with no stdin, does not do anything with command output
//          (not acme) maybe we should enforce that prefixes have a space after? that way we can treat it as a normal command name, and not have to check text.starts_with('|')...
//          | command, > command, < command, ! command, etc...
//          |command would be treated as an un-prefixed shell command, not a built-in. idk what behavior that would cause in the shell...
//  (not acme) set_option <option> <value>      - set config option. we could still support this, but really these would be set through the 9p file interface
//  (not acme) search <regex>
//  (not acme) search_selection <regex>
//  (not acme) split <regex>
//  (not acme) split_selection <regex>
//
//  any unknown command is run as an external program, with nothing sent to stdin(interpreted as shell command) (display output in a new window)
//  ls
//  grep main *.c
//
//
//
// Pipe impl:
//  determine selection: /acme/<id>/addr
//  read selected text: /acme/<id>/data
//  spawn process(via rc shell) and set up pipes(stdin/stdout)
//  send selection -> stdin
//  capture stdout
//  replace selected text: /acme/<id>/data
//
//  prefix  | sends selected text   | replaces selected text    | output to new window  | stderr goes to diagnostic line(not acme)  | built-in command name(not acme)
//  none    | no                    | no                        | yes                   | yes                                       | run
//  |       | yes                   | yes                       | no                    | yes                                       | pipe
//  >       | yes                   | no                        | yes                   | yes                                       | redirect
//  <       | no                    | yes                       | no                    | yes                                       | insert
//  !       | no                    | no                        | no                    | ?                                         | do?
//
struct CommandParser<'a>{rest: &'a str} //TODO: rest: Option<&'a str>
impl<'a> CommandParser<'a> {
    fn new(input: &'a str) -> Self{
        Self{rest: input.trim_start()}
    }
    fn next(&mut self) -> Option<&'a str>{  //TODO: could handle lexing values in "" here...
        if self.rest.is_empty(){
            return None;
        }

        let mut split = self.rest.splitn(2, char::is_whitespace);
        let word = split.next().unwrap();
        self.rest = split.next().unwrap_or("").trim_start();
        Some(word)
    }
    fn rest(&self) -> &'a str{self.rest}    //TODO: maybe should return Option<&'a str>, returning None if self.rest.is_empty()
}

//at the extreme, i think every action could end up being a command
//in that sense, the editor is just a command parser, with command specific response behavior
fn execute_command(app: &mut Application, command: &str) -> Result<(), String>{ //-> Result<Option<Action>, String>?
    //TODO: this should prob be threaded/async
    fn run_shell_command(app: &Application, stdin: Option<String>, command: &str) -> Result<String, String>{
        //TODO: if temp file(has no file path), get terminal current_dir, and pass that as the current dir for command. (and later plumber...)
        let mut environment_variables = std::collections::HashMap::new();
        //environment_variables.insert("MY_VAR", "environment variable content");
        //TODO: maybe options/settings should start with $EDIT_SETTING_
        if command.contains("$EDIT_OPT_SHOW_LINE_NUMBERS"){  //env vars can also be lower case...
            environment_variables.insert("EDIT_OPT_SHOW_LINE_NUMBERS", app.ui.document_viewport.line_number_widget.show.to_string());
        }
        if command.contains("$EDIT_OPT_SHOW_STATUS_BAR"){
            environment_variables.insert("EDIT_OPT_SHOW_STATUS_BAR", app.ui.status_bar.show.to_string());
        }
        //let output = match std::process::Command::new("bash").arg("-c").arg(command).output(){
        //    Err(e) => return Err(format!("{e}")),
        //    Ok(idk) => idk,
        //};
        //let output = std::process::Command::new("sh"/*"bash"*/)
        //    .arg("-c")
        //    .arg(command)
        //    //.env("MY_VAR", "environment variable content")
        //    .envs(&environment_variables)
        //    //.stdout(std::process::Stdio::piped()) //i think this is the default with .output()
        //    //.stderr(std::process::Stdio::piped()) //i think this is the default with .output()
        //    .output()
        //    .expect("failed to execute process");
        let mut child_process = std::process::Command::new(SHELL)
            .arg(SHELL_COMMAND_FLAG)
            .arg(command)
            //.env("MY_VAR", "environment variable content")
            .envs(&environment_variables)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("failed to execute process");
        //TODO: use std::process::Command::new(command)
        //then get PATH env var, and pass it to .env() to call commands directly with resolved paths, skipping "sh" invocation
        if let Some(stdin_string) = stdin{
            if let Some(stdin) = child_process.stdin.as_mut(){
                stdin.write_all(stdin_string.as_bytes()).expect("failed to write to child process' stdin")
            }
        }
        let output = child_process.wait_with_output().expect("failed to execute process");
        if output.status.success(){
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }else{
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    let mut parser = CommandParser::new(command);
    let first = match parser.next(){
        None => return Err(String::from("cannot execute empty command string")),
        Some(first) => first
    };
    match first{
        //"echo" => {       //don't want to use "echo" because it would clash with existing echo program
        "diagnostic" => {   //diagnostic may become an external gui program
            let args = parser.rest();
            let mut parser = CommandParser::new(args);
            let optional_diagnostic_mode = match parser.next(){
                None => return Err(String::from("too few arguments: diagnostic [diagnostic_mode] <message>")),
                Some(mode) => mode
            };
            let (mode, message) = match optional_diagnostic_mode{
                "--error" => (DisplayMode::Error, parser.rest()),
                "--warning" => (DisplayMode::Warning, parser.rest()),
                "--notify" => (DisplayMode::Notify, parser.rest()),
                "--info" => (DisplayMode::Info, parser.rest()),
                _ => (DisplayMode::Info, args) //default to info mode, and display all args as message
            };
            if message.trim().is_empty(){
                return Err(String::from("too few arguments: diagnostic [diagnostic_mode] <message>"));
            }
            handle_message(app, mode, message);
        }

        //"term" | "t" => app.action(Action::EditorAction(EditorAction::OpenNewTerminalWindow)),

        //can currently just: set_option show_line_numbers true|false
        "toggle_line_numbers" | "ln" => app.update(Action::EditorAction(EditorAction::ToggleLineNumbers)),  //these will prob end up using set-option command...

        //can currently just: set_option show_status_bar true|false
        "toggle_status_bar" | "sb" => app.update(Action::EditorAction(EditorAction::ToggleStatusBar)),      //these will prob end up using set-option command...
            
        "quit" | "q" => app.update(Action::EditorAction(EditorAction::Quit)),
        "quit!" | "q!" => app.update(Action::EditorAction(EditorAction::QuitIgnoringChanges)),
        //write buffer contents to file //should this optionally take a filepath to save to? then we don't need to implement save as    //would have to split util bar text on ' ' into separate args
        "write" | "w" => app.update(Action::EditorAction(EditorAction::Save)),

        "search" => {
            let regex = parser.rest();
            if regex.is_empty(){return Err(String::from("too few arguments: search <regex>"));}
            //search <regex>
            match search(regex, &app.buffer, app.config.semantics.clone()){
                Err(_) => return Err(String::from("no matching regex")),
                Ok(new_selections) => {
                    app.selections = new_selections;
                    app.checked_scroll_and_update(
                        &app.selections.primary.clone(), 
                        Application::update_ui_data_document, 
                        Application::update_ui_data_selections
                    );
                }
            }
        }
        "search_selection" => {
            let regex = parser.rest();
            if regex.is_empty(){return Err(String::from("too few arguments: search_selection <regex>"));}
            //search_selection <regex>
            match search_selection(&app.selections, &regex, &app.buffer, app.config.semantics.clone()){
                Err(_) => return Err(String::from("no matching regex")),
                Ok(new_selections) => {
                    app.selections = new_selections;
                    app.checked_scroll_and_update(
                        &app.selections.primary.clone(), 
                        Application::update_ui_data_document, 
                        Application::update_ui_data_selections
                    );
                }
            }
        }
        //"split" => {} //split whole buffer
        "split_selection" => {
            let regex = parser.rest();
            if regex.is_empty(){return Err(String::from("too few arguments: split_selection <regex>"));}
            //split_selection <regex>
            match split_selection(&app.selections, &regex, &app.buffer, app.config.semantics.clone()){
                Err(_) => return Err(String::from("no matching regex")),
                Ok(new_selections) => {
                    app.selections = new_selections;
                    app.checked_scroll_and_update(
                        &app.selections.primary.clone(), 
                        Application::update_ui_data_document, 
                        Application::update_ui_data_selections
                    );
                }
            }
        }
                
        //add_keybind <mode> <keybind> <command>
        //"add_keybind" => {
        //    let mode = Mode::Insert;    //get mode from positional args
        //    let keycode = crossterm::event::KeyCode::Char('n'); //get mode from positional args
        //    let modifiers = crossterm::event::KeyModifiers::CONTROL;    //get mode from positional args
        //    let key_event = crossterm::event::KeyEvent::new(keycode, modifiers);
        //    let _command = "idk some shit".to_string();  //get mode from positional args
        //    if app.config.keybinds.contains_key(&(mode, key_event)){
        //        return Err(String::from("this keybind has already been mapped"))
        //    }else{
        //        //app.config.keybinds.insert((mode, key_event), Action::EditorAction(EditorAction::EvalCommand(command)));
        //        handle_message(app, DisplayMode::Info, "keybind added");
        //    }
        //}
        //remove_keybind <keybind>

        //TODO: bug: when these are called from command mode, a success diagnostic is displayed stacked on top of command mode
        //command mode should exit to insert then display the diagnostic...
        //this will eventually be echo -n <value> > /mnt/edit/<instance_id>/settings/<setting>, when filesystem interface impled
        //"set_option" => {
        "set" => {
            //set_option <name> <value>
            let name = match parser.next(){
                None => return Err(String::from("too few arguments: set <name> <value>")),
                Some(name) => name,
            };
            let value = match parser.next(){
                None => return Err(String::from("too few arguments: set <name> <value>")),
                Some(value) => value,
            };
            if !parser.rest().is_empty(){return Err(String::from("too many arguments: set <name> <value>"));}
            match name{
                //NOTE: may not allow setting cursor semantics for TUI, because terminal cannot currently handle multicursor bar cursor display...
                "cursor_semantics" => { //TODO: maybe return error results in same state if already set to provided value. maybe do that for all options...
                    match value{
                        "Bar" | "bar" => {
                            if app.config.semantics == CursorSemantics::Bar{handle_message(app, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                            else{
                                app.config.semantics = CursorSemantics::Bar;
                                //TODO: change selections from Block to Bar
                                handle_message(app, DisplayMode::Notify, &format!("cursor_semantics set to {}", value));
                            }
                        }
                        "Block" | "block" => {
                            if app.config.semantics == CursorSemantics::Block{handle_message(app, SAME_STATE_DISPLAY_MODE, SAME_STATE);}
                            else{
                                app.config.semantics = CursorSemantics::Block;
                                //TODO: change selections from Bar to Block
                                handle_message(app, DisplayMode::Notify, &format!("cursor_semantics set to {}", value));
                            }
                        }
                        _ => return Err(format!("{} is not a valid value for cursor_semantics", value))
                    }
                }
                "use_full_file_path" => {
                    match value.parse::<bool>(){
                        Err(error) => return Err(format!("{}", error)),
                        Ok(parsed_value) => {
                            app.config.use_full_file_path = parsed_value;
                            if app.config.use_full_file_path{
                                app.ui.status_bar.file_name_widget.text = app.buffer.file_path().unwrap_or_default();
                            }else{
                                app.ui.status_bar.file_name_widget.text = app.buffer.file_name().unwrap_or_default();
                            }
                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                        }
                    }
                }
                "use_hard_tab" => {
                    match value.parse::<bool>(){
                        Err(error) => return Err(format!("{}", error)),
                        Ok(parsed_value) => {
                            app.config.use_hard_tab = parsed_value;
                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                        }
                    }
                }
                "tab_width" => {
                    match value.parse::<usize>(){
                        Err(error) => return Err(format!("{}", error)),
                        Ok(parsed_value) => {
                            app.config.tab_width = parsed_value;
                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                        }
                    }
                }
                "view_scroll_amount" => {
                    match value.parse::<usize>(){
                        Err(error) => return Err(format!("{}", error)),
                        Ok(parsed_value) => {
                            app.config.view_scroll_amount = parsed_value;
                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                        }
                    }
                }
                "show_cursor_column" => {
                    match value.parse::<bool>(){
                        Err(error) => return Err(format!("{}", error)),
                        Ok(parsed_value) => {
                            app.config.show_cursor_column = parsed_value;
                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                        }
                    }
                }
                "show_cursor_line" => {
                    match value.parse::<bool>(){
                        Err(error) => return Err(format!("{}", error)),
                        Ok(parsed_value) => {
                            app.config.show_cursor_line = parsed_value;
                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                        }
                    }
                }
                "show_line_numbers" => {
                    match value.parse::<bool>(){
                        Err(error) => return Err(format!("{}", error)),
                        Ok(parsed_value) => {
                            //TODO?: if app.mode() == Mode::Command{app.pop_to_insert()/*although, this fn is scoped within action()...*/}
                            app.ui.document_viewport.line_number_widget.show = parsed_value;
                            //
                            app.layout();
                            app.update_ui_data_document();
                            //
                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                        }
                    }
                }
                "show_status_bar" => {
                    match value.parse::<bool>(){
                        Err(error) => return Err(format!("{}", error)),
                        Ok(parsed_value) => {
                            //TODO?: if app.mode() == Mode::Command{app.pop_to_insert()/*although, this fn is scoped within action()...*/}
                            app.ui.status_bar.show = parsed_value;
                            //
                            app.layout();
                            app.update_ui_data_document();
                            //
                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                        }
                    }
                }
                _ => return Err(format!("{:?} is not a valid setting", name))
            }
        }
        "|" => {
            let stdin = if app.selections.primary.to_string(&app.buffer).is_empty(){
                None
            }else{
                Some(app.selections.primary.to_string(&app.buffer))
            };
            match run_shell_command(app, stdin, parser.rest()){
                Err(error) => {
                    if error.is_empty(){
                        handle_message(app, DisplayMode::Error, "shell command failed with empty error string");
                    }else{
                        handle_message(app, DisplayMode::Error, &format!("{error}"));
                    }
                }
                Ok(output) => {
                    if output.is_empty(){
                        //TODO: maybe just do the normal replace, which would replace the selection with ""
                        handle_message(app, DisplayMode::Warning, "shell command succeeded with empty output string");
                    }else{
                        //TODO: this works, but i would like to replace selections in one go...
                        for char in output.trim().chars(){
                            app.update(Action::EditAction(EditAction::InsertChar(char)));
                        }
                    }
                }
            }
        }
        ">" => {return Err(String::from("> not yet supported"));}
        "<" => {
            match run_shell_command(app, None, parser.rest()){
                Err(error) => {
                    if error.is_empty(){
                        handle_message(app, DisplayMode::Error, "shell command failed with empty error string");
                    }else{
                        handle_message(app, DisplayMode::Error, &format!("{error}"));
                    }
                }
                Ok(output) => {
                    if output.is_empty(){
                        handle_message(app, DisplayMode::Warning, "shell command succeeded with empty output string");
                    }else{
                        //TODO: this works, but i would like to replace selections in one go...
                        for char in output.trim().chars(){
                            app.update(Action::EditAction(EditAction::InsertChar(char)));
                        }
                    }
                }
            }
        }
        "!" => {return Err(String::from("! not yet supported"));}
        _ => {
            //run anything else as shell command
            match run_shell_command(app, None, command){
                Err(error) => {
                    if error.is_empty(){
                        return Err(String::from("shell command failed with empty error string"));
                    }else{
                        return Err(error.to_string());
                    }
                }
                Ok(output) => {
                    if output.is_empty(){
                        handle_message(app, DisplayMode::Warning, "shell command succeeded with empty output string");
                    }else{
                        //TODO: open new edit window with output in temp buffer, don't display in diagnostic panel
                        handle_message(app, DisplayMode::Info, &format!("unpiped command output \"{}\" will be sent to new window(not yet implemented)", output));
                    }
                }
            }
        }
    }
    Ok(())
}

/* some example commands to test
set_option show_status_bar true
set_option show_line_numbers true
set_option use_full_file_path true
idk
| idk
> idk
< idk
! idk
diagnostic --info idk
diagnostic --notify idk
diagnostic --warning idk
diagnostic --error idk
search idk
search_selection idk
split idk
split_selection idk

< printf "%s" $EDIT_OPT_SHOW_LINE_NUMBERS

| sort
some
shit
idk

//also test from command mode. there seem to be some bad behaviors there...
*/



//TODO: change Find mode to SearchSelection mode, and create Search mode for this full buffer search (ctrl+shift+/)
pub fn search(
    input: &str, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selections, SelectionsError>{
    if input.is_empty(){return Err(SelectionsError::NoSearchMatches);}
    let mut new_selections = Vec::new();
    if let Ok(regex) = regex::Regex::new(input){
        //regex returns byte indices, and the current Selection impl uses char indices...
        for search_match in regex.find_iter(&buffer.to_string()[..]){
            let start_char_index = buffer.byte_to_char(search_match.start());
            let end_char_index = buffer.byte_to_char(search_match.end());
            let new_selection = Selection::new_from_range(
                Range::new(start_char_index, end_char_index), 
                if buffer.next_grapheme_char_index(start_char_index) == end_char_index{None}    //this works for block semantics only...
                else{Some(selection::Direction::Forward)}, 
                buffer, 
                semantics.clone()
            );
            new_selections.push(new_selection);
        }
    }
    if new_selections.is_empty(){Err(SelectionsError::NoSearchMatches)}
    else{
        Ok(Selections::new(new_selections, 0, buffer, semantics))
    }
}
/// # Errors
/// when no matches.
//TODO: this, and related functions, should prob be made to work over just a string slice from a buffer.
//caller should handle slicing according to selection, and this fn should be agnostic to selections...
pub fn search_selection(
    selections: &Selections, 
    input: &str, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selections, SelectionsError>{
    if input.is_empty(){return Err(SelectionsError::NoSearchMatches);}
    let mut new_selections = Vec::new();
    let mut num_pushed: usize = 0;
    let primary_selection = &selections.primary;
    //let mut primary_selection_index = self.primary_selection_index;
    let mut primary_selection_index = 0;
    
    for selection in &selections.flatten(){  //self.selections.iter(){   //change suggested by clippy lint
        //let matches = incremental_search_in_selection(selection, input, buffer);
        let matches = {
            let mut match_selections = Vec::new();
            let start = selection.range.start;
            if let Ok(regex) = regex::Regex::new(input){
                //regex returns byte indices, and the current Selection impl uses char indices...
                for search_match in regex.find_iter(&buffer.to_string()[start..selection.range.end.min(buffer.len_chars())]){
                    let mut new_selection = selection.clone();
                    new_selection.range.start = buffer.byte_to_char(search_match.start()).saturating_add(start);
                    new_selection.range.end = buffer.byte_to_char(search_match.end()).saturating_add(start);
                    new_selection.extension_direction = if buffer.next_grapheme_char_index(new_selection.range.start) == new_selection.range.end{None}
                    else{Some(selection::Direction::Forward)};
                    match_selections.push(new_selection);
                }
            }
            //else{/*return error FailedToParseRegex*/} //no match found if regex parse fails
            match_selections  //if selections empty, no match found
        };
        if selection == primary_selection{
            primary_selection_index = num_pushed.saturating_sub(1);
        }
        for search_match in matches{
            new_selections.push(search_match);
            num_pushed = num_pushed + 1;
        }
    }

    if new_selections.is_empty(){Err(SelectionsError::NoSearchMatches)}
    else{
        Ok(Selections::new(new_selections, primary_selection_index, buffer, semantics))
    }
}
//TODO: impl tests
pub fn split_selection(
    selections: &Selections, 
    input: &str, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selections, SelectionsError>{
    if input.is_empty(){return Err(SelectionsError::NoSearchMatches);}
    let mut new_selections = Vec::new();
    let mut num_pushed: usize = 0;
    let primary_selection = &selections.primary;
    let mut primary_selection_index = 0;
    
    for selection in &selections.flatten(){
        let matches = {
            let mut match_selections = Vec::new();
            if let Ok(regex) = regex::Regex::new(input){
                let mut start = selection.range.start; //0;
                let mut found_split = false;
                // Iter over each split, and push the retained selection before it, if any...       TODO: test split at start of selection
                for split in regex.find_iter(&buffer./*inner.*/to_string()[selection.range.start..selection.range.end.min(buffer.len_chars())]){
                    found_split = true;
                    let selection_range = Range::new(start, split.start().saturating_add(selection.range.start));
                    if selection_range.start < selection_range.end{
                        let mut new_selection = selection.clone();
                        new_selection.range.start = selection_range.start;
                        new_selection.range.end = selection_range.end;
                        //new_selection.extension_direction = Some(Direction::Forward);
                        new_selection.extension_direction = if buffer.next_grapheme_char_index(new_selection.range.start) == new_selection.range.end{None}
                        else{Some(selection::Direction::Forward)};
                        match_selections.push(new_selection);
                    }
                    start = split.end().saturating_add(selection.range.start);
                }
                // Handle any remaining text after the last split
                //if split found and end of last split < selection end
                if found_split && start < selection.range.end.min(buffer.len_chars()){
                    let mut new_selection = selection.clone();
                    new_selection.range.start = start;
                    new_selection.range.end = selection.range.end.min(buffer.len_chars());
                    new_selection.extension_direction = if buffer.next_grapheme_char_index(new_selection.range.start) == new_selection.range.end{None}
                    else{Some(selection::Direction::Forward)};
                    match_selections.push(new_selection);
                }
            }
            match_selections
        };
        if matches.is_empty(){
            if selections.count() == 1{return Err(SelectionsError::NoSearchMatches);}
            if selection == primary_selection{
                primary_selection_index = num_pushed.saturating_sub(1);
            }
            new_selections.push(selection.clone());
            num_pushed = num_pushed + 1;
        }
        else{
            if selection == primary_selection{
                primary_selection_index = num_pushed.saturating_sub(1);
            }
            for search_match in matches{
                new_selections.push(search_match);
                num_pushed = num_pushed + 1;
            }
        }
    }

    let new_selections = Selections::new(new_selections, primary_selection_index, buffer, semantics.clone());
    if new_selections == *selections{return Err(SelectionsError::ResultsInSameState);}

    Ok(new_selections)
}
//TODO: also test Bar semantics
#[cfg(test)]
mod search_tests{
    use crate::{
        selection::{Selection, Direction, CursorSemantics},
        selections::Selections,
        range::Range,
        buffer::Buffer,
        application::search_selection,
        application::split_selection,
        application::search,
    };

    //search
    #[test] fn test_search(){
        let input = "idk";
        //                01234567890123456
        let text = "idk some shit idk";
        let buffer = Buffer::new(text, None, true);
        let semantics = CursorSemantics::Block;
        assert_eq!(
            Selections::new(
                vec![
                    Selection::new_unchecked(Range::new(0, 0+input.chars().count()), Some(Direction::Forward), None),
                    Selection::new_unchecked(Range::new(14, 14+input.chars().count()), Some(Direction::Forward), None)
                ], 
                0, 
                &buffer, 
                semantics.clone()
            ),
            search(input, &buffer, semantics).unwrap()
        );
    }

    //search_selection
    #[test] fn search_hard_tab(){
        let buffer_text = "\tidk\nsome\nshit\n";
        let buffer = Buffer::new(buffer_text, None, false);
        let semantics = CursorSemantics::Block;
        let selection = Selection::new_unchecked(Range::new(0, buffer.chars().count()), Some(Direction::Forward), None);
        let selections = Selections::new(vec![selection], 0, &buffer, semantics.clone());
        let expected_selections = vec![
            //Selection::new_unchecked(Range::new(0, 1), None, None)
            Selection::new_unchecked(Range::new(0, "\t".chars().count()), None, None)
        ];
        let expected_selections = Selections::new(expected_selections, 0, &buffer, semantics.clone());
        assert_eq!(expected_selections, search_selection(&selections, "\t", &buffer, semantics).unwrap());
    }

    #[test] fn search_multibyte_grapheme(){
        let buffer_text = "a̐éö̲\r\n";
        let buffer = Buffer::new(buffer_text, None, false);
        let semantics = CursorSemantics::Block;
        let selection = Selection::new_unchecked(Range::new(0, buffer_text.chars().count()), Some(Direction::Forward), None);
        let selections = Selections::new(vec![selection], 0, &buffer, semantics.clone());
        let expected_selections = vec![
            //Selection::new_unchecked(Range::new(0, 2), None, None)    //a̐ is 2 chars(unicode code points)
            Selection::new_unchecked(Range::new(0, "a̐".chars().count()), None, None)
        ];
        let expected_selections = Selections::new(expected_selections, 0, &buffer, semantics.clone());
        assert_eq!(expected_selections, search_selection(&selections, "a̐", &buffer, semantics).unwrap());
    }

    //TODO: impl tests for split_selection
}



use crate::{history::{Change, Operation}};
/// Inserts provided string into text at each selection.
pub fn insert_string(app: &mut Application, string: &str, use_hard_tab: bool, tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    //TODO: string lengths need to use char count, not length in bytes
    fn handle_insert_replace(app: &mut Application, current_selection_index: usize, semantics: CursorSemantics, new_text: &str) -> Change{
        use std::cmp::Ordering;
        let selection = app.selections.nth_mut(current_selection_index);
        //let change = Application::apply_replace(&mut app.buffer, new_text, selection, semantics);
        let change = app.buffer.apply_replace(new_text, selection, semantics);
        if let Operation::Replace{replacement_text} = change.inverse(){
            //match replacement_text.len().cmp(&new_text.len()){    //old selected text vs new text
            match replacement_text.chars().count().cmp(&new_text.chars().count()){
                Ordering::Greater => {
                    app.selections.shift_subsequent_selections_backward(
                        current_selection_index, 
                        //replacement_text.len().saturating_sub(new_text.len())
                        replacement_text.chars().count().saturating_sub(new_text.chars().count())
                    );
                }
                Ordering::Less => {
                    app.selections.shift_subsequent_selections_forward(
                        current_selection_index, 
                        //new_text.len().saturating_sub(replacement_text.len())
                        new_text.chars().count().saturating_sub(replacement_text.chars().count())
                    );
                }
                Ordering::Equal => {}   // no change to subsequent selections
            }
        }
        change
    }
    //TODO: string lengths need to use char count, not length in bytes
    fn handle_insert(app: &mut Application, string: &str, current_selection_index: usize, semantics: CursorSemantics) -> Change{
        let selection = app.selections.nth_mut(current_selection_index);
        //let change = Application::apply_insert(&mut app.buffer, string, selection, semantics);
        let change = app.buffer.apply_insert(string, selection, semantics);
        //app.selections.shift_subsequent_selections_forward(current_selection_index, string.len());
        app.selections.shift_subsequent_selections_forward(current_selection_index, string.chars().count());
        change
    }
    if app.buffer.read_only{return Err(ApplicationError::ReadOnlyBuffer);}
    if string.is_empty(){return Err(ApplicationError::InvalidInput);}
    
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();

    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        let change = match string{
            //"\n" => {}    //handle behavior specific to pressing "enter". auto-indent, etc... //TODO: create tests for newline behavior...
            "\t" => {   //handle behavior specific to pressing "tab".
                if use_hard_tab{
                    if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), "\t")}
                    else{handle_insert(app, "\t", i, semantics.clone())}
                }
                else{
                    let tab_distance = app.buffer.distance_to_next_multiple_of_tab_width(selection, semantics.clone(), tab_width);
                    let modified_tab_width = if tab_distance > 0 && tab_distance < tab_width{tab_distance}else{tab_width};
                    let soft_tab = " ".repeat(modified_tab_width);

                    if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), &soft_tab)}
                    else{handle_insert(app, &soft_tab, i, semantics.clone())}
                }
            }
            //handle any other inserted string
            _ => {
                if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), string)}
                else{handle_insert(app, string, i, semantics.clone())}
            }
        };

        changes.push(change);
    }

    // push change set to undo stack
    app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

    // clear redo stack. new actions invalidate the redo history
    app.redo_stack.clear();

    Ok(())
}

//TODO: can this function and backspace be combined?...
/// Deletes text inside each [`Selection`] in [`Selections`], or if [`Selection`] not extended, the next character, and pushes changes to undo stack.
pub fn delete(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();
    let mut cannot_delete = false;
    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        //handles cursor at doc end
        if selection.anchor() == app.buffer.len_chars() && selection.cursor(&app.buffer, semantics.clone()) == app.buffer.len_chars(){
            cannot_delete = true; //don't modify text buffer here...
            let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
            changes.push(change);
        }
        else{   //apply the delete
            //let change = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
            let change = app.buffer.apply_delete(selection, semantics.clone());
            if let Operation::Insert{inserted_text} = change.inverse(){
                //app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
                app.selections.shift_subsequent_selections_backward(i, inserted_text.chars().count());
            }
            changes.push(change);
        }
    }

    if app.selections.count() == 1 && cannot_delete{return Err(ApplicationError::SelectionAtDocBounds);}
    else{
        // push change set to undo stack
        app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

        // clear redo stack. new actions invalidate the redo history
        app.redo_stack.clear();
    }

    Ok(())
}

//TODO: combine backspace with delete (make delete take a direction::Forward/Backward)
/// Deletes the previous character, or deletes selection if extended.
/// #### Invariants:
/// - will not delete past start of doc
/// - at start of line, appends current line to end of previous line
/// - removes previous soft tab, if `TAB_WIDTH` spaces are before cursor
/// - deletes selection if selection extended
pub fn backspace(app: &mut Application, _use_hard_tab: bool, _tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::with_capacity(app.selections.count());
    let mut cannot_delete = false;

    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        if selection.is_extended(){
            let change = app.buffer.apply_delete(selection, semantics.clone());
            if let Operation::Insert{inserted_text} = change.inverse(){
                //app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
                app.selections.shift_subsequent_selections_backward(i, inserted_text.chars().count());
            }
            changes.push(change);
        }else{
            if selection.anchor() == 0 && selection.cursor(&app.buffer, semantics.clone()) == 0{
                cannot_delete = true; //don't modify text buffer here...
                let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
                changes.push(change);
            }
            else{
                //let offset_from_line_start = app.buffer.offset_from_line_start(selection.cursor(&app.buffer, semantics.clone()));
                    //let line = app.buffer.inner.line(app.buffer.char_to_line(selection.cursor(&app.buffer, semantics.clone())));
                //let is_deletable_soft_tab = !use_hard_tab 
                //                                && offset_from_line_start >= tab_width
                //                                // handles case where user adds a space after a tab, and wants to delete only the space
                //                                && offset_from_line_start % tab_width == 0
                //                                // if previous tab_width chars are spaces, delete tab_width. otherwise, use default behavior
                //                                && app.buffer.slice_is_all_spaces(
                //                                    offset_from_line_start.saturating_sub(tab_width), 
                //                                    offset_from_line_start
                //                                );

                //NOTE: commenting this until i have time to get it working correctly again...
                //if is_deletable_soft_tab{
                //    selection.shift_and_extend(tab_width, &app.buffer, semantics.clone());
                //    changes.push(app.buffer.apply_delete(selection, semantics.clone()));
                //    app.selections.shift_subsequent_selections_backward(i, tab_width);
                //}
                //else{
                    //if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(selection, 1, &app.buffer, None, semantics.clone()){
                    if let Ok(new_selection) = selection::move_cursor_left(selection, 1, &app.buffer, None, semantics.clone()){
                        *selection = new_selection;
                    }   //TODO: handle error    //first for loop guarantees no selection is at doc bounds, so this should be ok to ignore...
                    changes.push(app.buffer.apply_delete(selection, semantics.clone()));
                    app.selections.shift_subsequent_selections_backward(i, 1);
                //}
            }
        }
    }

    if app.selections.count() == 1 && cannot_delete{return Err(ApplicationError::SelectionAtDocBounds);}
    else{
        // push changes to undo stack
        app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

        // clear redo stack. new actions invalidate the redo history
        app.redo_stack.clear();
    }

    Ok(())
}

/// Cut single selection.
/// Copies text to clipboard and removes selected text from document.
/// Ensure single selection when calling this function.
pub fn cut(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    if app.selections.count() > 1{
        return Err(
            ApplicationError::SelectionsError(
                //crate::selections::SelectionsError::MultipleSelections
                SelectionsError::MultipleSelections
            )
        )
    }

    //let selection = app.selections.primary_mut();
    let selection = &app.selections.primary;
    // Copy the selected text to the clipboard
    app.clipboard = app.buffer.slice(selection.range.start, selection.range.end).to_string();
    delete(app, semantics)   //notice this is returning the result from delete
}

/// Insert clipboard contents at cursor position(s).
pub fn paste(app: &mut Application, use_hard_tab: bool, tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    insert_string(app, &app.clipboard.clone(), use_hard_tab, tab_width, semantics)
}

use std::cmp::Ordering;
/// Reverts the last set of changes made to the document.
pub fn undo(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    // Check if there is something to undo
    if let Some(change_set) = app.undo_stack.pop(){
        let changes = change_set.changes();
        
        app.selections = change_set.clone().selections_after_changes();    //set selections to selections_after_changes to account for any selection movements that may have occurred since edit
        assert!(app.selections.count() == changes.len());

        for (i, change) in changes.iter().enumerate().take(app.selections.count()){
            let selection = app.selections.nth_mut(i);
            match change.operation(){
                Operation::Insert{inserted_text} => {
                    //selection.shift_and_extend(inserted_text.len(), &app.buffer, semantics.clone());
                    selection.shift_and_extend(inserted_text.chars().count(), &app.buffer, semantics.clone());
                    //let _ = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
                    let _ = app.buffer.apply_delete(selection, semantics.clone());
                    //app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
                    app.selections.shift_subsequent_selections_backward(i, inserted_text.chars().count());
                }
                Operation::Delete => {
                    if let Operation::Insert{inserted_text} = change.inverse(){
                        //let _ = Application::apply_insert(&mut app.buffer, &inserted_text, selection, semantics.clone());   //apply inverse operation
                        let _ = app.buffer.apply_insert(&inserted_text, selection, semantics.clone());  //apply inverse operation
                        //app.selections.shift_subsequent_selections_forward(i, inserted_text.len());
                        app.selections.shift_subsequent_selections_forward(i, inserted_text.chars().count());
                    }
                }
                Operation::Replace{replacement_text} => {
                    let inserted_text = replacement_text;
                    if let Operation::Replace{replacement_text} = change.inverse(){
                        //selection.shift_and_extend(inserted_text.len(), &app.buffer, semantics.clone());
                        selection.shift_and_extend(inserted_text.chars().count(), &app.buffer, semantics.clone());
                        //let _ = Application::apply_replace(&mut app.buffer, &replacement_text, selection, semantics.clone());
                        let _ = app.buffer.apply_replace(&replacement_text, selection, semantics.clone());
                        //match inserted_text.len().cmp(&replacement_text.len()){    //old selected text vs new text
                        match inserted_text.chars().count().cmp(&replacement_text.chars().count()){
                            Ordering::Greater => {
                                //app.selections.shift_subsequent_selections_backward(i, inserted_text.len().saturating_sub(replacement_text.len()));
                                app.selections.shift_subsequent_selections_backward(
                                    i, 
                                    inserted_text.chars().count().saturating_sub(replacement_text.chars().count())
                                );
                            }
                            Ordering::Less => {
                                //app.selections.shift_subsequent_selections_forward(i, replacement_text.len().saturating_sub(inserted_text.len()));
                                app.selections.shift_subsequent_selections_forward(
                                    i, 
                                    replacement_text.chars().count().saturating_sub(inserted_text.chars().count())
                                );
                            }
                            Ordering::Equal => {}   // no change to subsequent selections
                        }
                    }
                }
                Operation::NoOp => {}
            }
        }
        // selections should be the same as they were before changes were made, because we are restoring that previous state
        app.selections = change_set.selections_before_changes();

        // Push inverted changes onto redo stack
        app.redo_stack.push(change_set);

        Ok(())
    }else{Err(ApplicationError::NoChangesToUndo)}
}
//#[cfg(test)]
//mod undo_tests{
//    use crate::utilities::undo;
//    use crate::{
//        application::Application,
//        selections::Selections,
//        selection::{Selection, CursorSemantics, ExtensionDirection},
//        view::View,
//        history::{Change, ChangeSet, Operation},
//        range::Range,
//        buffer::Buffer
//    };
//
//    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, undo_stack: Vec<ChangeSet>, expected_text: &str, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
//        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
//
//        let expected_buffer = crate::buffer::Buffer::new(expected_text, None, false);
//        let mut vec_expected_selections = Vec::new();
//        for tuple in tuple_expected_selections{
//            vec_expected_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &expected_buffer, semantics.clone()));
//        }
//        let expected_selections = Selections::new(vec_expected_selections, expected_primary, &expected_buffer, semantics.clone());
//        
//        let mut vec_selections = Vec::new();
//        for tuple in tuple_selections{
//            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
//        
//        app.selections = selections;
//        app.undo_stack = undo_stack;
//        
//        let result = undo::application_impl(&mut app, semantics);
//        assert!(!result.is_err());
//        
//        assert_eq!(expected_buffer, app.buffer);
//        assert_eq!(expected_selections, app.selections);
//        //println!("expected: {:?}\ngot: {:?}", expected_buffer, app.buffer);
//        //assert!(app.buffer.is_modified());    //is modified doesn't work with tests, because it now checks against a persistent file, which tests don't have
//    }
//    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, undo_stack: Vec<ChangeSet>){
//        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
//        
//        let mut vec_selections = Vec::new();
//        for tuple in tuple_selections{
//            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
//        
//        app.selections = selections;
//        app.undo_stack = undo_stack;
//        
//        assert!(undo::application_impl(&mut app, semantics).is_err());
//        assert!(!app.buffer.is_modified());
//    }
//
//    #[test] fn with_insert_change_on_stack(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (9, 10, None)
//            ], 0, 
//            vec![
//                //TODO: figure out how to move this changeset setup into the test fn...
//                ChangeSet::new(
//                    vec![
//                        Change::new(
//                            Operation::Insert{inserted_text: "some\n".to_string()}, 
//                            Selection::new(Range::new(4, 5), ExtensionDirection::Forward), 
//                            Selection::new(Range::new(9, 10), ExtensionDirection::Forward), 
//                            Operation::Delete
//                        )
//                    ], 
//                    Selections::new(
//                        vec![
//                            Selection::new(
//                                Range::new(4, 5), 
//                                ExtensionDirection::Forward
//                            )
//                        ], 
//                        0, 
//                        //&Rope::from("idk\nshit\n"), 
//                        &Buffer::new("idk\nshit\n", None, false),
//                        CursorSemantics::Block
//                    ), 
//                    Selections::new(
//                        vec![
//                            Selection::new(
//                                Range::new(9, 10), 
//                                ExtensionDirection::Forward
//                            )
//                        ], 
//                        0, 
//                        //&Rope::from("idk\nsome\nshit\n"), 
//                        &Buffer::new("idk\nsome\nshit\n", None, false),
//                        CursorSemantics::Block
//                    )
//                )
//            ], 
//            "idk\nshit\n", 
//            //"idk\nshit\n", 
//            vec![
//                (4, 5, None)
//            ], 0
//        );
//    }
//
//    //TODO: test with delete_change_on_stack
//    //TODO: test with replace change on stack
//    //TODO: test with no_op change on stack
//
//    //TODO: test with multiple selections/changes
//
//    #[test] fn undo_with_nothing_on_stack_errors(){
//        //test_error(CursorSemantics::Block);
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 1, None)
//            ], 
//            0, 
//            Vec::new()
//        );
//        //test_error(CursorSemantics::Bar);
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 0, None)
//            ], 
//            0, 
//            Vec::new()
//        );
//    }
//}

/// Re-applies the last undone changes to the document.
// Make sure to clear the redo stack in every edit fn. new actions invalidate the redo history
pub fn redo(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    // Check if there is something to redo
    if let Some(change_set) = app.redo_stack.pop(){
        let changes = change_set.changes();

        app.selections = change_set.clone().selections_before_changes();    //set selections to selections_before_changes to account for any selection movements that may have occurred since undo
        assert!(app.selections.count() == changes.len());   //num selections should match num changes

        for (i, change) in changes.iter().enumerate().take(app.selections.count()){
            let selection = app.selections.nth_mut(i);
            match change.operation(){
                Operation::Insert{inserted_text} => {
                    //let _ = Application::apply_insert(&mut app.buffer, &inserted_text, selection, semantics.clone());
                    let _ = app.buffer.apply_insert(&inserted_text, selection, semantics.clone());
                    //app.selections.shift_subsequent_selections_forward(i, inserted_text.len());
                    app.selections.shift_subsequent_selections_forward(i, inserted_text.chars().count());
                }
                Operation::Delete => {
                    *selection = change.selection_before_change();
                    //let change = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
                    let change = app.buffer.apply_delete(selection, semantics.clone());
                    if let Operation::Insert{inserted_text} = change.inverse(){
                        //app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
                        app.selections.shift_subsequent_selections_backward(i, inserted_text.chars().count());
                    }
                }
                Operation::Replace{replacement_text} => {
                    let inserted_text = replacement_text;
                    //let change = Application::apply_replace(&mut app.buffer, &inserted_text, selection, semantics.clone());
                    let change = app.buffer.apply_replace(&inserted_text, selection, semantics.clone());
                    if let Operation::Replace{replacement_text} = change.inverse(){   //destructure to get currently selected text
                        //match replacement_text.len().cmp(&inserted_text.len()){    //old selected text vs new text
                        match replacement_text.chars().count().cmp(&inserted_text.chars().count()){
                            Ordering::Greater => {
                                //app.selections.shift_subsequent_selections_backward(i, replacement_text.len().saturating_sub(inserted_text.len()));
                                app.selections.shift_subsequent_selections_backward(
                                    i, 
                                    replacement_text.chars().count().saturating_sub(inserted_text.chars().count())
                                );
                            }
                            Ordering::Less => {
                                //app.selections.shift_subsequent_selections_forward(i, inserted_text.len().saturating_sub(replacement_text.len()));
                                app.selections.shift_subsequent_selections_forward(
                                    i, 
                                    inserted_text.chars().count().saturating_sub(replacement_text.chars().count())
                                );
                            }
                            Ordering::Equal => {}   // no change to subsequent selections
                        }
                    }
                }
                Operation::NoOp => {}
            }
        }
        assert!(app.selections == change_set.clone().selections_after_changes());

        // Push changes back onto the undo stack
        app.undo_stack.push(change_set);

        Ok(())
    }else{Err(ApplicationError::NoChangesToRedo)}
}

//TODO: i think all edit actions + apply replace/insert/delete should prob be made purely functional...
//had to make the following public
    //Document.text
    //Document.selections
    //Document.undo_stack
    //Document.redo_stack
    //Document::apply_replace
//is this easing of encapsulation acceptable?...
pub fn add_surrounding_pair(app: &mut Application, leading_char: char, trailing_char: char, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();
    let mut cannot_add_surrounding_pair = false;  //to handle cursor at doc end...
    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        //handles cursor at doc end
        if selection.anchor() == app.buffer.len_chars() && selection.cursor(&app.buffer, semantics.clone()) == app.buffer.len_chars(){
            cannot_add_surrounding_pair = true; //don't modify text buffer here...
            let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
            changes.push(change);
        }
        else{   //replace each selection with its text contents + leading and trailing char added
            //let mut contents = selection.contents_as_string(&document.text);
            let mut contents = selection.to_string(&app.buffer);
            contents.insert(0, leading_char);
            contents.push(trailing_char);
            //let change = Application::apply_replace(&mut app.buffer, &contents, selection, CursorSemantics::Block);
            let change = app.buffer.apply_replace(&contents, selection, semantics.clone());
            changes.push(change);
            app.selections.shift_subsequent_selections_forward(i, 2);  //TODO: could this be handled inside apply_replace and similar functions?...
        }
    }

    if app.selections.count() == 1 && cannot_add_surrounding_pair{return Err(ApplicationError::InvalidInput);}
    else{
        // push change set to undo stack
        app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));
    
        // clear redo stack. new actions invalidate the redo history
        app.redo_stack.clear();
    }
    
    Ok(())
}

//TODO:
//swap selected text with line above
//swap selected text with line below
//align selected text vertically
//rotate text between selections




/// Copy single selection to clipboard.
/// Ensure single selection when calling this function.
pub fn copy(app: &mut Application) -> Result<(), ApplicationError>{
    if app.selections.count() > 1{return Err(ApplicationError::SelectionsError(SelectionsError::MultipleSelections));}
    
    let selection = app.selections.primary.clone();
    // Copy the selected text to the clipboard
    app.clipboard = app.buffer.slice(selection.range.start, selection.range.end).to_string();

    Ok(())
}
//#[cfg(test)]
//mod copy_tests{
//    use crate::utilities::copy;
//    use crate::{
//        application::Application,
//        selections::Selections,
//        selection::{Selection, CursorSemantics},
//        display_area::DisplayArea,
//    };
//
//    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
//    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, expected_clipboard: &str){
//        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
//        
//        let mut vec_selections = Vec::new();
//        for tuple in tuple_selections{
//            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
//        
//        app.selections = selections;
//        
//        let result = copy::application_impl(&mut app);
//        assert!(!result.is_err());
//        
//        assert_eq!(expected_clipboard, app.clipboard);
//        assert!(!app.buffer.is_modified());
//    }
//    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
//        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
//        
//        let mut vec_selections = Vec::new();
//        for tuple in tuple_selections{
//            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
//        
//        app.selections = selections;
//        
//        assert!(copy::application_impl(&mut app).is_err());
//        assert!(!app.buffer.is_modified());
//    }
//
//    //TODO: copy with no selection extension
//        //should fail with bar semantics?...
//        //should copy single char with block semantics
//
//    #[test] fn copy_with_selection_direction_forward_block_semantics(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (4, 9, None)
//            ], 0, 
//            "some\n"
//        );
//    }
//    #[test] fn copy_with_selection_direction_forward_bar_semantics(){
//        test(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (4, 9, None)
//            ], 0, 
//            "some\n"
//        );
//    }
//
//    #[test] fn copy_with_selection_direction_backward_block_semantics(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (9, 4, None)
//            ], 0, 
//            "some\n"
//        );
//    }
//    #[test] fn copy_with_selection_direction_backward_bar_semantics(){
//        test(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (9, 4, None)
//            ], 0, 
//            "some\n"
//        );
//    }
//
//    #[test] fn copy_with_multiple_selections_should_error(){
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 1, None),
//                (4, 5, None)
//            ], 0
//        );
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 0, None),
//                (4, 4, None)
//            ], 0
//        );
//    }
//}



use std::fs;
use std::io::BufWriter;
/// Saves the document's content to its file path.
pub fn save(app: &mut Application) -> Result<(), /*Box<dyn Error>*/String>{
    //if let Some(path) = &app.buffer.file_path{ // does nothing if path is None    //maybe return Err(()) instead?
    //    //app.buffer./*inner.*/write_to(BufWriter::new(fs::File::create(path)?))?;
    //    if app.buffer.is_modified(){
    //        app.buffer.write_to(BufWriter::new(fs::File::create(path)?))?;
    //    }else{
    //        //do nothing. we are already synched with file state.   //maybe return a same state error
    //    }
    //}
    //else{
    //    //return ApplicationError
    //}
    match &app.buffer.file_path{
        None => return Err(String::from("cannot save unnamed buffer")),
        Some(path) => {
            if app.buffer.read_only{return Err(String::from(crate::config::READ_ONLY_BUFFER));}
            //
            else if path.is_dir(){return Err(String::from("cannot save buffer text to directory"))}
            //
            else{
                if app.buffer.is_modified(){
                    match fs::File::create(path){
                        Err(e) => return Err(format!("{e}")),
                        Ok(file) => {
                            if let Err(e) = app.buffer.write_to(BufWriter::new(file)){
                                return Err(format!("{e}"));
                            }
                        }
                    }
                }else{
                    return Err(String::from(crate::config::SAME_STATE));
                }
            }
        }
    }
    
    Ok(())
}



/*
TODO: research plan9port plumber utility and figure out how it can work with single edit instance, multiple edit instances, and/or single edit server instance
i think instances would read from plumb/edit and wait for plumbed messages
but how can we determine which instance should handle that message, and if relevant to none, can we have some fallback behavior to spawn a new instance?...maybe in a plumb rule?...
*/

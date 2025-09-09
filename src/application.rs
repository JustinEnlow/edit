// insert <text>
    //hooks.run(InsertTextPre)  //hook just before text insertion
        //if text is not the same word type as previous changes in changeset || if selection count has changed
            //push pending changeset to history
            //let mut pending_changeset = ChangeSet::new(); //create new changeset
        //else
            //following steps should append existing pending changeset
    //for selection in selections{
        //insert text at/replacing selection (depends on selection extension)
        //hooks.run(InsertText)
            //if text.len() > 1 //extend selection to encompass text (extension direction could be input language dependent(like arabic could be backwards))
            //if text.len() == 1 //move cursor (movement direction could be input language dependent(like arabic could be backwards))
            //update subsequent selection positions to reflect new changes
            //add change to pending changeset (figure out how to group related subsequent changes(like type each char in a word) in to one single changeset)
    //}
    //if selections have changed run hooks for SelectionsModified
    //hooks.run(InsertTextPost) //hook just after text insertion
        //push changeset to history

use std::path::PathBuf;
use crossterm::event;
use ratatui::{
    prelude::*,
    widgets::*
};
//use unicode_segmentation::UnicodeSegmentation;
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
                match event{
                    event::Event::Key(key_event) => {
                        self.action(
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
                    event::Event::Mouse(_mouse_event) => self.action(Action::EditorAction(EditorAction::NoOpEvent)),
                    event::Event::Resize(width, height) => {
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
                    event::Event::FocusLost => self.action(Action::EditorAction(EditorAction::NoOpEvent)), //maybe quit displaying cursor(s)/selection(s)?...
                    event::Event::FocusGained => self.action(Action::EditorAction(EditorAction::NoOpEvent)),   //display cursor(s)/selection(s)?...
                    event::Event::Paste(_) => self.action(Action::EditorAction(EditorAction::NoOpEvent))
                }
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
                        if self.mode() != Mode::Insert{pop_to_insert(self);}
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
                    //could become a command: evaluate_command %val{selection}
                    EditorAction::EvaluateSelectionAsCommand => {
                        if self.mode() != Mode::Insert{pop_to_insert(self);}    //handle insert fallthrough
                        //TODO: figure out best way to handle multiple selections...
                        if self.selections.count() > 1{
                            handle_application_error(self, ApplicationError::SelectionsError(SelectionsError::MultipleSelections));
                        }else{
                            let parse_result = parse_command(self.selections.primary().to_string(&self.buffer));
                            if Result::is_ok(&parse_result){
                                let commands = Result::unwrap(parse_result);
                                let execute_result = execute_commands(self, commands);
                                if Result::is_err(&execute_result){
                                    let error = Result::unwrap_err(execute_result);
                                    handle_message(self, DisplayMode::Error, &error);
                                }
                            }else{
                                //handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, COMMAND_PARSE_FAILED);
                                let error = Result::unwrap_err(parse_result);
                                handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, &error);
                            }
                        }
                    }
                    //this, in combination with copy, is the keyboard centric version of plan9's acme's 2-1 mouse chording
                    //evaluate_command %val{clipboard}
                    EditorAction::EvaluateClipboardAsCommand => {
                        if self.mode() != Mode::Insert{pop_to_insert(self);}    //handle insert fallthrough
                        let parse_result = parse_command(self.clipboard.clone());
                        if Result::is_ok(&parse_result){
                            let commands = Result::unwrap(parse_result);
                            let execute_result = execute_commands(self, commands);
                            if Result::is_err(&execute_result){
                                let error = Result::unwrap_err(execute_result);
                                handle_message(self, DisplayMode::Error, &error);
                            }
                        }else{
                            //handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, COMMAND_PARSE_FAILED);
                            let error = Result::unwrap_err(parse_result);
                            handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, &error);
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
                    
                        //pop_to_insert(self);  //testing to see if this increments SELECTION_ACTION_DISPLAY_MODE if selection_out_of_view
                        fn same_mode(mode: Mode, display_mode: DisplayMode) -> bool{
                            if mode == Mode::Error && display_mode == DisplayMode::Error{true}
                            else if mode == Mode::Warning && display_mode == DisplayMode::Warning{true}
                            else if mode == Mode::Notify && display_mode == DisplayMode::Notify{true}
                            else if mode == Mode::Info && display_mode == DisplayMode::Info{true}
                            else{false}
                        }
                        if same_mode(self.mode(), SELECTION_ACTION_DISPLAY_MODE) && self.mode_stack.top().text == Some(SELECTION_ACTION_OUT_OF_VIEW.to_string()){/* retain mode as SELECTION_ACTION_DISPLAY_MODE */}
                        else{
                            pop_to_insert(self);
                        }
                        //
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
                            //pop_to_insert(self);  //testing to see if this increments EDIT_ACTION_DISPLAY_MODE if selection_out_of_view
                            fn same_mode(mode: Mode, display_mode: DisplayMode) -> bool{
                                if mode == Mode::Error && display_mode == DisplayMode::Error{true}
                                else if mode == Mode::Warning && display_mode == DisplayMode::Warning{true}
                                else if mode == Mode::Notify && display_mode == DisplayMode::Notify{true}
                                else if mode == Mode::Info && display_mode == DisplayMode::Info{true}
                                else{false}
                            }
                            if same_mode(self.mode(), EDIT_ACTION_DISPLAY_MODE) && self.mode_stack.top().text == Some(EDIT_ACTION_OUT_OF_VIEW.to_string()){/* retain mode as EDIT_ACTION_DISPLAY_MODE */}
                            else{
                                pop_to_insert(self);
                            }
                            //
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
                                let parse_result = parse_command(self.ui.util_bar.utility_widget.text_box.buffer.to_string());
                                if Result::is_ok(&parse_result){
                                    let commands = Result::unwrap(parse_result);
                                    let execute_result = execute_commands(self, commands);
                                    if Result::is_ok(&execute_result){
                                        //only checking command mode because parsed resultant fn may need to enter error/warning/notify/info mode, and user should see that
                                        if self.mode() == Mode::Command{pop_to_insert(self);}
                                    }else{
                                        let error = Result::unwrap_err(execute_result);
                                        handle_message(self, DisplayMode::Error, &error);
                                    }
                                }else{
                                    //handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, COMMAND_PARSE_FAILED)
                                    let error = Result::unwrap_err(parse_result);
                                    handle_message(self, COMMAND_PARSE_FAILED_DISPLAY_MODE, &error);
                                }
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

#[derive(PartialEq, Debug, Clone)] enum ExpansionType{Option, Value, Register, Shell}
#[derive(PartialEq, Debug, Clone)] enum WordType{
    Unquoted,                   //word
    Quoted,                     //'a word', "a word", %{a word}
    Expansion(ExpansionType)    //%value{value_name}   //valid types are "shell", "register", "option", "value"
}
#[derive(PartialEq, Debug, Clone)] pub struct Word{
    word_type: WordType,
    content: String
}
//at the extreme, i think every action could end up being a command
//in that sense, the editor is just a command parser, with command specific response behavior
//NOTE: expansions should be performed at the time of execution. fn execute_command()
pub fn parse_command(command_string: String) -> Result<Vec<Vec<Word>>, String>{
    if command_string.is_empty(){return Err(String::from("cannot parse empty string"));}
    let mut commands = Vec::new();
    let mut command = Vec::new();
    let mut word = String::new();
    let mut expansion_type_string = String::new();

    let mut inside_of_quotations = false;
    let mut quote_char: Vec<char> = Vec::new();   //may need to become a Vec<char>, so that we can have nestable brace quotes no_op %sh{{ sleep 10 } > /dev/null 2>&1 < /dev/null &}     //push to vec on '{', pop from vec on '}'. push word to command if vec empty
    let mut inside_of_comment = false;
    //let mut escape_next = false;
    let mut follows_percent = false;
    #[cfg(test)] println!("command string:\n{}\n", command_string);
    #[cfg(test)] println!("command string length: {}\n", command_string.len());
    //TODO: for grapheme in command_string.graphemes(true){
    for (_i, char) in command_string.chars().enumerate(){
        //TODO: maybe we should push '\' to word, and pop from word if the following char is something we should escape
        //that way we don't have to double escape unquoted strings containg '\'
        #[cfg(test)] println!("char: {char}, index: {_i}");
        match char{
            ' ' | '\t' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{   //this may become inside_of_single_quote || inside_of_double_quote || inside_of_percent_quote
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    if !word.is_empty(){
                        #[cfg(test)] println!("word pushed to command: {:?}", word);
                        command.push(Word{word_type: WordType::Unquoted, content: word});
                        //reset
                        word = String::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                }
            }
            '\n' => {
                if inside_of_comment{
                    #[cfg(test)] println!("end of comment");
                    inside_of_comment = false;
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    if !word.is_empty(){
                        #[cfg(test)] println!("word pushed to command: {:?}", word);
                        command.push(Word{word_type: WordType::Unquoted, content: word});
                        //reset
                        word = String::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                    if !command.is_empty(){
                        #[cfg(test)] println!("command pushed to commands: {:?}", command);
                        commands.push(command);
                        //reset
                        command = Vec::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                }
            }
            ';' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    if !word.is_empty(){
                        #[cfg(test)] println!("word pushed to command: {:?}", word);
                        command.push(Word{word_type: WordType::Unquoted, content: word});
                        //reset
                        word = String::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                    if !command.is_empty(){
                        #[cfg(test)] println!("command pushed to commands: {:?}", command);
                        commands.push(command);
                        //reset
                        command = Vec::new();
                        inside_of_quotations = false;
                        quote_char = Vec::new();
                        follows_percent = false;
                    }
                }
            }
            '#' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    #[cfg(test)] println!("comment started");
                    inside_of_comment = true;
                }
            }
//TODO: reimpl how escapes work... 
//            '\\' => {
//                if inside_of_comment{
//                    #[cfg(test)] println!("char ignored as comment");
//                }
//                else if inside_of_quotations{
//                    #[cfg(test)] println!("char pushed to word: {char}");
//                    word.push(char);
//                }
//                else if escape_next{
//                    #[cfg(test)] println!("char pushed to word: {char}");
//                    word.push(char);
//                    escape_next = false;
//                }
//                else{
//                    #[cfg(test)] println!("escaping next char");
//                    escape_next = true;
//                }
//            }

//TODO: support expansion inside double quotes: echo "the date is %sh{date}"
            '\'' | '"' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    if quote_char.last() == Some(&char){    //if same as opening quote char
                        let _ = quote_char.pop();   //remove opening quote char from stack
                        if Option::is_none(&quote_char.last()){ //if quote char stack is empty  //should always be the case for '\'' and '"'
                            let _removed_char = word.remove(0); //remove leading '\'' or '"' from word
                            #[cfg(test)] println!("leading {} removed", _removed_char);

                            #[cfg(test)] println!("word pushed to command: {:?}", word);
                            command.push(Word{word_type: WordType::Quoted, content: word});
                            //reset necessary variables
                            word = String::new();
                            inside_of_quotations = false;
                            assert!(quote_char.is_empty()); //could prob remove if Option::is_none, and assert after quote_char.pop() above...
                        }
                    }else{  //for all other quote chars than same as opening, push to word
                        #[cfg(test)] println!("char pushed to word: {char}");
                        word.push(char);
                    }
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                    inside_of_quotations = true;
                    quote_char.push(char);
                }
            }
            '%' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    if word.is_empty(){follows_percent = true;}
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
            }
            '{' | '[' | '(' => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                    if quote_char.last() == Some(&char){
                        quote_char.push(char);
                    }
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else if follows_percent{
                    expansion_type_string = word.clone();   //copy preceding chars in word as expansion_type
                    expansion_type_string.remove(0);    //remove leading '%'
                    #[cfg(test)] println!("expansion_type_string: {}", expansion_type_string);

                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                    inside_of_quotations = true;
                    quote_char.push(char);
                    follows_percent = false;
                }
                else{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
            }
            '}' | ']' | ')' => {    //TODO: add '<' and '>' to supported percent quote characters
                fn inverse_brace(char: char) -> Option<char>{
                    if char == '}'{Some('{')}
                    else if char == ']'{Some('[')}
                    else if char == ')'{Some('(')}
                    else{None}  //or maybe unreachable!
                }
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    if quote_char.last() == Some(&inverse_brace(char).unwrap()){    //if char matches opening quote char    //ok to unwrap here because inputs are verified by parent match expression
                        let _ = quote_char.pop();   //remove latest opening quote char from stack
                        if Option::is_none(&quote_char.last()){
                            if expansion_type_string.is_empty(){
                                let _removed_char = word.remove(0); //remove leading '%' from word
                                #[cfg(test)] println!("leading {} removed", _removed_char);
                                let _removed_char = word.remove(0); //remove trailing '{', '[', '(', or '<' from word
                                #[cfg(test)] println!("trailing {} removed", _removed_char);

                                #[cfg(test)] println!("word pushed to command: {:?}", word);
                                command.push(Word{word_type: WordType::Quoted, content: word});
                            }else{
                                let _removed_char = word.remove(0); //remove leading '%' from word
                                #[cfg(test)] println!("leading {} removed", _removed_char);
                                for _ in 0..expansion_type_string.len(){
                                    let _removed_char = word.remove(0); //remove expansion type chars from word
                                    #[cfg(test)] println!("expansion string {} removed", _removed_char);
                                }
                                let _removed_char = word.remove(0); //remove trailing '{', '[', '(', or '<' from word
                                #[cfg(test)] println!("trailing {} removed", _removed_char);

                                let expansion_type = match expansion_type_string.as_str(){
                                    "opt" => ExpansionType::Option,
                                    "reg" => ExpansionType::Register,
                                    "sh" => ExpansionType::Shell,
                                    "val" => ExpansionType::Value,
                                    _ => return Err(String::from("unsupported expansion type"))
                                };
                                #[cfg(test)] println!("word pushed to command: {:?}", word);
                                command.push(Word{word_type: WordType::Expansion(expansion_type), content: word});
                            }
                            word = String::new();
                            inside_of_quotations = false;
                            assert!(quote_char.is_empty());
                        }else{
                            #[cfg(test)] println!("char pushed to word: {char}");
                            word.push(char);
                        }
                    }
                    else{
                        #[cfg(test)] println!("char pushed to word: {char}");
                        word.push(char);
                    }
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
            }
            //| => {}
            _ => {
                if inside_of_comment{
                    #[cfg(test)] println!("char ignored as comment");
                }
                else if inside_of_quotations{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
                //else if escape_next{
                //    #[cfg(test)] println!("char pushed to word: {char}");
                //    word.push(char);
                //    escape_next = false;
                //}
                else{
                    #[cfg(test)] println!("char pushed to word: {char}");
                    word.push(char);
                }
            }
        }
    }
    if !word.is_empty(){
        #[cfg(test)] println!("word pushed to command: {:?}", word);
        command.push(Word{word_type: WordType::Unquoted, content: word});
    }
    if !command.is_empty(){
        #[cfg(test)] println!("command pushed to commands: {:?}", command);
        commands.push(command);
    }
    if commands.is_empty(){return Err(String::from("failed to parse string as commands"));}
    #[cfg(test)] println!("commands: {:?}", commands);
    #[cfg(test)] println!("");
    Ok(commands)    
}
#[test]fn empty_command_string_should_error(){
    assert_eq!(Err(String::from("cannot parse empty string")), parse_command(String::from("")));
}
#[test] fn single_unquoted_word(){
    //idk
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("idk") //word
                    Word{
                        word_type: WordType::Unquoted,
                        content: String::from("idk")
                    }
                ]
            ]
        ), 
        parse_command(String::from("idk"))
    );
}
#[test] fn multiple_unquoted_words_separated_by_spaces(){
    //command --flag flag_item positional
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("command"),    //word
                    Word{word_type: WordType::Unquoted, content: String::from("command")},
                    //String::from("--flag"),     //word
                    Word{word_type: WordType::Unquoted, content: String::from("--flag")},
                    //String::from("flag_item"),  //word
                    Word{word_type: WordType::Unquoted, content: String::from("flag_item")},
                    //String::from("positional")  //word
                    Word{word_type: WordType::Unquoted, content: String::from("positional")},
                ]
            ]
        ), 
        parse_command(String::from("command --flag flag_item positional"))
    );
}
#[test] fn multiple_unquoted_words_separated_by_tabs(){
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("this"),       //word
                    Word{word_type: WordType::Unquoted, content: String::from("this")},
                    //String::from("command"),    //word
                    Word{word_type: WordType::Unquoted, content: String::from("command")},
                    //String::from("contains"),   //word
                    Word{word_type: WordType::Unquoted, content: String::from("contains")},
                    //String::from("tabs"),       //word
                    Word{word_type: WordType::Unquoted, content: String::from("tabs")}
                ]
            ]
        ),
        parse_command(String::from("this\tcommand\tcontains\ttabs"))
    )
}
#[test] fn newline_splits_multiple_commands(){
    //ln
    //sb
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("ln")//word
                    Word{word_type: WordType::Unquoted, content: String::from("ln")}
                ], 
                vec![//command
                    //String::from("sb")//word
                    Word{word_type: WordType::Unquoted, content: String::from("sb")}
                ]
            ]
        ), 
        parse_command(String::from("ln\nsb"))
    );
}
#[test] fn semicolon_splits_multiple_commands(){
    //ln;sb
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("ln")//word
                    Word{word_type: WordType::Unquoted, content: String::from("ln")}
                ], 
                vec![//command
                    //String::from("sb")//word
                    Word{word_type: WordType::Unquoted, content: String::from("sb")}
                ]
            ]
        ), 
        parse_command(String::from("ln;sb"))
    );
}
#[test] fn with_comment(){
    //# this is a comment
    //and this is a command
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("and"),    //word
                    Word{word_type: WordType::Unquoted, content: String::from("and")},
                    //String::from("this"),   //word
                    Word{word_type: WordType::Unquoted, content: String::from("this")},
                    //String::from("is"),     //word
                    Word{word_type: WordType::Unquoted, content: String::from("is")},
                    //String::from("a"),      //word
                    Word{word_type: WordType::Unquoted, content: String::from("a")},
                    //String::from("command") //word
                    Word{word_type: WordType::Unquoted, content: String::from("command")}
                ]
            ]
        ), 
        parse_command(String::from("# this is a comment\nand this is a command"))
    );
}
//#[test] fn with_escaped_characters(){
//    //idk \"some shit\"
//    assert_eq!(
//        Ok(
//            vec![//commands
//                vec![//command
//                    String::from("idk"),    //word
//                    String::from("\"some"), //word
//                    String::from("shit\"")  //word
//                ]
//            ]
//        ),
//        parse_command(String::from("idk \\\"some shit\\\""))
//    );
//}
#[test] fn single_quoted_string(){
    //'this is a quoted string'
    assert_eq!(
        Ok(
            vec![
                vec![
                    //String::from("'this is a quoted string'")
                    Word{word_type: WordType::Quoted, content: String::from("this is a quoted string")}
                ]
            ]
        ),
        parse_command(String::from("'this is a quoted string'"))
    );
}
#[test] fn unbalanced_single_quoted_returns_unquoted(){
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Unquoted, content: String::from("'idk")}
                ]
            ]
        ),
        parse_command(String::from("echo 'idk"))
    )
}
#[test] fn double_quoted_string(){
    //"this is a quoted string"
    assert_eq!(
        Ok(
            vec![
                vec![
                    //String::from("\"this is a quoted string\"")
                    Word{word_type: WordType::Quoted, content: String::from("this is a quoted string")}
                ]
            ]
        ),
        parse_command(String::from("\"this is a quoted string\""))
    );
}
#[test] fn unbalanced_double_quoted_returns_unquoted(){
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Unquoted, content: String::from("\"idk")}
                ]
            ]
        ),
        parse_command(String::from("echo \"idk"))
    )
}
#[test] fn with_space_inside_quotation(){
    //split ' '
    assert_eq!(
        Ok(
            vec![//commands
                vec![//command
                    //String::from("split"),  //word
                    Word{word_type: WordType::Unquoted, content: String::from("split")},
                    //String::from("' '")     //word
                    Word{word_type: WordType::Quoted, content: String::from(" ")}
                ]
            ]
        ), 
        parse_command(String::from("split ' '"))
    );
}
#[test] fn with_percent_string_no_type(){
    //echo %{this is quoted}
    assert_eq!(
        Ok(
            vec![
                vec![
                    //String::from("echo"),
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    //String::from("%{this is quoted}")
                    Word{word_type: WordType::Quoted, content: String::from("this is quoted")},
                ]
            ]
        ),
        parse_command(String::from("echo %{this is quoted}"))
    );
}
#[test] fn unbalanced_percent_string_returns_unquoted(){
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Unquoted, content: String::from("%{idk")}
                ]
            ]
        ),
        parse_command(String::from("echo %{idk"))
    )
}
#[test] fn with_percent_string_typed(){
    //echo %sh{this is quoted}
    assert_eq!(
        Ok(
            vec![
                vec![
                    //String::from("echo"),
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    //String::from("%sh{this is quoted}")
                    Word{word_type: WordType::Expansion(ExpansionType::Shell), content: String::from("this is quoted")}
                ]
            ]
        ),
        parse_command(String::from("echo %sh{this is quoted}"))
    );
}
#[test] fn percent_string_opt_typed(){
    //echo %opt{cursor_semantics}
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Expansion(ExpansionType::Option), content: String::from("cursor_semantics")}
                ]
            ]
        ),
        parse_command(String::from("echo %opt{cursor_semantics}"))
    );
}
#[test] fn unbalanced_expansion_returns_unquoted(){
    assert_eq!(
        Ok(
            vec![
                vec![
                    Word{word_type: WordType::Unquoted, content: String::from("echo")},
                    Word{word_type: WordType::Unquoted, content: String::from("%val{idk")}
                ]
            ]
        ),
        parse_command(String::from("echo %val{idk"))
    )
}


//TODO: consider how to handle a failed command in a list of commands. should we just error on first failed command?...
//TODO: execute_command should return a new instance of Application instead of modifying existing
//that way, we could apply changes to the new instance, and if an error occurs, default back to the old instance with no changes
//also, create a successful_commands counter. on each successful command, increment by 1;
//if unsuccessful command, return "Error: {error} on command {counter + 1}"
fn execute_commands(app: &mut Application, commands: Vec<Vec<Word>>) -> Result<(), String>{//Result<Application, ApplicationError>{
    fn expand(app: &Application, word_content: &str, expansion_type: &ExpansionType) -> Result<String, String>{
        fn expand_option(app: &Application, option: String) -> Result<String, String>{
            match option.as_ref(){
                "cursor_semantics" => Ok(format!("{:?}", app.config.semantics)),
                "use_full_file_path" => Ok(app.config.use_full_file_path.to_string()),
                "use_hard_tab" => Ok(app.config.use_hard_tab.to_string()),
                "tab_width" => Ok(app.config.tab_width.to_string()),
                "view_scroll_amount" => Ok(app.config.view_scroll_amount.to_string()),
                "show_cursor_column" => Ok(app.config.show_cursor_column.to_string()),
                "show_cursor_line" => Ok(app.config.show_cursor_line.to_string()),
                _ => {
                    match app.config.user_options.get(&option){
                        Some(option_type) => {
                            Ok(
                                match option_type{
                                    OptionType::Bool(bool) => bool.to_string(),
                                    OptionType::U8(u8) => u8.to_string(),
                                    OptionType::String(string) => string.clone()
                                }
                            )
                        }
                        None => Err(format!("{} option does not exist", option))
                    }
                }
            }
        }
        //fn expand_register() -> Result<String, ()>{Err(())}
        fn expand_shell(command_string: String) -> Result<String, String>{    //check content for $values, and set as environment variables
            let mut environment_variables = std::collections::HashMap::new();
            environment_variables.insert("MY_VAR", "environment variable content");
            let output = std::process::Command::new("sh"/*"bash"*/) //TODO: should this be calling the first arg in command string instead?...
                .arg("-c")
                .arg(command_string)
                //.env("MY_VAR", "environment variable content")
                .envs(&environment_variables)
                //.stdout(std::process::Stdio::piped()) //i think this is the default with .output()
                //.stderr(std::process::Stdio::piped()) //i think this is the default with .output()
                .output()
                .expect("failed to execute process");

            if output.status.success(){
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if stdout.is_empty(){
                    Ok(String::from("shell command succeeded with empty output string"))
                }else{
                    Ok(stdout)
                }
            }else{
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if stderr.is_empty(){
                    Err(String::from("shell command failed with empty error string"))
                }else{
                    Err(stderr)
                }
            }
        }
        //fn expand_value() -> Result<String, ()>{Err(())}

        match expansion_type{
            ExpansionType::Option => {
                expand_option(app, word_content.to_string())
            }
            ExpansionType::Register => {
                Err("register expansion unimplemented".to_string())
            }
            ExpansionType::Shell => {
                expand_shell(word_content.to_string())
            }
            ExpansionType::Value => {
                Err("value expansion unimplemented".to_string())
            }
        }
    }

    //thinking this should be useful to help with handling possible expansion on each command_words.next()
    fn resolve_content(app: &Application, word: &Word) -> Result<String, String>{   //maybe move Word instead of using &Word...
        match &word.word_type{
            WordType::Expansion(expansion_type) => {
                match expand(app, &word.content, expansion_type){
                    Ok(output) => Ok(output),
                    Err(error) => Err(error)
                }
            }
            _ => Ok(word.content.clone())
        }
    }

    for command in commands{
        let mut command_words = command.iter();
        let maybe_first = command_words.next();
        if Option::is_some(&maybe_first){
            let first_word = maybe_first.unwrap();
            let first_word_content = match resolve_content(app, first_word){
                Ok(content) => content,
                Err(error) => return Err(error)
            };
            match first_word_content.as_str(){
                "evaluate_commands" => {
                    //evaluate_commands <commands>
                    if command.len() > 2{
                        return Err(String::from("too many arguments supplied to evaluate_commands")); //too many arguments
                    }
                    let maybe_word = command_words.next();
                    if Option::is_some(&maybe_word){
                        let word = maybe_word.unwrap();
                        match resolve_content(app, word){
                            Ok(commands) => {
                                let parse_result = parse_command(commands);
                                if Result::is_ok(&parse_result){
                                    let commands = Result::unwrap(parse_result);
                                    //execute
                                    let execute_result = execute_commands(app, commands);
                                    if Result::is_err(&execute_result){
                                        let error = Result::unwrap_err(execute_result);
                                        return Err(error);
                                    }
                                }else{
                                    let error = Result::unwrap_err(parse_result);
                                    return Err(error);
                                }
                            }
                            Err(error) => return Err(error)
                        };
                    }else{
                        return Err(String::from("evaluate_commands requires more arguments"));
                    }
                }
                "echo" => { //TODO: bug if "echo --error --warning". should output "--warning" in error mode
                    //echo [diagnostic_mode] <message>
                    let mut display_mode = DisplayMode::Info;
                    let mut process_next_word = true;
                    while process_next_word{
                        let word = command_words.next();
                        if Option::is_some(&word){
                            let word = word.unwrap();
                            let word_content = match resolve_content(app, word){
                                Ok(content) => content,
                                Err(error) => return Err(error)
                            };
                            match word_content.as_str(){
                                "--error" => display_mode = DisplayMode::Error,
                                "--warning" => display_mode = DisplayMode::Warning,
                                "--notify" => display_mode = DisplayMode::Notify,
                                "--info" => {}  //already in DisplayMode::Info
                                _ => {
                                    let next_word = command_words.next();
                                    if Option::is_some(&next_word){
                                        return Err(String::from("too many arguments: echo [diagnostic_mode] <message>"));
                                    }
                                    process_next_word = false;
                                    handle_message(app, display_mode.clone(), &word_content);
                                }
                            }
                        }else{
                            return Err(String::from("too few arguments: echo [diagnostic_mode] <message>"));
                        }
                    }
                }
                //TODO: this should be a user defined command instead of built in
                //add-command "open new alacritty window" --doc_string "opens a new alacritty window" %sh{alacritty msg create-window}
                "term" | "t" => app.action(Action::EditorAction(EditorAction::OpenNewTerminalWindow)),
                "toggle_line_numbers" | "ln" => app.action(Action::EditorAction(EditorAction::ToggleLineNumbers)),  //these will prob end up using set-option command...
                "toggle_status_bar" | "sb" => app.action(Action::EditorAction(EditorAction::ToggleStatusBar)),      //these will prob end up using set-option command...
                "quit" | "q" => app.action(Action::EditorAction(EditorAction::Quit)),
                "quit!" | "q!" => app.action(Action::EditorAction(EditorAction::QuitIgnoringChanges)),
                //write buffer contents to file //should this optionally take a filepath to save to? then we don't need to implement save as    //would have to split util bar text on ' ' into separate args
                "write" | "w" => app.action(Action::EditorAction(EditorAction::Save)),
                "search" => {
                    //search <regex>
                    let maybe_word = command_words.next();
                    if Option::is_none(&maybe_word){
                        return Err(String::from("too few args: search <regex>"))
                    }else{
                        let word = maybe_word.unwrap();
                        let regex = match resolve_content(app, word){
                            Ok(content) => content,
                            Err(error) => return Err(error)
                        };
                        match crate::utilities::incremental_search_in_selection::selections_impl(
                            &app.selections, 
                            &regex,
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
                                return Err(String::from("no matching regex"))
                            }
                        }
                    }
                }
                "split" => {    //we may need to take certain regexes in quotes. i would assume the same applies to search
                    //split <regex>
                    let maybe_word = command_words.next();
                    if Option::is_none(&maybe_word){
                        return Err(String::from("too few args: search <regex>"))
                    }else{
                        let word = maybe_word.unwrap();
                        let regex = match resolve_content(app, word){
                            Ok(content) => content,
                            Err(error) => return Err(error)
                        };
                        match crate::utilities::incremental_split_in_selection::selections_impl(
                            &app.selections, 
                            &regex,
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
                                return Err(String::from("no matching regex"));
                            }
                        }   
                    }
                }
                
                //user defined commands may need to be quoted "if spaces are used"...
                //"\"idk some shit\"" => handle_message(app, DisplayMode::Error, "idk some shit"),  //commands with whitespace can be handled this way
                
                "add_command" => {
                    //add_command <command_name> <command> [optional_doc_string]
                    match command_words.next(){
                        Some(word) => {
                            let name = match resolve_content(app, word){
                                Ok(content) => content,
                                Err(error) => return Err(error)
                            };
                            match command_words.next(){
                                Some(word) => {
                                    let command = match resolve_content(app, word){
                                        Ok(content) => content,
                                        Err(error) => error
                                    };
                                    if app.config.user_commands.contains_key(&name){
                                        return Err(format!("commands already contains {} command", &name));
                                    }
                                    app.config.user_commands.insert(
                                        name.clone(), 
                                        Command{
                                            aliases: Vec::new(), 
                                            documentation: match command_words.next(){
                                                Some(word) => {
                                                    match resolve_content(app, word){
                                                        Ok(content) => Some(content),
                                                        Err(error) => return Err(error)
                                                    }
                                                }
                                                None => None
                                            }, 
                                            command_body: match parse_command(command){
                                                Ok(commands) => commands,
                                                Err(error) => return Err(error)
                                            }
                                        }
                                    );
                                    handle_message(app, DisplayMode::Notify, &format!("{} added to commands", name));
                                }
                                None => return Err(String::from("too few arguments: add_command <command_name> <command> [optional_documentation]"))
                            }
                        }
                        None => return Err(String::from("too few arguments: add_command <command_name> <command> [optional_documentation]"))
                    }
                }
                "remove_command" => {
                    //remove_command <command_name>
                    match command_words.next(){
                        None => return Err(String::from("too few arguments: remove_command <command_name>")),
                        Some(word) => {
                            let command_name = match resolve_content(app, word){
                                Ok(content) => content,
                                Err(error) => return Err(error)
                            };
                            match app.config.user_commands.remove(&command_name){
                                None => return Err(format!("{} does not exist in user commands", &command_name)),
                                Some(_) => handle_message(app, DisplayMode::Notify, &format!("{} removed from user commands", &command_name)),
                            }
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
                
                "add_option" => {   //TODO: handle excess args
                    //add_option <name> <option_type> [initial_value]
                    match command_words.next(){
                        Some(word) => {
                            //let name = word.content.clone();
                            let name = match resolve_content(app, word){
                                Ok(content) => content,
                                Err(error) => return Err(error)
                            };
                            let option_type = match command_words.next(){
                                Some(word) => {
                                    let option_type = match resolve_content(app, word){
                                        Ok(content) => content,
                                        Err(error) => return Err(error)
                                    };
                                    match option_type.as_str(){
                                        "bool" => OptionType::Bool(
                                            match command_words.next(){
                                                Some(word) => {
                                                    let value = match resolve_content(app, word){
                                                        Ok(content) => content,
                                                        Err(error) => return Err(error)
                                                    };
                                                    match value.as_str(){
                                                        "true" => true,
                                                        "false" => false,
                                                        _ => return Err(format!("{} is not a valid boolean value", value))
                                                    }
                                                }
                                                None => false
                                            }
                                        ),
                                        "u8" => OptionType::U8(
                                            match command_words.next(){
                                                Some(word) => {
                                                    let value = match resolve_content(app, word){
                                                        Ok(content) => content,
                                                        Err(error) => return Err(error)
                                                    };
                                                    let parsed_value: Result<u8, std::num::ParseIntError> = value.parse();
                                                    match parsed_value{
                                                        Ok(val) => val,
                                                        Err(error) => return Err(format!("{}", error))
                                                    }
                                                }
                                                None => 0
                                            }
                                        ),
                                        "string" => OptionType::String(
                                            match command_words.next(){
                                                Some(word) => {
                                                    match resolve_content(app, word){
                                                        Ok(content) => content,
                                                        Err(error) => return Err(error)
                                                    }
                                                }
                                                None => String::new()
                                            }
                                        ),
                                        _ => return Err(String::from("invalid option type"))
                                    }
                                }
                                None => return Err(String::from("too few arguments: add_option <name> <option_type> [initial_value]"))
                            };
                            //if command_words.next().is_some(){
                            //    return Err(String::from("too many arguments: add_option <name> <option_type> [initial_value]"))
                            //}
                            if app.config.user_options.contains_key(&name){
                                return Err(format!("user_options already contains {}", name));
                            }else{
                                app.config.user_options.insert(name.clone(), option_type);
                                handle_message(app, DisplayMode::Notify, &format!("{:?} added to user_options", name));
                            }
                        }
                        None => return Err(String::from("too few arguments: add_option <name> <option_type> [initial_value]"))
                    }
                }
                "remove_option" => {    //TODO: handle excess args
                    //remove_option <name>
                    match command_words.next(){
                        Some(word) => {
                            let name = match resolve_content(app, word){
                                Ok(content) => content,
                                Err(error) => return Err(error)
                            };
                            if app.config.user_options.contains_key(&name){
                                app.config.user_options.remove(&name);
                                handle_message(app, DisplayMode::Notify, &format!("{} removed from user_options", &name));
                            }else{
                                return Err(format!("{} is not a valid user_option", &name))
                            }
                        }
                        None => return Err(String::from("too few arguments: remove_option <name>"))
                    }
                }
                "set_option" => {   //TODO: handle excess args
                    //set_option <name> <value>
                    match command_words.next(){
                        Some(word) => {
                            let name = match resolve_content(app, word){
                                Ok(content) => content,
                                Err(error) => return Err(error)
                            };
                            match command_words.next(){
                                Some(word) => {
                                    let value = match resolve_content(app, word){
                                        Ok(content) => content,
                                        Err(error) => return Err(error)
                                    };
                                    match name.as_ref(){
                                        "cursor_semantics" => {
                                            match value.as_str(){
                                                "Bar" => {
                                                    app.config.semantics = CursorSemantics::Bar;
                                                    handle_message(app, DisplayMode::Notify, &format!("cursor_semantics set to {}", value));
                                                }
                                                "Block" => {
                                                    app.config.semantics = CursorSemantics::Block;
                                                    handle_message(app, DisplayMode::Notify, &format!("cursor_semantics set to {}", value));
                                                }
                                                _ => return Err(format!("{} is not a valid value for cursor_semantics", value))
                                            }
                                        }
                                        "use_full_file_path" => {
                                            let maybe_parsed_value: Result<bool, std::str::ParseBoolError> = value.parse();
                                            match maybe_parsed_value{
                                                Ok(parsed_value) => {
                                                    app.config.use_full_file_path = parsed_value;
                                                    handle_message(app, DisplayMode::Notify, &format!("use_full_file_path set to {}", parsed_value));
                                                }
                                                Err(error) => return Err(format!("{}", error))
                                            }
                                        }
                                        "use_hard_tab" => {
                                            let maybe_parsed_value: Result<bool, std::str::ParseBoolError> = value.parse();
                                            match maybe_parsed_value{
                                                Ok(parsed_value) => {
                                                    app.config.use_hard_tab = parsed_value;
                                                    handle_message(app, DisplayMode::Notify, &format!("use_hard_tab set to {}", parsed_value));
                                                }
                                                Err(error) => return Err(format!("{}", error))
                                            }
                                        }
                                        "tab_width" => {
                                            let maybe_parsed_value: Result<usize, std::num::ParseIntError> = value.parse();
                                            match maybe_parsed_value{
                                                Ok(parsed_value) => {
                                                    app.config.tab_width = parsed_value;
                                                    handle_message(app, DisplayMode::Notify, &format!("tab_width set to {}", parsed_value));
                                                }
                                                Err(error) => return Err(format!("{}", error))
                                            }
                                        }
                                        "view_scroll_amount" => {
                                            let maybe_parsed_value: Result<usize, std::num::ParseIntError> = value.parse();
                                            match maybe_parsed_value{
                                                Ok(parsed_value) => {
                                                    app.config.view_scroll_amount = parsed_value;
                                                    handle_message(app, DisplayMode::Notify, &format!("view_scroll_amount set to {}", parsed_value));
                                                }
                                                Err(error) => return Err(format!("{}", error))
                                            }
                                        }
                                        "show_cursor_column" => {
                                            let maybe_parsed_value: Result<bool, std::str::ParseBoolError> = value.parse();
                                            match maybe_parsed_value{
                                                Ok(parsed_value) => {
                                                    app.config.show_cursor_column = parsed_value;
                                                    handle_message(app, DisplayMode::Notify, &format!("show_cursor_column set to {}", parsed_value));
                                                }
                                                Err(error) => return Err(format!("{}", error))
                                            }
                                        }
                                        "show_cursor_line" => {
                                            let maybe_parsed_value: Result<bool, std::str::ParseBoolError> = value.parse();
                                            match maybe_parsed_value{
                                                Ok(parsed_value) => {
                                                    app.config.show_cursor_line = parsed_value;
                                                    handle_message(app, DisplayMode::Notify, &format!("show_cursor_line set to {}", parsed_value));
                                                }
                                                Err(error) => return Err(format!("{}", error))
                                            }
                                        }
                                        _ => {
                                            let option_type = match app.config.user_options.get(&name){
                                                Some(option_type) => option_type,
                                                None => return Err(format!("user_options does not contain {}", &name))
                                            };
                                            match option_type{
                                                OptionType::Bool(_) => {
                                                    let maybe_parsed_value: Result<bool, std::str::ParseBoolError> = value.parse();
                                                    match maybe_parsed_value{
                                                        Ok(parsed_value) => {
                                                            app.config.user_options.insert(name.clone(), OptionType::Bool(parsed_value));
                                                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                                        }
                                                        Err(error) => return Err(format!("{}", error))
                                                    }
                                                }
                                                OptionType::U8(_) => {
                                                    let maybe_parsed_value: Result<u8, std::num::ParseIntError> = value.parse();//word.content.parse();
                                                    match maybe_parsed_value{
                                                        Ok(parsed_value) => {
                                                            app.config.user_options.insert(name.clone(), OptionType::U8(parsed_value));
                                                            handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, parsed_value));
                                                        }
                                                        Err(error) => return Err(format!("{}", error))
                                                    }
                                                }
                                                OptionType::String(_) => {
                                                    app.config.user_options.insert(name.clone(), OptionType::String(value.clone()));
                                                    handle_message(app, DisplayMode::Notify, &format!("{} set to {}", name, value));
                                                }
                                            }
                                        }
                                    }
                                }
                                None => return Err(String::from("too few arguments: set_option <name> <value>"))
                            }
                        }
                        None => return Err(String::from("too few arguments: set_option <name> <value>"))
                    }
                }
                
                "no_op" => {    //this would be used to start some external program or similar. no editor explicit behavior
                    //no_op <command>
                    match command_words.next(){
                        None => return Err(String::from("too few args: no_op <command>")),
                        Some(word) => {
                            match resolve_content(app, word){
                                Ok(_content) => {}
                                Err(error) => return Err(error)
                            }
                        }
                    }
                    //should we really be displaying anything here?...
                    handle_message(app, DisplayMode::Info, "no op");
                }
                //add_hook <group_name> <event> <filtering_regex> <response_command>    //maybe set a hook name instead of group?...    //if no group/name provided, only trigger once, then remove
                //remove hook <group_name>
                //TODO: add-selection
                //TODO: set-selection
                //add-highlighter <group_id> [buffer_offset|widget_coords|screen_coords] <value>
                    //value = buffer range | widget line/column/cell | screen line/column/cell
                    //buffer_offset highlighter could map directly to the buffer, which would convert to widget_coords for render...
                //remove-highlighter <group_id>
                _ => {
                    match app.config.user_commands.get(&first_word_content){
                        Some(command) => {
                            match execute_commands(app, command.command_body.clone()){
                                Ok(()) => {}
                                Err(error) => return Err(error)
                            }
                        }
                        None => return Err(String::from("no matching command registered"))
                    }
                }
            }
        }else{
            return Err(String::from("cannot execute empty command"));
        }
    }
    Ok(())
}

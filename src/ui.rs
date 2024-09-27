use crate::application::{Mode, WarningKind};
use edit::Position;
use std::error::Error;
use ratatui::Terminal;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color, Stylize};
use ratatui::layout::{Alignment, Direction, Layout, Constraint};
use unicode_segmentation::UnicodeSegmentation;



const GOTO_PROMPT: &str = " Go to: ";
const FIND_PROMPT: &str = " Find: ";
const REPLACE_PROMPT: &str = " Replace: ";
const MODIFIED_INDICATOR: &str = "[Modified]";
const COMMAND_PROMPT: &str = " Command: ";



// TODO: can util bar use Selection from selection.rs? and use all the predefined movement functionality?
#[derive(Default)]
struct Selection{
    anchor: u16,
    head: u16,
}

pub struct UtilBar{
    text: String,
    text_is_valid: bool,
    selection: Selection,
    offset: u16,
    widget_width: u16
}
impl UtilBar{
    pub fn default() -> Self{
        Self{
            text: String::new(),
            text_is_valid: false,
            selection: Selection::default(),
            offset: 0,
            widget_width: 0
        }
    }

    pub fn text(&self) -> &str{
        &self.text
    }

    pub fn set_text_is_valid(&mut self, text_is_valid: bool){
        self.text_is_valid = text_is_valid
    }

    pub fn cursor_position(&self) -> u16{
        self.selection.head
    }

    pub fn set_widget_width(&mut self, width: u16){
        self.widget_width = width;
    }

    pub fn clear(&mut self){
        self.text.clear();
        self.selection.head = 0;
        self.selection.anchor = 0;
    }

    pub fn insert_char(&mut self, char: char){
        if self.selection.head != self.selection.anchor{
            self.delete();
        }
        self.text.insert(self.selection.head.into(), char);
        self.move_cursor_right();
    }

    pub fn delete(&mut self){
        let mut result = String::new();
        if self.selection.head != self.selection.anchor{
            for (index, grapheme) in self.text.graphemes(true).enumerate(){
                let index = index as u16;
                if self.selection.head < self.selection.anchor{
                    if index < self.selection.head || index > self.selection.anchor{
                        result.push_str(grapheme);
                    }
                }else if self.selection.head > self.selection.anchor{
                    if index < self.selection.anchor || index > self.selection.head{
                        result.push_str(grapheme);
                    }
                }
            }
            self.collapse_selection();
        }else{
            for (index, grapheme) in self.text[..].graphemes(true).enumerate(){
                if index != self.selection.head as usize{
                    result.push_str(grapheme);
                }
            }
        }
        self.text = result;
    }

    pub fn backspace(&mut self){
        if self.selection.head != self.selection.anchor{
            self.delete();
        }
        else{
            self.move_cursor_left();
            self.delete();
        }
    }

    pub fn move_cursor_right(&mut self){
        if self.selection.head < self.text.graphemes(true).count() as u16{
            self.selection.head = self.selection.head.saturating_add(1);
            self.selection.anchor = self.selection.anchor.saturating_add(1);
        }
    }
    pub fn extend_selection_right(&mut self){
        if self.selection.head < self.text.graphemes(true).count() as u16{
            self.selection.head = self.selection.head.saturating_add(1);
        }
    }

    pub fn move_cursor_left(&mut self){
        if self.selection.head > 0{
            self.selection.head = self.selection.head.saturating_sub(1);
            self.selection.anchor = self.selection.anchor.saturating_sub(1);
        }
    }
    pub fn extend_selection_left(&mut self){
        if self.selection.head > 0{
            self.selection.head = self.selection.head.saturating_sub(1);
        }
    }

    pub fn move_cursor_home(&mut self){
        self.selection.head = 0;
        self.selection.anchor = 0;
        self.offset = 0;
    }
    pub fn extend_selection_home(&mut self){
        self.selection.head = 0;
        self.offset = 0;
    }

    pub fn move_cursor_end(&mut self){
        self.selection.head = self.text.graphemes(true).count() as u16;
        self.selection.anchor = self.text.graphemes(true).count() as u16;
    }
    pub fn extend_selection_end(&mut self){
        self.selection.head = self.text.graphemes(true).count() as u16;
    }

    pub fn offset(&self) -> u16{
        self.offset
    }
    pub fn set_offset(&mut self, offset: u16){
        self.offset = offset;
    }

    pub fn scroll(&mut self){
        if self.selection.head < self.offset{
            self.offset = self.selection.head;
        }
        else if self.selection.head >= self.offset.saturating_add(self.widget_width){
            self.offset = self.selection.head.saturating_sub(self.widget_width).saturating_add(1);
        }
    }

    pub fn collapse_selection(&mut self){
        if self.selection.head < self.selection.anchor{
            self.selection.anchor = self.selection.head;
        }
        else if self.selection.head > self.selection.anchor{
            self.selection.head = self.selection.anchor;
        }
    }
}

pub struct UserInterface{
    terminal_size: Rect,
    display_line_numbers: bool,
    display_status_bar: bool,
    /// the area of the terminal filled by an open document
    document_rect: Rect,
    /// the area of the terminal filled by line numbers
    line_number_rect: Rect,
    /// the area of the status bar for indicating file modification status
    status_bar_modified_indicator_rect: Rect,
    /// the area of the status bar for indicating file name
    status_bar_file_name_rect: Rect,
    /// the area of the status bar for indicating cursor position within document
    status_bar_cursor_position_rect: Rect,
    /// the area of the util bar for primary utility prompts
    util_bar_prompt_rect: Rect,
    /// the area of the util bar for primary user input
    util_bar_rect: Rect,
    /// the area of the util bar for alternate utility prompts
    util_bar_alternate_prompt_rect: Rect,
    /// the area of the util bar for alternate user input
    util_bar_alternate_rect: Rect,
    util_bar_alternate_focused: bool,
    /// holds util bar specific state
    util_bar: UtilBar,
    util_bar_alternate: UtilBar,
    text_in_view: String,
    line_numbers_in_view: String,
    client_cursor_position: Option<Position>,
    document_length: usize,
    document_modified_status: bool,
    document_file_name: Option<String>,
    document_cursor_position: Option<Position>,
}
impl UserInterface{
    pub fn new(terminal_size: Rect) -> Self{
        Self{
            terminal_size,
            display_line_numbers: true,
            display_status_bar: true,
            document_rect: Rect::default(),
            line_number_rect: Rect::default(),
            status_bar_modified_indicator_rect: Rect::default(),
            status_bar_file_name_rect: Rect::default(),
            status_bar_cursor_position_rect: Rect::default(),
            util_bar_prompt_rect: Rect::default(),
            util_bar_rect: Rect::default(),
            util_bar_alternate_prompt_rect: Rect::default(),
            util_bar_alternate_rect: Rect::default(),
            util_bar_alternate_focused: false,
            util_bar: UtilBar::default(),
            util_bar_alternate: UtilBar::default(),
            text_in_view: String::new(),
            line_numbers_in_view: String::new(),
            client_cursor_position: None,
            document_length: 0,
            document_modified_status: false,
            document_file_name: None,
            document_cursor_position: None,
        }
    }
    pub fn document_modified(&self) -> bool{
        self.document_modified_status
    }
    pub fn set_document_modified(&mut self, modified: bool){
        self.document_modified_status = modified;
    }
    pub fn set_terminal_size(&mut self, width: u16, height: u16){
        self.terminal_size.width = width;
        self.terminal_size.height = height;
    }
    pub fn set_file_name(&mut self, file_name: Option<String>){
        self.document_file_name = file_name;
    }
    pub fn document_length(&self) -> usize{
        self.document_length
    }
    pub fn set_document_length(&mut self, document_length: usize){
        self.document_length = document_length;
    }
    pub fn set_document_cursor_position(&mut self, cursor_position: Position){
        self.document_cursor_position = Some(cursor_position);
    }

    pub fn document_rect(&self) -> Rect{
        self.document_rect
    }

    pub fn display_line_numbers(&self) -> bool{
        self.display_line_numbers
    }
    pub fn set_display_line_numbers(&mut self, display_line_numbers: bool){
        self.display_line_numbers = display_line_numbers
    }

    pub fn display_status_bar(&self) -> bool{
        self.display_status_bar
    }
    pub fn set_display_status_bar(&mut self, display_status_bar: bool){
        self.display_status_bar = display_status_bar
    }

    pub fn util_bar(&self) -> &UtilBar{
        &self.util_bar
    }
    pub fn util_bar_mut(&mut self) -> &mut UtilBar{
        &mut self.util_bar
    }

    pub fn util_bar_alternate_mut(&mut self) -> &mut UtilBar{
        &mut self.util_bar_alternate
    }

    pub fn util_bar_alternate_focused(&self) -> bool{
        self.util_bar_alternate_focused
    }
    pub fn set_util_bar_alternate_focused(&mut self, util_bar_alternate_focused: bool){
        self.util_bar_alternate_focused = util_bar_alternate_focused
    }

    pub fn set_text_in_view(&mut self, text: String){
        self.text_in_view = text;
    }
    pub fn set_line_numbers_in_view(&mut self, line_numbers: String){
        self.line_numbers_in_view = line_numbers;
    }

    pub fn set_client_cursor_position(&mut self, positions: Vec<Position>){
        //self.client_cursor_position = position;
        if !positions.is_empty(){
            self.client_cursor_position = Some(*positions.last().unwrap());
        }else{
            self.client_cursor_position = None;
        }
    }


    pub fn update_layouts(&mut self, mode: Mode){
        // layout of viewport rect (the whole terminal screen)
        let viewport_rect = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                vec![
                    // document + line num rect height
                    Constraint::Min(0),
                    // status bar rect height
                    Constraint::Length(if self.display_status_bar{1}else{0}),
                    // util(goto/find/command) bar rect height
                    Constraint::Length(
                        match mode{
                            Mode::Warning(_) 
                            | Mode::Goto 
                            | Mode::FindReplace
                            | Mode::Command => 1,
                            
                            Mode::Insert => if self.display_status_bar{1}else{0}
                        }
                    )
                ]
            )
            .split(self.terminal_size);

        // layout of document + line num rect
        let document_and_line_num_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // line number left padding
                    //Constraint::Length(if self.display_line_numbers{1}else{0}),
                    // line number rect width
                    Constraint::Length(
                        if self.display_line_numbers{
                            count_digits(self.document_length)
                        }else{0}
                    ),
                    // line number right padding
                    Constraint::Length(if self.display_line_numbers{1}else{0}),
                    // document rect width
                    Constraint::Min(5)
                ]
            )
            .split(viewport_rect[0]);

        // layout of status bar rect (modified_indicator/file_name/cursor_position)
        let status_bar_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // modified indicator width
                    Constraint::Max(
                        if self.document_modified_status{
                            MODIFIED_INDICATOR.len() as u16
                        }else{0}
                    ),
                    // file_name width
                    Constraint::Max(
                        if let Some(file_name) = &self.document_file_name{
                            file_name.len() as u16
                        }else{0}
                    ),
                    // cursor position indicator width
                    Constraint::Min(0)
                ]
            )
            .split(viewport_rect[1]);

        // layout of util rect (goto/find/command/save as)
        let util_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // util bar prompt width
                    Constraint::Length(
                        match mode{
                            Mode::Goto => GOTO_PROMPT.len() as u16,
                            Mode::FindReplace => FIND_PROMPT.len() as u16,
                            Mode::Command => COMMAND_PROMPT.len() as u16,
                            _ => 0
                        }
                    ),
                    // util bar rect width
                    Constraint::Length(
                        match mode{
                            Mode::Warning(_) | Mode::Insert => viewport_rect[2].width,
                            Mode::Goto => viewport_rect[2].width - GOTO_PROMPT.len() as u16,
                            Mode::Command => viewport_rect[2].width - COMMAND_PROMPT.len() as u16,                            
                            Mode::FindReplace => (viewport_rect[2].width / 2) - FIND_PROMPT.len() as u16,
                        }
                    ),
                    // util bar alternate prompt width
                    Constraint::Length(
                        match mode{
                            Mode::FindReplace => REPLACE_PROMPT.len() as u16,
                            _ => 0
                        }
                    ),
                    // util bar alternate rect width
                    Constraint::Length(
                        match mode{
                            Mode::FindReplace => (viewport_rect[2].width / 2) - REPLACE_PROMPT.len() as u16,
                            _ => 0
                        }
                    ),
                    // used to fill in space when other two are 0 length
                    Constraint::Length(0)
                ]
            )
            .split(viewport_rect[2]);

        self.line_number_rect = document_and_line_num_rect[0];
        // dont have to set line num right padding(document_and_line_num_rect[1])
        self.document_rect = document_and_line_num_rect[2];
        self.status_bar_modified_indicator_rect = status_bar_rect[0];
        self.status_bar_file_name_rect = status_bar_rect[1];
        self.status_bar_cursor_position_rect = status_bar_rect[2];
        self.util_bar_prompt_rect = util_rect[0];
        self.util_bar_rect = util_rect[1];
        self.util_bar_alternate_prompt_rect = util_rect[2];
        self.util_bar_alternate_rect = util_rect[3];

        match mode{
            Mode::Command | Mode::Goto | Mode::FindReplace=> {
                self.util_bar.set_widget_width(self.util_bar_rect.width);
                self.util_bar_alternate.set_widget_width(self.util_bar_alternate_rect.width);
            }
            _ => {
                self.util_bar.set_widget_width(0);
                self.util_bar_alternate.set_widget_width(0);
            }
        }
    }

    //TODO: find out why we have double padding to left of line nums
    pub fn line_number_widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.line_numbers_in_view.clone())
            .style(Style::default().fg(Color::Rgb(100, 100, 100)))
            .alignment(Alignment::Right)
    }

    pub fn document_widget(&self) -> Paragraph<'static>{
            Paragraph::new(self.text_in_view.clone())
    }

    pub fn status_bar_modified_indicator_widget(&self) -> Paragraph<'static>{
        Paragraph::new(MODIFIED_INDICATOR)
            .alignment(Alignment::Left)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
                    .bold()
            )
    }

    pub fn status_bar_file_name_widget(&self) -> Paragraph<'static>{
        let file_name = match &self.document_file_name{
            Some(file_name) => file_name.to_string(),
            None => "None".to_string()
        };
        Paragraph::new(file_name)
            .alignment(Alignment::Left)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
                    .bold()
                )
    }

    pub fn status_bar_cursor_position_widget(&self) -> Paragraph<'static>{
        let position = match self.document_cursor_position{
            Some(cursor_position) => {
                format!(
                    "{}:{}",
                    cursor_position.y(),// + 1,
                    cursor_position.x()// + 1
                )
            }
            None => "None".to_string()
        };
        Paragraph::new(position)
            .alignment(Alignment::Right)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
                    .bold()
            )
    }

    pub fn util_bar_prompt_widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Goto => Paragraph::new(GOTO_PROMPT),
            Mode::FindReplace => Paragraph::new(FIND_PROMPT),
            Mode::Command => Paragraph::new(COMMAND_PROMPT),
            _ => Paragraph::new("")
        }
    }

    pub fn util_bar_widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Goto | Mode::FindReplace => {
                if self.util_bar.text_is_valid{
                    Paragraph::new(self.util_bar.text().to_string()).scroll((0, self.util_bar.offset()))
                }else{
                    Paragraph::new(self.util_bar.text().to_string())
                        .scroll((0, self.util_bar.offset()))
                        .style(Style::default().fg(Color::Red))
                }
            }
            Mode::Command => {
                Paragraph::new(self.util_bar.text().to_string()).scroll((0, self.util_bar.offset()))
            }
            Mode::Warning(kind) => Paragraph::new(
                match kind{
                    WarningKind::FileIsModified => {
                        "WARNING! File has unsaved changes. Press close again to ignore and close."
                    }
                    WarningKind::FileSaveFailed => {
                        "WARNING! File could not be saved."
                    }
                    //WarningKind::FileOpenFailed => {
                    //    "WARNING! File could not be opened."
                    //}
                }
            )
                .alignment(ratatui::prelude::Alignment::Center)
                .style(Style::default().bg(Color::Red).bold())
            ,
            _ => Paragraph::new("".to_string())
        }
    }

    pub fn util_bar_alternate_prompt_widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::FindReplace => {
                Paragraph::new(REPLACE_PROMPT)
            },
            _ => Paragraph::new("")
        }
    }

    pub fn util_bar_alternate_widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::FindReplace => {
                Paragraph::new(self.util_bar_alternate.text().to_string())
                    .scroll((0, self.util_bar_alternate.offset()))
            }
            _ => Paragraph::new(self.util_bar_alternate.text().to_string())
        }
    }

    // when in select mode, figure out how to change background color of text within cursor_head and cursor_anchor
    pub fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, mode: Mode) -> Result<(), Box<dyn Error>>{        
        terminal.draw(
            |frame| {

                // render widgets
                frame.render_widget(self.line_number_widget(), self.line_number_rect);
                frame.render_widget(self.document_widget(), self.document_rect);
                frame.render_widget(self.status_bar_modified_indicator_widget(), self.status_bar_modified_indicator_rect);
                frame.render_widget(self.status_bar_file_name_widget(), self.status_bar_file_name_rect);
                frame.render_widget(self.status_bar_cursor_position_widget(), self.status_bar_cursor_position_rect);
                frame.render_widget(self.util_bar_prompt_widget(mode), self.util_bar_prompt_rect);
                frame.render_widget(self.util_bar_widget(mode), self.util_bar_rect);
                frame.render_widget(self.util_bar_alternate_prompt_widget(mode), self.util_bar_alternate_prompt_rect);
                frame.render_widget(self.util_bar_alternate_widget(mode), self.util_bar_alternate_rect);

                // render cursor
                match mode{
                    Mode::Insert => {
                        if let Some(pos) = self.client_cursor_position{
                            frame.set_cursor(
                                self.document_rect.x + pos.x() as u16, 
                                self.document_rect.y + pos.y() as u16
                            )
                        }
                    }
                    Mode::Goto | Mode::Command => {
                        frame.set_cursor(
                            self.util_bar_rect.x + self.util_bar.cursor_position().saturating_sub(self.util_bar.offset()),
                            self.terminal_size.height
                        );
                    }
                    Mode::FindReplace => {
                        frame.set_cursor(
                            if self.util_bar_alternate_focused{
                                self.util_bar_alternate_rect.x + self.util_bar_alternate.cursor_position()
                                    .saturating_sub(self.util_bar_alternate.offset())
                            }else{
                                self.util_bar_rect.x + self.util_bar.cursor_position().saturating_sub(self.util_bar.offset())
                            },
                            self.terminal_size.height
                        );
                    }
                    Mode::Warning(_) => {}
                }
            }

        )?;

        Ok(())
    }
}

//fn _centered_rect(percent_x: u16, percent_y: u16, r: ratatui::prelude::Rect) -> ratatui::prelude::Rect{
//    let popup_layout = Layout::default()
//        .direction(Direction::Vertical)
//        .constraints(
//            [
//                Constraint::Percentage((100 - percent_y) / 2),
//                Constraint::Percentage(percent_y),
//                Constraint::Percentage((100 - percent_y) / 2),
//            ]
//            .as_ref(),
//        )
//        .split(r);
//
//    Layout::default()
//        .direction(Direction::Horizontal)
//        .constraints(
//            [
//                Constraint::Percentage((100 - percent_x) / 2),
//                Constraint::Percentage(percent_x),
//                Constraint::Percentage((100 - percent_x) / 2),
//            ]
//            .as_ref(),
//        )
//        .split(popup_layout[1])[1]
//}



fn count_digits(mut n: usize) -> u16{
    if n == 0{
        return 1;
    }

    let mut count = 0;
    while n > 0{
        count += 1;
        n /= 10;
    }

    count
}

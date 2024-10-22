use crate::application::{Mode, UtilityKind, WarningKind};
use edit_core::{
    selection::{CursorSemantics, Movement, Selection}, view::View, Position
};
use ropey::Rope;
use std::cmp::Ordering;
use std::error::Error;
use ratatui::Terminal;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color, Stylize};
use ratatui::layout::{Alignment, Direction, Layout, Constraint};



const GOTO_PROMPT: &str = " Go to: ";
const FIND_PROMPT: &str = " Find: ";
const REPLACE_PROMPT: &str = " Replace: ";
const MODIFIED_INDICATOR: &str = "[Modified]";
const COMMAND_PROMPT: &str = " Command: ";



pub struct Utility{
    text: Rope,
    text_is_valid: bool,
    selection: Selection,
    view: View
}
impl Default for Utility{
    fn default() -> Self{
        Self{
            text: Rope::from(""),
            text_is_valid: false,
            selection: Selection::new(0, 1),
            view: View::new(0, 0, 0, 1)
        }
    }
}
impl Utility{
    pub fn selection(&self) -> &Selection{
        &self.selection
    }
    pub fn selection_mut(&mut self) -> &mut Selection{
        &mut self.selection
    }
    pub fn view(&self) -> &View{
        &self.view
    }
    pub fn view_mut(&mut self) -> &mut View{
        &mut self.view
    }
    pub fn text(&self) -> &Rope{
        &self.text
    }
    pub fn set_text_is_valid(&mut self, text_is_valid: bool){
        self.text_is_valid = text_is_valid
    }
    pub fn cursor_position(&self) -> u16{
        self.selection.cursor(CursorSemantics::Block) as u16
    }
    pub fn set_widget_width(&mut self, width: u16){
        let height = self.view.height();
        self.view.set_size(width as usize, height);
    }
    pub fn clear(&mut self){
        *self = Self::default();
    }
    pub fn insert_char(&mut self, char: char){
        if self.selection.is_extended(CursorSemantics::Block){
            self.delete();
        }
        let text = self.text.clone();
        let mut new_text = text.clone();
        new_text.insert_char(self.selection.cursor(CursorSemantics::Block), char);
        self.text = new_text;
        self.selection = self.selection.move_right(&self.text.clone(), CursorSemantics::Block);
    }
    pub fn delete(&mut self){
        let text = self.text.clone();
        let mut new_text = self.text.clone();

        match self.selection.cursor(CursorSemantics::Block).cmp(&self.selection.anchor()){
            Ordering::Less => {
                new_text.remove(self.selection.head()..self.selection.anchor());
                self.selection = self.selection.put_cursor(self.selection.cursor(CursorSemantics::Block), &text, Movement::Move, CursorSemantics::Block, true);
            }
            Ordering::Greater => {
                if self.selection.cursor(CursorSemantics::Block) == text.len_chars(){
                    new_text.remove(self.selection.anchor()..self.selection.cursor(CursorSemantics::Block));
                }
                else{
                    new_text.remove(self.selection.anchor()..self.selection.head());
                }
                self.selection = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true);
            }
            Ordering::Equal => {
                if self.selection.cursor(CursorSemantics::Block) == text.len_chars(){}    //do nothing
                else{
                    new_text.remove(self.selection.anchor()..self.selection.head());
                    self.selection = self.selection.put_cursor(self.selection.anchor(), &text, Movement::Move, CursorSemantics::Block, true);
                }
            }
        }

        self.text = new_text;
    }
    #[allow(clippy::collapsible_else_if)]
    pub fn backspace(&mut self){
        let semantics = CursorSemantics::Block;
        if self.selection.is_extended(semantics){
            self.delete();
        }else{
            if self.selection.cursor(semantics) > 0{
                self.selection = self.selection.move_left(&self.text, semantics);
                self.delete();
            }
        }
    }
}

#[derive(Default)]
pub struct UtilityWidget{
    rect: Rect,
    util_bar: Utility,
}
impl UtilityWidget{
    pub fn util_bar(&self) -> &Utility{
        &self.util_bar
    }
    pub fn util_bar_mut(&mut self) -> &mut Utility{
        &mut self.util_bar
    }
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Utility(UtilityKind::Goto) | Mode::Utility(UtilityKind::FindReplace) => {
                let text = self.util_bar.text.clone();
                if self.util_bar.text_is_valid{
                    Paragraph::new(self.util_bar.view.text(&text))
                }else{
                    Paragraph::new(self.util_bar.view.text(&text))
                        .style(Style::default().fg(Color::Red))
                }
            }
            Mode::Utility(UtilityKind::Command) => {
                let text = self.util_bar.text.clone();
                Paragraph::new(self.util_bar.view.text(&text))
            }
            Mode::Utility(UtilityKind::Warning(kind)) => Paragraph::new(
                match kind{
                    WarningKind::FileIsModified => {
                        "WARNING! File has unsaved changes. Press close again to ignore and close."
                    }
                    WarningKind::FileSaveFailed => {
                        "WARNING! File could not be saved."
                    }
                    WarningKind::CommandParseFailed => {
                        "WARNING! Failed to parse command. Command may be undefined."
                    }
                    WarningKind::SingleSelection => {
                        "WARNING! Requested action cannot be performed on single selection."
                    }
                    WarningKind::MultipleSelections => {
                        "WARNING! Requested action cannot be performed on multiple selections."
                    }
                    WarningKind::InvalidInput => {
                        "WARNING! Invalid input."
                    }
                }
            )
                .alignment(ratatui::prelude::Alignment::Center)
                .style(Style::default().bg(Color::Red).bold())
            ,
            _ => Paragraph::new("".to_string())
        }
    }
}
#[derive(Default)]
pub struct UtilityAlternateWidget{
    rect: Rect,
    util_bar: Utility,
}
impl UtilityAlternateWidget{
    pub fn util_bar(&self) -> &Utility{
        &self.util_bar
    }
    pub fn util_bar_mut(&mut self) -> &mut Utility{
        &mut self.util_bar
    }
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        let text = self.util_bar.text.clone();
        match mode{
            Mode::Utility(UtilityKind::FindReplace) => {
                Paragraph::new(self.util_bar.view.text(&text))
            }
            _ => Paragraph::new(self.util_bar.view.text(&text))
        }
    }
}

#[derive(Default)]
struct UtilityPromptWidget{
    rect: Rect
}
impl UtilityPromptWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Utility(UtilityKind::Goto) => Paragraph::new(GOTO_PROMPT),
            Mode::Utility(UtilityKind::FindReplace) => Paragraph::new(FIND_PROMPT),
            Mode::Utility(UtilityKind::Command) => Paragraph::new(COMMAND_PROMPT),
            _ => Paragraph::new("")
        }
    }
}

#[derive(Default)]
struct UtilityAlternatePromptWidget{
    rect: Rect
}
impl UtilityAlternatePromptWidget{
    pub fn widget(&self, mode: Mode) -> Paragraph<'static>{
        match mode{
            Mode::Utility(UtilityKind::FindReplace) => {
                Paragraph::new(REPLACE_PROMPT)
            },
            _ => Paragraph::new("")
        }
    }
}

#[derive(Default)]
pub struct DocumentCursorPositionWidget{
    rect: Rect,
    document_cursor_position: Option<Position>
}
impl DocumentCursorPositionWidget{
    pub fn set(&mut self, cursor_position: Position){
        self.document_cursor_position = Some(cursor_position);
    }
    pub fn widget(&self) -> Paragraph<'static>{
        let position = match self.document_cursor_position{
            Some(cursor_position) => {
                format!(
                    "{}:{}",
                    cursor_position.y(),
                    cursor_position.x()
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
}

#[derive(Default)]
pub struct LineNumberWidget{
    rect: Rect,
    line_numbers_in_view: String,
}
impl LineNumberWidget{
    pub fn set(&mut self, line_numbers: String){
        self.line_numbers_in_view = line_numbers;
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.line_numbers_in_view.clone())
            .style(Style::default().fg(Color::Rgb(100, 100, 100)))
            .alignment(Alignment::Right)
    }
}

#[derive(Default)]
pub struct FileNameWidget{
    rect: Rect,
    file_name: Option<String>
}
impl FileNameWidget{
    pub fn set(&mut self, file_name: Option<String>){
        self.file_name = file_name;
    }
    pub fn widget(&self) -> Paragraph<'static>{
        let file_name = match &self.file_name{
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
}

#[derive(Default)]
pub struct ModifiedIndicatorWidget{
    rect: Rect,
    document_modified_status: bool
}
impl ModifiedIndicatorWidget{
    pub fn set(&mut self, modified: bool){
        self.document_modified_status = modified;
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(MODIFIED_INDICATOR)
            .alignment(Alignment::Left)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
                    .bold()
            )
    }
}

#[derive(Default)]
pub struct DocumentWidget{
    rect: Rect,
    doc_length: usize,
    client_cursor_position: Option<Position>,
    text_in_view: String,
}
impl DocumentWidget{
    pub fn set_length(&mut self, document_length: usize){
        self.doc_length = document_length;
    }
    pub fn rect(&self) -> Rect{
        self.rect
    }
    pub fn set_text_in_view(&mut self, text: String){
        self.text_in_view = text;
    }
    pub fn set_client_cursor_position(&mut self, positions: Vec<Position>){
        if !positions.is_empty(){
            self.client_cursor_position = Some(*positions.last().unwrap());
        }else{
            self.client_cursor_position = None;
        }
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.text_in_view.clone())
    }
}
impl ratatui::widgets::Widget for DocumentWidget{
    fn render(self, _area: Rect, _buf: &mut ratatui::prelude::Buffer){
        
    }
}

struct SpaceModeWidget{
    rect: Rect,
    widest_element_len: u16,    //+2 for border //the number of chars in the widest option in the space menu
    num_elements: u16,  //+2 for border //the number of options in the space menu
}
impl SpaceModeWidget{
    fn new() -> Self{
        Self{
            rect: Rect::default(), 
            widest_element_len: 46, 
            num_elements: 6
        }
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(
            concat!(
                " r  rename symbol(not implemented)\n",
                " b  insert debug breakpoint(not implemented)\n",   //widest element len 44
                " p  increment primary selection\n",
                " c  center cursor vertically in view"
            )   //num elements 4
        )
            .block(ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::all())
                .title("context menu"))
            .style(Style::new().bg(Color::Rgb(20, 20, 20)))
    }
}



#[derive(Default)]
pub struct StatusBar{
    modified_indicator: ModifiedIndicatorWidget,
    file_name: FileNameWidget,
    cursor_position: DocumentCursorPositionWidget,
}
#[derive(Default)]
pub struct UtilBar{
    prompt: UtilityPromptWidget,
    alternate_prompt: UtilityAlternatePromptWidget,
    widget: UtilityWidget,
    alternate_widget: UtilityAlternateWidget,
}
#[derive(Default)]
pub struct DocumentViewport{
    document_widget: DocumentWidget,
    line_number_widget: LineNumberWidget,
}



pub struct UserInterface{
    terminal_size: Rect,
    display_line_numbers: bool,
    display_status_bar: bool,
    util_bar_alternate_focused: bool,
    document_viewport: DocumentViewport,
    status_bar: StatusBar,
    util_bar: UtilBar,
    space_mode_widget: SpaceModeWidget,
}
impl UserInterface{
    pub fn new(terminal_size: Rect) -> Self{
        Self{
            terminal_size,
            display_line_numbers: true,
            display_status_bar: true,
            util_bar_alternate_focused: false,
            document_viewport: DocumentViewport::default(),
            status_bar: StatusBar::default(),
            util_bar: UtilBar::default(),
            space_mode_widget: SpaceModeWidget::new(),
        }
    }
    pub fn set_terminal_size(&mut self, width: u16, height: u16){
        self.terminal_size.width = width;
        self.terminal_size.height = height;
    }
    pub fn toggle_line_numbers(&mut self){
        self.display_line_numbers = !self.display_line_numbers;
    }
    pub fn toggle_status_bar(&mut self){
        self.display_status_bar = !self.display_status_bar;
    }
    pub fn util_bar_alternate_focused(&self) -> bool{
        self.util_bar_alternate_focused
    }
    pub fn set_util_bar_alternate_focused(&mut self, util_bar_alternate_focused: bool){
        self.util_bar_alternate_focused = util_bar_alternate_focused
    }
    pub fn document_widget(&self) -> &DocumentWidget{
        &self.document_viewport.document_widget
    }
    pub fn document_widget_mut(&mut self) -> &mut DocumentWidget{
        &mut self.document_viewport.document_widget
    }
    pub fn line_number_widget_mut(&mut self) -> &mut LineNumberWidget{
        &mut self.document_viewport.line_number_widget
    }
    pub fn modified_indicator_widget_mut(&mut self) -> &mut ModifiedIndicatorWidget{
        &mut self.status_bar.modified_indicator
    }
    pub fn file_name_widget_mut(&mut self) -> &mut FileNameWidget{
        &mut self.status_bar.file_name
    }
    pub fn document_cursor_position_widget_mut(&mut self) -> &mut DocumentCursorPositionWidget{
        &mut self.status_bar.cursor_position
    }
    pub fn util_bar_widget(&self) -> &UtilityWidget{
        &self.util_bar.widget
    }
    pub fn util_bar_widget_mut(&mut self) -> &mut UtilityWidget{
        &mut self.util_bar.widget
    }
    pub fn util_bar_alternate_widget(&self) -> &UtilityAlternateWidget{
        &self.util_bar.alternate_widget
    }
    pub fn util_bar_alternate_widget_mut(&mut self) -> &mut UtilityAlternateWidget{
        &mut self.util_bar.alternate_widget
    }



    fn layout_viewport(&self, mode: Mode) -> std::rc::Rc<[Rect]>{
        // layout of viewport rect (the whole terminal screen)
        Layout::default()
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
                            Mode::Utility(_) => 1,
                            Mode::Insert
                            | Mode::Space => if self.display_status_bar{1}else{0}
                        }
                    )
                ]
            )
            .split(self.terminal_size)
    }
    fn layout_document_and_line_number(&self, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of document + line num rect
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // line number left padding
                    //Constraint::Length(if self.display_line_numbers{1}else{0}),
                    // line number rect width
                    Constraint::Length(
                        if self.display_line_numbers{
                            count_digits(self.document_viewport.document_widget.doc_length)
                        }else{0}
                    ),
                    // line number right padding
                    Constraint::Length(if self.display_line_numbers{1}else{0}),
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
                    // modified indicator width
                    Constraint::Max(
                        if self.status_bar.modified_indicator.document_modified_status{
                            MODIFIED_INDICATOR.len() as u16
                        }else{0}
                    ),
                    //TODO: num selections widget
                    //
                    // file_name width
                    Constraint::Max(
                        if let Some(file_name) = &self.status_bar.file_name.file_name{
                            file_name.len() as u16
                        }else{0}
                    ),
                    // cursor position indicator width
                    Constraint::Min(0)
                ]
            )
            .split(rect)
    }
    fn layout_util_bar(&self, mode: Mode, rect: Rect) -> std::rc::Rc<[Rect]>{
        // layout of util rect (goto/find/command/save as)
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                vec![
                    // util bar prompt width
                    Constraint::Length(
                        match mode{
                            Mode::Utility(UtilityKind::Goto) => GOTO_PROMPT.len() as u16,
                            Mode::Utility(UtilityKind::FindReplace) => FIND_PROMPT.len() as u16,
                            Mode::Utility(UtilityKind::Command) => COMMAND_PROMPT.len() as u16,
                            Mode::Utility(UtilityKind::Warning(_))
                            | Mode::Insert
                            | Mode::Space => 0
                        }
                    ),
                    // util bar rect width
                    Constraint::Length(
                        match mode{
                            Mode::Insert
                            | Mode::Space
                            | Mode::Utility(UtilityKind::Warning(_)) => rect.width,
                            Mode::Utility(UtilityKind::Goto) => rect.width - GOTO_PROMPT.len() as u16,
                            Mode::Utility(UtilityKind::Command) => rect.width - COMMAND_PROMPT.len() as u16,                            
                            Mode::Utility(UtilityKind::FindReplace) => (rect.width / 2) - FIND_PROMPT.len() as u16,
                        }
                    ),
                    // util bar alternate prompt width
                    Constraint::Length(
                        match mode{
                            Mode::Utility(UtilityKind::FindReplace) => REPLACE_PROMPT.len() as u16,
                            _ => 0
                        }
                    ),
                    // util bar alternate rect width
                    Constraint::Length(
                        match mode{
                            Mode::Utility(UtilityKind::FindReplace) => (rect. width / 2).saturating_sub(REPLACE_PROMPT.len() as u16),
                            _ => 0
                        }
                    ),
                    // used to fill in space when other two are 0 length
                    Constraint::Length(0)
                ]
            )
            .split(rect)
    }
    pub fn update_layouts(&mut self, mode: Mode){
        let viewport_rect = self.layout_viewport(mode);
        let document_and_line_num_rect = self.layout_document_and_line_number(viewport_rect[0]);
        let status_bar_rect = self.layout_status_bar(viewport_rect[1]);
        let util_rect = self.layout_util_bar(mode, viewport_rect[2]);

        self.document_viewport.line_number_widget.rect = document_and_line_num_rect[0];
        // dont have to set line num right padding(document_and_line_num_rect[1])
        self.document_viewport.document_widget.rect = document_and_line_num_rect[2];
        self.status_bar.modified_indicator.rect = status_bar_rect[0];
        self.status_bar.file_name.rect = status_bar_rect[1];
        self.status_bar.cursor_position.rect = status_bar_rect[2];
        self.util_bar.prompt.rect = util_rect[0];
        self.util_bar.widget.rect = util_rect[1];
        self.util_bar.alternate_prompt.rect = util_rect[2];
        self.util_bar.alternate_widget.rect = util_rect[3];
        self.space_mode_widget.rect = sized_centered_rect(self.space_mode_widget.widest_element_len, self.space_mode_widget.num_elements, self.terminal_size);

        self.update_util_bar_width(mode);
    }

    // may not be needed if alternate util bar removed
    fn update_util_bar_width(&mut self, mode: Mode){
        match mode{ //TODO: can these be set from relevant fns in application.rs? display_line_numbers, display_status_bar, resize, any mode change, etc
            Mode::Utility(UtilityKind::Command) 
            | Mode::Utility(UtilityKind::Goto) 
            | Mode::Utility(UtilityKind::FindReplace) => {
                self.util_bar.widget.util_bar.set_widget_width(self.util_bar.widget.rect.width);
                self.util_bar.alternate_widget.util_bar.set_widget_width(self.util_bar.alternate_widget.rect.width);
            }
            _ => {
                self.util_bar.widget.util_bar.set_widget_width(0);
                self.util_bar.alternate_widget.util_bar.set_widget_width(0);
            }
        }
    }

    pub fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, mode: Mode) -> Result<(), Box<dyn Error>>{        
        let _ = terminal.draw(  // Intentionally discarding `CompletedFrame`
            |frame| {
                // always render
                frame.render_widget(self.document_viewport.document_widget.widget(), self.document_viewport.document_widget.rect);
                
                // conditionally render
                if self.display_line_numbers{
                    frame.render_widget(self.document_viewport.line_number_widget.widget(), self.document_viewport.line_number_widget.rect);
                }
                if self.display_status_bar{
                    frame.render_widget(self.status_bar.modified_indicator.widget(), self.status_bar.modified_indicator.rect);
                    frame.render_widget(self.status_bar.file_name.widget(), self.status_bar.file_name.rect);
                    // TODO: add widget for number of selections
                    frame.render_widget(self.status_bar.cursor_position.widget(), self.status_bar.cursor_position.rect);
                }

                // render according to mode
                // cursor rendering will prob change from frame.render_widget style to handling cursor drawing in each widget
                match mode{
                    Mode::Insert => {
                        if let Some(pos) = self.document_viewport.document_widget.client_cursor_position{
                            frame.set_cursor(
                                self.document_viewport.document_widget.rect.x + pos.x() as u16,
                                self.document_viewport.document_widget.rect.y + pos.y() as u16
                            )
                        }
                    }
                    Mode::Utility(UtilityKind::Goto | UtilityKind::Command) => {
                        frame.render_widget(self.util_bar.prompt.widget(mode), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.widget.widget(mode), self.util_bar.widget.rect);
                        frame.set_cursor(
                            self.util_bar.widget.rect.x + self.util_bar.widget.util_bar.cursor_position().saturating_sub(self.util_bar.widget.util_bar.view.horizontal_start() as u16),
                            self.terminal_size.height
                        );
                    }
                    Mode::Utility(UtilityKind::FindReplace) => {
                        frame.render_widget(self.util_bar.prompt.widget(mode), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.widget.widget(mode), self.util_bar.widget.rect);
                        frame.render_widget(self.util_bar.alternate_prompt.widget(mode), self.util_bar.alternate_prompt.rect);
                        frame.render_widget(self.util_bar.alternate_widget.widget(mode), self.util_bar.alternate_widget.rect);
                        frame.set_cursor(
                            if self.util_bar_alternate_focused{
                                self.util_bar.alternate_widget.rect.x + self.util_bar.alternate_widget.util_bar.cursor_position()
                                    .saturating_sub(self.util_bar.alternate_widget.util_bar.view.horizontal_start() as u16)
                            }else{
                                self.util_bar.widget.rect.x + self.util_bar.widget.util_bar.cursor_position().saturating_sub(self.util_bar.widget.util_bar.view.horizontal_start() as u16)
                            },
                            self.terminal_size.height
                        );
                    }
                    Mode::Utility(UtilityKind::Warning(_)) => {
                        frame.render_widget(self.util_bar.prompt.widget(mode), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.widget.widget(mode), self.util_bar.widget.rect);
                    }
                    Mode::Space => {
                        frame.render_widget(ratatui::widgets::Clear, self.space_mode_widget.rect);
                        frame.render_widget(self.space_mode_widget.widget(), self.space_mode_widget.rect);
                    }
                }
            }
        )?;

        Ok(())
    }
}

//fn centered_rect(percent_x: u16, percent_y: u16, r: /*ratatui::prelude::*/Rect) -> /*ratatui::prelude::*/Rect{
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

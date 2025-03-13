use crate::application::Mode;
use document_viewport::DocumentViewport;
use popups::Popups;
use util_bar::UtilBar;
use status_bar::StatusBar;
use std::error::Error;
use ratatui::Terminal;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::layout::{Direction, Layout, Constraint};



mod document_viewport;
mod status_bar;
mod util_bar;
mod interactive_text_box;
mod popups;



pub struct UserInterface{
    pub terminal_size: Rect,
    pub document_viewport: DocumentViewport,
    pub status_bar: StatusBar,
    pub util_bar: UtilBar,
    pub popups: Popups,
}
impl UserInterface{
    pub fn new(terminal_size: Rect) -> Self{
        Self{
            terminal_size,
            document_viewport: DocumentViewport::default(),
            status_bar: StatusBar::default(),
            util_bar: UtilBar::default(),
            popups: Popups::new(),
        }
    }
    pub fn set_terminal_size(&mut self, width: u16, height: u16){
        self.terminal_size.width = width;
        self.terminal_size.height = height;
    }

    fn layout_terminal(&self, mode: &Mode) -> std::rc::Rc<[Rect]>{       //TODO: maybe rename layout_terminal_vertical_ui_components
        // layout of the whole terminal screen
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                vec![
                    // document + line num rect height
                    Constraint::Min(0),
                    // status bar rect height
                    Constraint::Length(if self.status_bar.display{1}else{0}),
                    // util(goto/find/command) bar rect height
                    Constraint::Length(
                        match mode{
                            Mode::Warning(_) | Mode::Command | Mode::Find | Mode::Goto | Mode::Notify | Mode::Split => 1,
                            Mode::Insert => if /*self.util_bar.utility_widget.display_copied_indicator || */self.status_bar.display{1}else{0}
                            Mode::View => if self.status_bar.display{1}else{0}
                        }
                    )
                ]
            )
            .split(self.terminal_size)
    }
    pub fn update_layouts(&mut self, mode: &Mode){
        let terminal_rect = self.layout_terminal(mode);
        let document_viewport_rect = self.document_viewport.layout(terminal_rect[0]);
        let status_bar_rect = self.status_bar.layout(terminal_rect[1]);
        let util_rect = self.util_bar.layout(mode, terminal_rect[2]);

        self.document_viewport.line_number_widget.rect = document_viewport_rect[0];
        // dont have to set line num right padding(document_and_line_num_rect[1])
        self.document_viewport.document_widget.rect = document_viewport_rect[2];
        self.status_bar.file_name_widget.rect = status_bar_rect[0];
        self.status_bar.modified_indicator_widget.rect = status_bar_rect[1];
        //selections padding(status_bar_rect[2])
        self.status_bar.selections_widget.rect = status_bar_rect[2];    //[3] with selections padding enabled
        //selections padding(status_bar_rect[4])
        self.status_bar.document_cursor_position_widget.rect = status_bar_rect[3];  //[5] with selections padding enabled
        self.util_bar.prompt.rect = util_rect[0];
        self.util_bar.utility_widget.rect = util_rect[1];
        self.popups.view_mode_widget.rect = sized_centered_rect(self.popups.view_mode_widget.widest_element_len, self.popups.view_mode_widget.num_elements, self.terminal_size);
        self.popups.goto_mode_widget.rect = sized_centered_rect(self.popups.goto_mode_widget.widest_element_len, self.popups.goto_mode_widget.num_elements, self.terminal_size);

        self.util_bar.update_width(mode);
    }

    pub fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, mode: &Mode) -> Result<(), Box<dyn Error>>{
        let _ = terminal.draw(  // Intentionally discarding `CompletedFrame`
            |frame| {
                // always render
                frame.render_widget(self.document_viewport.document_widget.widget(), self.document_viewport.document_widget.rect);
                frame.render_widget(self.document_viewport.highlighter.clone(), self.document_viewport.document_widget.rect);
                
                // conditionally render
                if self.document_viewport.display_line_numbers{
                    frame.render_widget(self.document_viewport.line_number_widget.widget(), self.document_viewport.line_number_widget.rect);
                }
                if self.status_bar.display{
                    frame.render_widget(self.status_bar.modified_indicator_widget.widget(), self.status_bar.modified_indicator_widget.rect);
                    frame.render_widget(self.status_bar.file_name_widget.widget(), self.status_bar.file_name_widget.rect);
                    frame.render_widget(self.status_bar.selections_widget.widget(), self.status_bar.selections_widget.rect);
                    frame.render_widget(self.status_bar.document_cursor_position_widget.widget(), self.status_bar.document_cursor_position_widget.rect);
                }

                // render according to mode
                match mode{
                    Mode::Insert => {
                        // built in cursor handling. now handling cursor rendering ourselves
                        // frame.set_cursor_position((
                        //     self.document_viewport.document_widget.rect.x + pos.x() as u16,
                        //     self.document_viewport.document_widget.rect.y + pos.y() as u16
                        // ))
                        
                        // clear_copied_indicator exists because copied_indicator widget rendering needs to persist for an entire loop cycle(until next keypress)
                        //if self.util_bar.utility_widget.clear_copied_indicator{
                        //    self.util_bar.utility_widget.display_copied_indicator = false;
                        //    self.util_bar.utility_widget.clear_copied_indicator = false;
                        //}
                        //if self.util_bar.utility_widget.display_copied_indicator{
                        //    frame.render_widget(self.util_bar.utility_widget.widget(mode.clone()), self.util_bar.utility_widget.rect);
                        //    self.util_bar.utility_widget.clear_copied_indicator = true;
                        //}
                    }
                    Mode::Goto => {
                        frame.render_widget(self.util_bar.prompt.widget(mode), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.utility_widget.widget(mode.clone()), self.util_bar.utility_widget.rect);
                        
                        self.util_bar.highlighter.selection = Some(self.util_bar.utility_widget.text_box.selection.clone());    //TODO: maybe these should be moved into Application::update_ui_data_util_bar
                        self.util_bar.highlighter.cursor = self.util_bar.utility_widget.text_box.cursor_position();             //TODO: maybe these should be moved into Application::update_ui_data_util_bar
                        frame.render_widget(self.util_bar.highlighter.clone(), self.util_bar.utility_widget.rect);

                        //TODO: render a pop up widget that displays the available keys to the user //do this for all util modes
                        //config.rs should have a const that can enable/disable this behavior. SHOW_UTIL_KEY_POPUP

                        // if SHOW_CONTEXTUAL_KEYBINDS{     //should displaying the popup be optional?...
                        frame.render_widget(ratatui::widgets::Clear, self.popups.goto_mode_widget.rect);
                        frame.render_widget(self.popups.goto_mode_widget.widget(), self.popups.goto_mode_widget.rect);
                        // }
                    }
                    /*Mode::Goto | */Mode::Command => {
                        frame.render_widget(self.util_bar.prompt.widget(mode), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.utility_widget.widget(mode.clone()), self.util_bar.utility_widget.rect);
                        
                        self.util_bar.highlighter.selection = Some(self.util_bar.utility_widget.text_box.selection.clone());    //TODO: maybe these should be moved into Application::update_ui_data_util_bar
                        self.util_bar.highlighter.cursor = self.util_bar.utility_widget.text_box.cursor_position();             //TODO: maybe these should be moved into Application::update_ui_data_util_bar
                        frame.render_widget(self.util_bar.highlighter.clone(), self.util_bar.utility_widget.rect);

                        //TODO: render a pop up widget that displays the available keys to the user //do this for all util modes
                        //config.rs should have a const that can enable/disable this behavior. SHOW_UTIL_KEY_POPUP
                    }
                    Mode::Find => {
                        frame.render_widget(self.util_bar.prompt.widget(mode), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.utility_widget.widget(mode.clone()), self.util_bar.utility_widget.rect);

                        self.util_bar.highlighter.selection = Some(self.util_bar.utility_widget.text_box.selection.clone());    //TODO: maybe these should be moved into Application::update_ui_data_util_bar
                        self.util_bar.highlighter.cursor = self.util_bar.utility_widget.text_box.cursor_position();             //TODO: maybe these should be moved into Application::update_ui_data_util_bar
                        frame.render_widget(self.util_bar.highlighter.clone(), self.util_bar.utility_widget.rect);
                    }
                    Mode::Split => {
                        frame.render_widget(self.util_bar.prompt.widget(mode), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.utility_widget.widget(mode.clone()), self.util_bar.utility_widget.rect);

                        self.util_bar.highlighter.selection = Some(self.util_bar.utility_widget.text_box.selection.clone());    //TODO: maybe these should be moved into Application::update_ui_data_util_bar
                        self.util_bar.highlighter.cursor = self.util_bar.utility_widget.text_box.cursor_position();             //TODO: maybe these should be moved into Application::update_ui_data_util_bar
                        frame.render_widget(self.util_bar.highlighter.clone(), self.util_bar.utility_widget.rect);
                    }
                    Mode::Warning(_) => {
                        //frame.render_widget(self.util_bar.prompt.widget(mode.clone()), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.utility_widget.widget(mode.clone()), self.util_bar.utility_widget.rect);
                    }
                    Mode::Notify => {
                        //frame.render_widget(self.util_bar.prompt.widget(mode.clone()), self.util_bar.prompt.rect);
                        frame.render_widget(self.util_bar.utility_widget.widget(mode.clone()), self.util_bar.utility_widget.rect);
                    }
                    Mode::View => {
                        // if SHOW_CONTEXTUAL_KEYBINDS{     //should displaying the popup be optional?...
                        frame.render_widget(ratatui::widgets::Clear, self.popups.view_mode_widget.rect);
                        frame.render_widget(self.popups.view_mode_widget.widget(), self.popups.view_mode_widget.rect);
                        // }
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

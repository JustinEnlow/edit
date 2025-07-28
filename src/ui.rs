use document_viewport::DocumentViewport;
use popups::Popups;
use util_bar::UtilBar;
use status_bar::StatusBar;
use ratatui::layout::Rect;
use ratatui::layout::{Direction, Layout, Constraint};



mod document_viewport;
mod status_bar;
pub mod util_bar;
mod interactive_text_box;
mod popups;



pub struct UserInterface{
    pub terminal_size: Rect,    //TODO: could this be passed in to needed functions instead?... //TODO: this can prob be removed if terminal.size() called in update_layouts...
    pub document_viewport: DocumentViewport,
    pub status_bar: StatusBar,
    pub util_bar: UtilBar,
    pub popups: Popups,
    pub error_mode_text: String,
    pub warning_mode_text: String,
    pub notify_mode_text: String,
    pub info_mode_text: String,
}
impl UserInterface{
    pub fn new(terminal_size: Rect) -> Self{
        Self{
            terminal_size,  //TODO: this can prob be removed if terminal.size() called in update_layouts...
            document_viewport: DocumentViewport::default(),
            status_bar: StatusBar::default(),
            util_bar: UtilBar::default(),
            popups: Popups::new(),
            error_mode_text: String::new(),
            warning_mode_text: String::new(),
            notify_mode_text: String::new(),
            info_mode_text: String::new(),
        }
    }
    //TODO: this can prob be removed if terminal.size() called in update_layouts...
    pub fn set_terminal_size(&mut self, width: u16, height: u16){
        self.terminal_size.width = width;
        self.terminal_size.height = height;
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

pub fn sized_centered_rect(x: u16, y: u16, r: Rect) -> Rect{
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

pub fn count_digits(mut n: usize) -> u16{
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

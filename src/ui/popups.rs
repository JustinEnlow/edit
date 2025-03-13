use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color};



//
pub struct GotoModeWidget{
    pub rect: Rect,
    pub widest_element_len: u16,
    pub num_elements: u16,
}
impl GotoModeWidget{
    fn new() -> Self{
        Self{
            rect: Rect::default(),
            widest_element_len: 42,
            num_elements: 5
        }
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(
            concat!(
                " ⏎  go to specified line number\n",
                " ↑  move up specified number of times\n",
                " ↓  move down specified number of times\n",    //widest element len 40
            )
        )
            .block(ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::all())
                .title("context menu"))
            .style(Style::new().bg(Color::Rgb(20, 20, 20)))
    }
}
//

pub struct ViewModeWidget{
    pub rect: Rect,
    pub widest_element_len: u16,    //+2 for border //the number of chars in the widest option in the space menu
    pub num_elements: u16,  //+2 for border //the number of options in the space menu
}
impl ViewModeWidget{
    fn new() -> Self{
        Self{
            rect: Rect::default(), 
            widest_element_len: 48, //+2 for border(actually did +3 for some padding as well)
            num_elements: 10    //+2 for border
        }
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(
            concat!(    //TODO: generate keybind display string from keybind.rs
                " v  center vertically around primary cursor\n",
                " h  center horizontally around primary cursor\n", //widest element len 45
                " t  align with primary cursor at top\n",
                " b  align with primary cursor at bottom\n",
                " ↑  scroll up\n",
                " ↓  scroll down\n",
                " ←  scroll left\n",
                " →  scroll right\n",
                //  //num elements 8
            )
        )
            .block(ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::all())
                .title("context menu"))
            .style(Style::new().bg(Color::Rgb(20, 20, 20)))
    }
}

// TODO: suggestions widget

/// Container type for popup style widgets.
pub struct Popups{
    pub view_mode_widget: ViewModeWidget,
    pub goto_mode_widget: GotoModeWidget,
}
impl Popups{
    pub fn new() -> Self{
        Self{
            view_mode_widget: ViewModeWidget::new(),
            goto_mode_widget: GotoModeWidget::new(),
        }
    }
}

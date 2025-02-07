use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color};



pub struct ViewModeWidget{
    pub rect: Rect,
    pub widest_element_len: u16,    //+2 for border //the number of chars in the widest option in the space menu
    pub num_elements: u16,  //+2 for border //the number of options in the space menu
}
impl ViewModeWidget{
    fn new() -> Self{
        Self{
            rect: Rect::default(), 
            widest_element_len: 47,//46, 
            num_elements: 10//6
        }
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(
            concat!(    //TODO: generate keybind display string from keybind.rs
                //" r  rename symbol(not implemented)\n",
                //" b  insert debug breakpoint(not implemented)\n",   //widest element len 44
                //" p  increment primary selection\n",
                //" c  center cursor vertically in view"
                //TODO when changed to ViewWidget:
                " v  center vertically around primary cursor\n",
                " h  center horizontally around primary cursor\n", //widest element len 45
                " t  align with primary cursor at top\n",
                " b  align with primary cursor at bottom\n",
                " ↑  scroll up\n",
                " ↓  scroll down\n",
                " ←  scroll left\n",
                " →  scroll right\n",
                //  //num elements 8
            )   //num elements 4
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
}
impl Popups{
    pub fn new() -> Self{
        Self{
            view_mode_widget: ViewModeWidget::new(),
        }
    }
}

use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color};



pub struct SpaceModeWidget{
    pub rect: Rect,
    pub widest_element_len: u16,    //+2 for border //the number of chars in the widest option in the space menu
    pub num_elements: u16,  //+2 for border //the number of options in the space menu
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

/// Container type for popup style widgets.
pub struct Popups{
    pub space_mode_widget: SpaceModeWidget,
}
impl Popups{
    pub fn new() -> Self{
        Self{
            space_mode_widget: SpaceModeWidget::new(),
        }
    }
}

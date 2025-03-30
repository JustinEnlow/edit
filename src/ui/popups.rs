use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color};


//TODO: always make sure to add new widgets to update_layouts fn in ui.rs, so that they have screen space assigned to them

/* TODO: consider
    menu commands should have:
        - a keybind             (the key the user should press to select this option)(assigned in config, and kept up to date in this module)
        - a command string      (a short explanation of the command)(assigned in config or external utility?)
        - a source              (edit_core or external utility name)(should have a SHOW_SOURCE toggle in config...)

    menu:
        " <keybind>  command string  (source)\n"
        " <keybind>  command string  (source)\n"
*/

//TODO: maybe we could have one popup widget, whose rect, widest_element_len and num_elements can be determined from
//the content passed in to it. this may ease the addition of new functionality/popup contents
// let object_mode_widget = PopupWidget::new(object_mode_menu_content);
//we could lose mode specific tui styling this way...


//
pub struct ObjectModeWidget{
    pub rect: Rect,
    pub widest_element_len: u16,
    pub num_elements: u16,
}
impl ObjectModeWidget{
    fn new() -> Self{
        Self{
            rect: Rect::default(),
            widest_element_len: 33,
            num_elements: 8
        }
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(
            concat!(
                " w  word\n",
                " s  sentence\n",
                " p  paragraph\n",
                " b  surrounding bracket pair\n",
                " e  exclusive surrounding pair\n", //widest element len 31
                " i  inclusive surrounding pair\n",
                //TODO?: surrounding whitespace
            )
        )
        .block(ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::all())
            .title("context menu"))
        .style(Style::new().bg(Color::Rgb(20, 20, 20)))
    }
}
//

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
    pub object_mode_widget: ObjectModeWidget,
}
impl Popups{
    pub fn new() -> Self{
        Self{
            view_mode_widget: ViewModeWidget::new(),
            goto_mode_widget: GotoModeWidget::new(),
            object_mode_widget: ObjectModeWidget::new(),
        }
    }
}

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

const OBJECT_MODE_MENU: &str = concat!(
    " w  word  (core)\n",
    " s  sentence  (core)\n",
    " p  paragraph  (core)\n",
    " b  surrounding bracket pair  (core)\n",
    " e  exclusive surrounding pair  (core)\n",
    " i  inclusive surrounding pair  (core)\n",
    //TODO?: surrounding whitespace
);
const GOTO_MODE_MENU: &str = concat!(
    " ⏎  go to specified line number  (core)\n",
    " ↑  move up specified number of times  (core)\n",
    " ↓  move down specified number of times  (core)\n",
);
const VIEW_MODE_MENU: &str = concat!(
    " v  center vertically around primary cursor  (core)\n",
    " h  center horizontally around primary cursor  (core)\n",
    " t  align with primary cursor at top  (core)\n",
    " b  align with primary cursor at bottom  (core)\n",
    " ↑  scroll up  (core)\n",
    " ↓  scroll down  (core)\n",
    " ←  scroll left  (core)\n",
    " →  scroll right  (core)\n",
);



//
//pub struct ObjectModeWidget{
//    pub rect: Rect,
//    pub widest_element_len: u16,
//    pub num_elements: u16,
//}
//impl ObjectModeWidget{
//    fn new() -> Self{
//        Self{
//            rect: Rect::default(),
//            widest_element_len: 33,
//            num_elements: 8
//        }
//    }
//    pub fn widget(&self) -> Paragraph<'static>{
//        Paragraph::new(
//            concat!(
//                " w  word\n",
//                " s  sentence\n",
//                " p  paragraph\n",
//                " b  surrounding bracket pair\n",
//                " e  exclusive surrounding pair\n", //widest element len 31
//                " i  inclusive surrounding pair\n",
//                //TODO?: surrounding whitespace
//            )
//        )
//        .block(ratatui::widgets::Block::default()
//            .borders(ratatui::widgets::Borders::all())
//            .title("context menu"))
//        .style(Style::new().bg(Color::Rgb(20, 20, 20)))
//    }
//}
//

//
//pub struct GotoModeWidget{
//    pub rect: Rect,
//    pub widest_element_len: u16,
//    pub num_elements: u16,
//}
//impl GotoModeWidget{
//    fn new() -> Self{
//        Self{
//            rect: Rect::default(),
//            widest_element_len: 42,
//            num_elements: 5
//        }
//    }
//    pub fn widget(&self) -> Paragraph<'static>{
//        Paragraph::new(
//            concat!(
//                " ⏎  go to specified line number\n",
//                " ↑  move up specified number of times\n",
//                " ↓  move down specified number of times\n",    //widest element len 40
//            )
//        )
//            .block(ratatui::widgets::Block::default()
//                .borders(ratatui::widgets::Borders::all())
//                .title("context menu"))
//            .style(Style::new().bg(Color::Rgb(20, 20, 20)))
//    }
//}
//

//pub struct ViewModeWidget{
//    pub rect: Rect,
//    pub widest_element_len: u16,    //+2 for border //the number of chars in the widest option in the space menu
//    pub num_elements: u16,  //+2 for border //the number of options in the space menu
//}
//impl ViewModeWidget{
//    fn new() -> Self{
//        Self{
//            rect: Rect::default(), 
//            widest_element_len: 48, //+2 for border(actually did +3 for some padding as well)
//            num_elements: 10    //+2 for border
//        }
//    }
//    pub fn widget(&self) -> Paragraph<'static>{
//        Paragraph::new(
//            concat!(    //TODO: generate keybind display string from keybind.rs
//                " v  center vertically around primary cursor\n",
//                " h  center horizontally around primary cursor\n", //widest element len 45
//                " t  align with primary cursor at top\n",
//                " b  align with primary cursor at bottom\n",
//                " ↑  scroll up\n",
//                " ↓  scroll down\n",
//                " ←  scroll left\n",
//                " →  scroll right\n",
//                //  //num elements 8
//            )
//        )
//            .block(ratatui::widgets::Block::default()
//                .borders(ratatui::widgets::Borders::all())
//                .title("context menu"))
//            .style(Style::new().bg(Color::Rgb(20, 20, 20)))
//    }
//}

// TODO: suggestions widget

//TODO: maybe we could have one popup widget, whose rect, widest_element_len and num_elements can be determined from
//the content passed in to it. this may ease the addition of new functionality/popup contents
// let object_mode_widget = PopupWidget::new(object_mode_menu_content);
//NOTE: we could lose mode specific tui styling this way...
pub struct Popup{       //TODO: maybe this should be PopupMenu, since other popups, like suggestions, may have different behavior...
    pub rect: Rect,
    pub widest_element_len: u16,    //+2 for border //the number of chars in the widest option in the space menu
    pub num_elements: u16,  //+2 for border //the number of options in the space menu
    content: String,
    context_menu_title: String,
    //bg_color: Color   //if we want to keep mode specific styling for popup
    //fg_color: Color   //if we want to keep mode specific styling for popup
}
impl Popup{
    //TODO: how can we automate the padding around keybinds, commands, and sources?...
    //TODO: maybe take (keys: &[&str], commands: &[&str], sources: &[&str])...      //assert that all slices have same len
    pub fn new(content: &str, context_menu_title: &str) -> Self{
        //let content = format!(" KEY  COMMAND  SOURCE\n{}", content);    //prepend labels to content
        
        //get len of longest line and number of lines from content
        let lines = content.lines();
        let mut num_lines = 0;
        let mut longest_line_len = 0;
        for line in lines{
            if line.len() as u16 > longest_line_len{    //TODO: this seems to be counting chars, we need it to count graphemes, or more accurately, the number of terminal cells used for display. (wide graphemes may take up multiple terminal cells...)
                longest_line_len = line.len() as u16;
            }
            num_lines = num_lines + 1;
        }
        
        Self{
            rect: Rect::default(),
            widest_element_len: longest_line_len + 3,   //TODO: make a note why we need to add this number
            num_elements: num_lines + 2,                //TODO: make a note why we need to add this number
            content: String::from(content),
            context_menu_title: String::from(context_menu_title),
        }
    }
    pub fn widget(&self) -> Paragraph<'static>{
        Paragraph::new(self.content.clone())
            .block(ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::all())
                .title(self.context_menu_title.clone()))
            .style(Style::new().bg(Color::Rgb(20, 20, 20)))
    }
}

/// Container type for popup style widgets.
pub struct Popups{
    //pub view_mode_widget: ViewModeWidget,
    pub view_mode_widget: Popup,
    //pub goto_mode_widget: GotoModeWidget,
    pub goto_mode_widget: Popup,
    //pub object_mode_widget: ObjectModeWidget,
    pub object_mode_widget: Popup,
}
impl Popups{
    pub fn new() -> Self{
        Self{
            //view_mode_widget: ViewModeWidget::new(),
            view_mode_widget: Popup::new(VIEW_MODE_MENU, "View"),
            //goto_mode_widget: GotoModeWidget::new(),
            goto_mode_widget: Popup::new(GOTO_MODE_MENU, "Goto"),
            //object_mode_widget: ObjectModeWidget::new(),
            object_mode_widget: Popup::new(OBJECT_MODE_MENU, "Object"),
        }
    }
}

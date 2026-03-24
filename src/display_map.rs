use crate::{
    selections::Selections,
    display_area::DisplayArea
};

pub struct DisplayBufferItem;

pub struct Grapheme{
    bytes_utf8: Vec<u8>,    //or maybe chars: Vec<char>...
    buffer_item_type: DisplayBufferItem,
    display_width: u8,  //for _ in 0..display_width{/*push buffer_item_type color_bg/fg to highlights*/}
}

pub struct Highlight{
    color_bg: u8,   //Color
    color_fg: u8,   //Color
}

pub struct DisplayMap{
    pub display_buffer: Vec<Grapheme>,
    pub highlights: Vec<Highlight>  //1 for each cell in display_area
}
impl DisplayMap{
    fn new(selections: &Selections, /*expansions, */ display_area: &DisplayArea) -> DisplayMap{
        //if line is shorter than display_area.width, fill the rest with spaces(to allow for highlighting)
        //for every line, if at display_area.width, add a newline
        DisplayMap{display_buffer: Vec::new(), highlights: Vec::new()}
    }
    fn text() -> String{
        String::new()
    }
}

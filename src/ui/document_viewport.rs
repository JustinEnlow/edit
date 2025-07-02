use ratatui::layout::Rect;
use crate::position::Position;
use crate::selection2d::Selection2d;



//pub const LINE_NUMBER_PADDING: u16 = 1;



/// This is used to fill space between other widgets
#[derive(Default)]
pub struct Padding{
    pub rect: Rect,
}

#[derive(Default)]
pub struct LineNumberWidget{
    pub rect: Rect,
    pub text: String,
    pub show: bool,
}

#[derive(Default, Clone)]
pub struct DocumentWidget{
    pub rect: Rect,
    //pub doc_length: usize,  //used in DocumentViewport  //TODO: can this be set elsewhere?...maybe pass in to DocumentViewport::layout()?...
    pub text: String,
}

// render order matters. for example, always render cursors after selections, so that the cursor shows on top of the selection.
#[derive(Default, Clone)]
pub struct Highlighter{
    // debug highlights //bg color
    // lsp highlights   //fg color
    pub selections: Vec<Selection2d>,   //bg color
    pub primary_cursor: Option<Position>, //bg color + fg color?
    pub cursors: Vec<Position>, 
    // others idk
}

/// Container type for widgets in the document viewport.
#[derive(Default)]
pub struct DocumentViewport{
    pub line_number_widget: LineNumberWidget,
    pub padding: Padding,
    pub document_widget: DocumentWidget,
    pub highlighter: Highlighter,
}

use ratatui::layout::Rect;

/// This is used to fill space between other widgets
#[derive(Default)] pub struct Padding{pub rect: Rect}

#[derive(Default)] pub struct ReadOnlyWidget{
    pub rect: Rect,
    pub text: String,
    pub show: bool,
}

#[derive(Default)] pub struct ModeWidget{
    pub rect: Rect,
    pub text: String,
}

#[derive(Default)] pub struct SelectionsWidget{
    pub rect: Rect,
    pub text: String,
}

#[derive(Default)] pub struct CursorPositionWidget{
    pub rect: Rect,
    pub text: String,
}

#[derive(Default)] pub struct FileNameWidget{
    pub rect: Rect,
    pub text: String,
    pub show: bool
}

#[derive(Default)] pub struct ModifiedWidget{
    pub rect: Rect,
    pub text: String,
    pub show: bool,
}

/// Container type for widgets on the status bar.
#[derive(Default)] pub struct StatusBar{
    pub show: bool,
    pub read_only_widget: ReadOnlyWidget,
    pub padding_1: Padding,
    pub file_name_widget: FileNameWidget,
    pub padding_2: Padding,
    pub modified_widget: ModifiedWidget,
    pub padding_3: Padding,
    pub selections_widget: SelectionsWidget,
    pub padding_4: Padding,
    pub cursor_position_widget: CursorPositionWidget,
    pub padding_5: Padding,
    pub mode_widget: ModeWidget,
}

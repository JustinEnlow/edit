#[derive(Clone)] pub enum EditorAction{
    ModePop,
    ModePush(crate::mode_stack::StackMember),
    Resize(u16, u16),
    NoOpKeypress,
    NoOpEvent,
    Quit,
    QuitIgnoringChanges,
    Save,
    Copy,
    ToggleLineNumbers,
    ToggleStatusBar,
    OpenNewTerminalWindow,
}
impl EditorAction{
    fn action_name(&self) -> String{
        let name = match self{
            EditorAction::Copy => "copy",
            EditorAction::ModePop => "exit mode",
            EditorAction::ModePush(mode) => &format!("push {:?} to mode stack", mode),
            EditorAction::NoOpEvent => "no op event",
            EditorAction::NoOpKeypress => "no op keypress",
            EditorAction::OpenNewTerminalWindow => "open new terminal window",
            EditorAction::Quit => "quit",
            EditorAction::QuitIgnoringChanges => "force quit",
            EditorAction::Resize(x, y) => &format!("resize to {},{}", x, y),
            EditorAction::Save => "save",
            EditorAction::ToggleLineNumbers => "toggle line numbers",
            EditorAction::ToggleStatusBar => "toggle status bar"
        };
        name.to_string()
    }
}
#[derive(Clone, Debug)] pub enum SelectionAction{   //TODO?: have (all?) selection actions take an amount, for action repetition. MoveCursorDown(2) would move the cursor down two lines, if possible, or saturate at buffer end otherwise, and error if already at buffer end
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorWordBoundaryForward,  //TODO: this isn't working with count, for some reason. check move_cursor_word_boundary_backward impl to determine cause...
    MoveCursorWordBoundaryBackward, //TODO: this isn't working with count, for some reason. check move_cursor_word_boundary_forward impl to determine cause...
    MoveCursorLineEnd,
    MoveCursorHome,
    MoveCursorBufferStart,
    MoveCursorBufferEnd,
    MoveCursorPageUp,
    MoveCursorPageDown,
    ExtendSelectionUp,
    ExtendSelectionDown,
    ExtendSelectionLeft,
    ExtendSelectionRight,
    ExtendSelectionWordBoundaryBackward,    //TODO: this isn't working with count, for some reason. check extend_selection_word_boundary_backward impl to determine cause...
    ExtendSelectionWordBoundaryForward,     //TODO: this isn't working with count, for some reason. check extend_selection_word_boundary_forward impl to determine cause...
    ExtendSelectionLineEnd,
    ExtendSelectionHome,
        //TODO: ExtendSelectionBufferStart,
        //TODO: ExtendSelectionBufferEnd,
        //TODO: ExtendSelectionPageUp,
        //TODO: ExtendSelectionPageDown,
    SelectLine,           //TODO: this may benefit from using a count. would the next count # of lines including current
    SelectAll,
    CollapseSelectionToAnchor,
    CollapseSelectionToCursor,
    ClearNonPrimarySelections,
    AddSelectionAbove,    //TODO: this may benefit from using a count. would add count # of selections
    AddSelectionBelow,    //TODO: this may benefit from using a count. would add count # of selections
    RemovePrimarySelection,
    IncrementPrimarySelection,  //TODO: this may benefit from using a count. would increment primary selection index by 'count'
    DecrementPrimarySelection,  //TODO: this may benefit from using a count. would decrement primary selection index by 'count'
    Surround,         //this would not benefit from using a count. use existing selection primitives to select text to surround
    SurroundingPair,  //TODO: this may benefit from using a count. would select the 'count'th surrounding pair
    FlipDirection,
        //TODO: SplitSelectionLines,    //split current selection into a selection for each line. error if single line
}
impl SelectionAction{
    fn action_name(&self) -> String{
        let name = match self{
            SelectionAction::AddSelectionAbove => "add selection above",
            SelectionAction::AddSelectionBelow => "add selection below",
            SelectionAction::ClearNonPrimarySelections => "clear non primary selections",
            SelectionAction::CollapseSelectionToAnchor => "collapse selection to anchor",
            SelectionAction::CollapseSelectionToCursor => "collapse selection to cursor",
            SelectionAction::DecrementPrimarySelection => "decrement primary selection",
            SelectionAction::ExtendSelectionDown => "extend selection down",
            SelectionAction::ExtendSelectionHome => "extend selection home",
            SelectionAction::ExtendSelectionLeft => "extend selection left",
            SelectionAction::ExtendSelectionLineEnd => "extend selection to line end",
            SelectionAction::ExtendSelectionRight => "extend selection right",
            SelectionAction::ExtendSelectionUp => "extend selection up",
            SelectionAction::ExtendSelectionWordBoundaryBackward => "extend selection word boundary backward",
            SelectionAction::ExtendSelectionWordBoundaryForward => "extend selection word boundary forward",
            SelectionAction::FlipDirection => "flip direction",
            SelectionAction::IncrementPrimarySelection => "increment primary selection",
            SelectionAction::MoveCursorBufferEnd => "move cursor buffer end",
            SelectionAction::MoveCursorBufferStart => "move cursor buffer start",
            SelectionAction::MoveCursorDown => "move cursor down",
            SelectionAction::MoveCursorHome => "move cursor home",
            SelectionAction::MoveCursorLeft => "move cursor left",
            SelectionAction::MoveCursorLineEnd => "move cursor to line end",
            SelectionAction::MoveCursorPageDown => "move cursor page down",
            SelectionAction::MoveCursorPageUp => "move cursor page up",
            SelectionAction::MoveCursorRight => "move cursor right",
            SelectionAction::MoveCursorUp => "move cursor up",
            SelectionAction::MoveCursorWordBoundaryBackward => "move cursor word boundary backward",
            SelectionAction::MoveCursorWordBoundaryForward => "move cursor word boundary forward",
            SelectionAction::RemovePrimarySelection => "remove primary selection",
            SelectionAction::SelectAll => "select all",
            SelectionAction::SelectLine => "select line",
            SelectionAction::Surround => "surround",
            SelectionAction::SurroundingPair => "select nearest surrounding bracket pair",
        };
        name.to_string()
    }
}
#[derive(Clone)] pub enum EditAction{
        //TODO: AlignSelectedTextVertically,
    InsertChar(char),
    InsertNewline,
    InsertTab,
    Delete,
        //TODO: DeleteToNextWordBoundary,
        //TODO: DeleteToPrevWordBoundary,
    Backspace,
    Cut,
    Paste,
    Undo,
    Redo,
        //TODO: SwapUp,   (if text selected, swap selected text with line above. if no selection, swap current line with line above)
        //TODO: SwapDown, (if text selected, swap selected text with line below. if no selection, swap current line with line below)
        //TODO: RotateTextInSelections,
    AddSurround(char, char),
}
#[derive(Clone)] pub enum ViewAction{
    CenterVerticallyAroundCursor,
        //TODO: CenterHorizontallyAroundCursor,
        //TODO: AlignWithCursorAtTop,
        //TODO: AlignWithCursorAtBottom,    
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}
impl ViewAction{
    fn action_name(&self) -> String{
        let name = match self{
            ViewAction::CenterVerticallyAroundCursor => "center vertically around primary cursor",
            ViewAction::ScrollDown => "scroll down",
            ViewAction::ScrollLeft => "scroll left",
            ViewAction::ScrollRight => "scroll right",
            ViewAction::ScrollUp => "scroll up"
        };
        name.to_string()
    }
}
#[derive(Clone)] pub enum UtilAction{
    Backspace,
    Delete,
    InsertChar(char),
    ExtendEnd,
    ExtendHome,
    ExtendLeft,
    ExtendRight,
    MoveEnd,
    MoveHome,
    MoveLeft,
    MoveRight,
    Cut,
    Copy,
    Paste,
    Accept,
    Exit,
    GotoModeSelectionAction(SelectionAction),
}
impl UtilAction{
    fn action_name(&self) -> String{
        let name = match self{
            UtilAction::Accept => "accept",
            UtilAction::Backspace => "util text box backspace",
            UtilAction::Copy => "util text box copy",
            UtilAction::Cut => "util text box cut",
            UtilAction::Delete => "util text box delete",
            UtilAction::Exit => "exit mode",
            UtilAction::ExtendEnd => "util text box extend selection to text end",
            UtilAction::ExtendHome => "util text box extend selection home",
            UtilAction::ExtendLeft => "util text box extend selection left",
            UtilAction::ExtendRight => "util text box extend selection right",
            UtilAction::GotoModeSelectionAction(selection_action) => &selection_action.action_name(),
            UtilAction::InsertChar(c) => &format!("util text box insert char {}", c),
            UtilAction::MoveEnd => "util text box move cursor text end",
            UtilAction::MoveHome => "util text box move cursor home",
            UtilAction::MoveLeft => "util text box move cursor left",
            UtilAction::MoveRight => "util text box move cursor right",
            UtilAction::Paste => "util text box paste"
        };
        name.to_string()
    }
}
#[derive(Clone)] pub enum Action{
    EditorAction(EditorAction),
    SelectionAction(SelectionAction, usize),
    EditAction(EditAction),
    ViewAction(ViewAction),
    UtilAction(UtilAction)
}
impl Action{
    pub fn command_name(&self) -> String{
        let command_name = match self{
            Action::EditorAction(editor_action) => {&editor_action.action_name()}
            Action::SelectionAction(selection_action, _) => {&selection_action.action_name()}
            Action::EditAction(_edit_action) => "unimplemented",
            Action::ViewAction(view_action) => {&view_action.action_name()}
            Action::UtilAction(util_action) => {&util_action.action_name()}
        };
        command_name.to_string()
    }
    pub fn command_source(&self) -> String{
        let command_source = match self{
            Action::EditorAction(_) | Action::ViewAction(_) | Action::UtilAction(_) => "(edit cli)",
            Action::SelectionAction(_, _) | Action::EditAction(_) => "(edit core)",
        };
        command_source.to_string()
    }
}

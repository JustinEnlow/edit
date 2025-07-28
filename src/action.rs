#[derive(Clone)] pub enum EditorAction{
    ModePop,
    ModePush(crate::mode::Mode),
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
#[derive(Clone)] pub enum SelectionAction{   //TODO?: have (all?) selection actions take an amount, for action repetition. MoveCursorDown(2) would move the cursor down two lines, if possible, or saturate at buffer end otherwise, and error if already at buffer end
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
#[derive(Clone)] pub enum Action{
    EditorAction(EditorAction),
    SelectionAction(SelectionAction, usize),
    EditAction(EditAction),
    ViewAction(ViewAction),
    UtilAction(UtilAction)
}

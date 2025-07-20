#[derive(Clone, PartialEq, Debug)]
pub enum Mode{
    /// for editing text and moving/extending selections
    Insert,
    
    /// for display of errors in the use of the editor(such as invalid input)
    /// should block input until mode exited
    /// to be displayed in ERROR_MODE_BACKGROUND_COLOR and ERROR_MODE_FOREGROUND_COLOR
    Error(String),   //maybe same state warnings should be in notify, so they don't block
    
    /// for display of warnings(such as same state)
    /// unhandled keybinds should fall through to Insert mode, clearing util bar
    /// to be displayed in WARNING_MODE_BACKGROUND_COLOR and WARNING_MODE_FOREGROUND_COLOR
    Warning(String), 
    
    /// for display of notifications(such as text copied indicator, or "action performed outside of view" for non-visible actions)
    /// unhandled keybinds should fall through to Insert mode, clearing util bar
    /// to be displayed in NOTIFY_MODE_BACKGROUND_COLOR and NOTIFY_MODE_FOREGROUND_COLOR
    Notify(String),
    
    /// for display of any information(such as resolved command variables)
    /// unhandled keybinds should fall through to Insert mode, clearing util bar
    /// to be displayed in INFO_MODE_BACKGROUND_COLOR and INFO_MODE_FOREGROUND_COLOR
    /// for example, the command: info %{file_name} , should display the file name or None in the util bar
    /// or info date    , should display the current date in the util bar
    Info(String),
    
    /// for adjusting the visible area of text
    View,
    
    /// for jumping to specified line number    //potentially more in the future...
    Goto,
    
    /// for issuing editor commands
    Command,
    
    /// for selecting any matching regex from inside selections
    Find,
    
    /// for retaining everything within selections that isn't a matching regex pattern
    Split,
    
    /// for selecting text objects
    Object,
    
    /// for inserting bracket pairs around selection(s) contents
    AddSurround,    //maybe change to AddSurroundingPair or AddBracketPair

    // NOTE: may not ever implement the following, but good to think about...
    //select the next occurring instance of a search pattern
    //SearchNextAhead,
    //select the prev occurring instance of a search pattern
    //SearchPrevBehind
    //select until the next occuring instance of a search pattern
    //SelectUntilNext,
    //select until the prev occuring instance of a search pattern
    //SelectUntilPrev,
}

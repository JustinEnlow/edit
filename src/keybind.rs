use crate::{
    mode::Mode,
    application::Application,
    action::{Action, EditorAction, SelectionAction, EditAction, ViewAction, UtilAction}
};
use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};


//TODO: deserialize user config keybinds to hashmap{KeyBind, Action}
//then call app.action(action_map.get(KeyBind))
//would action have to be a string for user to define their own?...
//maybe define EditorAction::Custom(user_action_string) that performs a user defined set of commands


//TODO: figure out how to remove app from args
pub fn handle_key_event(app: &mut Application, key_event: KeyEvent) -> Action{
    match app.mode(){
        Mode::Insert => {handle_insert_mode_keypress(key_event.code, key_event.modifiers)}
        Mode::View => {handle_view_mode_keypress(key_event.code, key_event.modifiers)}
        Mode::Goto => {handle_goto_mode_keypress(key_event.code, key_event.modifiers)}
        Mode::Find => {handle_find_mode_keypress(key_event.code, key_event.modifiers)}
        Mode::Command => {handle_command_mode_keypress(key_event.code, key_event.modifiers)}
        Mode::Error(_) => {handle_error_mode_keypress(key_event.code, key_event.modifiers)}
        Mode::Warning(_) => {
            //unhandled keybinds in warning mode fall through to insert mode //TODO: do the same for suggestions mode(not impled yet)
            handle_warning_mode_keypress(app, key_event.code, key_event.modifiers)
        }
        Mode::Notify(_) => {
            //unhandled keybinds in notify mode fall through to insert mode //TODO: do the same for suggestions mode(not impled yet)
            handle_notify_mode_keypress(app, key_event.code, key_event.modifiers)
        }
        Mode::Info(_) => {
            //unhandled keybinds in info mode fall through to insert mode //TODO: do the same for suggestions mode(not impled yet)
            handle_info_mode_keypress(app, key_event.code, key_event.modifiers)
        }
        Mode::Split => {handle_split_mode_keypress(key_event.code, key_event.modifiers)}
        Mode::Object => {handle_object_mode_keypress(key_event.code, key_event.modifiers)}
        Mode::AddSurround => {handle_add_surround_mode_keypress(key_event.code, key_event.modifiers)}
    }
}

pub fn handle_insert_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        KeyCode::Char('p') if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::SelectionAction(SelectionAction::DecrementPrimarySelection, 1),
        KeyCode::Char('z') if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::EditAction(EditAction::Redo),
            //this seems to be triggering some mode in my alacritty settings?...  yeah, doesn't seem to be a thing in GNOME terminal...
            //KeyCode::Char('b') if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.mode_push(Mode::AddSurround),
            //'|' is <shift+\>
            //KeyCode::Char('\\') if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.mode_push(Mode::Pipe),
        KeyCode::Char(' ') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::ModePush(Mode::View)),
        KeyCode::Char('a') if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::SelectAll, 1),
        KeyCode::Char('b') if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::Surround, 1),
        KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::Copy),
        KeyCode::Char('d') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::ModePush(Mode::AddSurround)),
        KeyCode::Char('f') if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::FlipDirection, 1),
        KeyCode::Char('g') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::ModePush(Mode::Goto)),
        KeyCode::Char('l') if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::SelectLine, 1),
        KeyCode::Char('o') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::ModePush(Mode::Object)),
        KeyCode::Char('p') if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::IncrementPrimarySelection, 1),
        KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::Quit),
        KeyCode::Char('r') if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::RemovePrimarySelection, 1),
        KeyCode::Char('s') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::Save),
        KeyCode::Char('t') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::OpenNewTerminalWindow),
        KeyCode::Char('v') if modifiers == KeyModifiers::CONTROL => Action::EditAction(EditAction::Paste),
        KeyCode::Char('x') if modifiers == KeyModifiers::CONTROL => Action::EditAction(EditAction::Cut),
        KeyCode::Char('z') if modifiers == KeyModifiers::CONTROL => Action::EditAction(EditAction::Undo),
        KeyCode::Char('/') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::ModePush(Mode::Find)),
        KeyCode::Char(',') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::ModePush(Mode::Split)),
        KeyCode::Char(';') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::ModePush(Mode::Command)),
        KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => Action::EditAction(EditAction::InsertChar(c)),
        KeyCode::Char('v') if modifiers == KeyModifiers::ALT => Action::ViewAction(ViewAction::CenterVerticallyAroundCursor),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::InsertChar(c)),
            //KeyCode::PageDown if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionPageDown, 1),
        KeyCode::PageDown if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::MoveCursorPageDown, 1),
            //KeyCode::PageUp if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionPageUp, 1),
        KeyCode::PageUp if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::MoveCursorPageUp, 1),
        KeyCode::Up if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::SelectionAction(SelectionAction::AddSelectionAbove, 1),
        KeyCode::Up if modifiers == KeyModifiers::SHIFT => Action::SelectionAction(SelectionAction::ExtendSelectionUp, 1),
        KeyCode::Up if modifiers == KeyModifiers::ALT => Action::ViewAction(ViewAction::ScrollUp),
        KeyCode::Up if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::MoveCursorUp, 1),
        KeyCode::Down if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::SelectionAction(SelectionAction::AddSelectionBelow, 1),
        KeyCode::Down if modifiers == KeyModifiers::SHIFT => Action::SelectionAction(SelectionAction::ExtendSelectionDown, 1),
        KeyCode::Down if modifiers == KeyModifiers::ALT => Action::ViewAction(ViewAction::ScrollDown),
        KeyCode::Down if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::MoveCursorDown, 1),
        KeyCode::Home if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::MoveCursorBufferStart, 1),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => Action::SelectionAction(SelectionAction::ExtendSelectionHome, 1),
        KeyCode::Home if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::MoveCursorHome, 1),
        KeyCode::End if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::MoveCursorBufferEnd, 1),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => Action::SelectionAction(SelectionAction::ExtendSelectionLineEnd, 1),
        KeyCode::End if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::MoveCursorLineEnd, 1),
        KeyCode::Right if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::SelectionAction(SelectionAction::ExtendSelectionWordBoundaryForward, 1),
        KeyCode::Right if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::MoveCursorWordBoundaryForward, 1),
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => Action::SelectionAction(SelectionAction::ExtendSelectionRight, 1),
        KeyCode::Right if modifiers == KeyModifiers::ALT => Action::ViewAction(ViewAction::ScrollRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::MoveCursorRight, 1),
        KeyCode::Left if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::SelectionAction(SelectionAction::ExtendSelectionWordBoundaryBackward, 1),
        KeyCode::Left if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::MoveCursorWordBoundaryBackward, 1),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => Action::SelectionAction(SelectionAction::ExtendSelectionLeft, 1),
        KeyCode::Left if modifiers == KeyModifiers::ALT => Action::ViewAction(ViewAction::ScrollLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::MoveCursorLeft, 1),
        KeyCode::Tab if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::InsertTab),
        KeyCode::Enter if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::InsertNewline),
            //KeyCode::Delete if modifiers == KeyModifiers::CONTROL => app.delete_to_next_word_boundary(),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::Delete),
            //KeyCode::Backspace if modifiers == KeyModifiers::CONTROL => app.delete_to_revious_word_boundary(),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::Backspace),
        KeyCode::Esc if modifiers == KeyModifiers::CONTROL => Action::SelectionAction(SelectionAction::ClearNonPrimarySelections, 1),
        KeyCode::Esc if modifiers == KeyModifiers::SHIFT => Action::SelectionAction(SelectionAction::CollapseSelectionToAnchor, 1),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::CollapseSelectionToCursor, 1),
            //or use this for automatic esc behavior    //how can this be disambiguated as custom behavior vs builtin fn?
            //KeyCode::Esc if modifiers == KeyModifiers::NONE => app.esc_handle(),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

pub fn handle_view_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::EditorAction(EditorAction::ModePop),
        KeyCode::Char('v') if modifiers == KeyModifiers::NONE => Action::ViewAction(ViewAction::CenterVerticallyAroundCursor),
        KeyCode::Up if modifiers == KeyModifiers::NONE => Action::ViewAction(ViewAction::ScrollUp),
        KeyCode::Down if modifiers == KeyModifiers::NONE => Action::ViewAction(ViewAction::ScrollDown),
        KeyCode::Left if modifiers == KeyModifiers::NONE => Action::ViewAction(ViewAction::ScrollLeft),
        KeyCode::Right if modifiers == KeyModifiers::NONE => Action::ViewAction(ViewAction::ScrollRight),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

pub fn handle_goto_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        KeyCode::PageUp if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorPageUp)),
        KeyCode::PageDown if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorPageDown)),
        KeyCode::Up if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionUp)),
        KeyCode::Up if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorUp)),
        KeyCode::Down if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionDown)),
        KeyCode::Down if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorDown)),
            //KeyCode::Right if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendRight),
            //KeyCode::Right if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveRight),
        KeyCode::Right if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionWordBoundaryForward)),
        KeyCode::Right if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorWordBoundaryForward)),
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionRight)),
        KeyCode::Right if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorRight)),
            //KeyCode::Left if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendLeft),
            //KeyCode::Left if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveLeft),
        KeyCode::Left if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionWordBoundaryBackward)),
        KeyCode::Left if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorWordBoundaryBackward)),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionLeft)),
        KeyCode::Left if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorLeft)),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendHome),
        KeyCode::Home if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveHome),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendEnd),
        KeyCode::End if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveEnd),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Exit),
        KeyCode::Enter if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Accept),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Backspace),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Delete),
            //KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::InsertChar(c)),
            //KeyCode::Char(c) if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::InsertChar(c)),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE && c.is_numeric() => Action::UtilAction(UtilAction::InsertChar(c)),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

pub fn handle_find_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveRight),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveLeft),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendHome),
        KeyCode::Home if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveHome),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendEnd),
        KeyCode::End if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveEnd),
        KeyCode::Char('x') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Cut),
        KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Copy),
        KeyCode::Char('v') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Paste),
        KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::InsertChar(c)),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::InsertChar(c)),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Exit),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Backspace),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Delete),
        //TODO: set warning if util text invalid
        KeyCode::Enter if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Accept),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

pub fn handle_split_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveRight),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveLeft),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendHome),
        KeyCode::Home if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveHome),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendEnd),
        KeyCode::End if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveEnd),
        KeyCode::Char('x') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Cut),
        KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Copy),
        KeyCode::Char('v') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Paste),
        KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::InsertChar(c)),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::InsertChar(c)),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Exit),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Backspace),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Delete),
        //TODO: set warning if util text invalid
        KeyCode::Enter if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Accept),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

pub fn handle_command_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveRight),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveLeft),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendHome),
        KeyCode::Home if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveHome),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::ExtendEnd),
        KeyCode::End if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::MoveEnd),
        KeyCode::Char('x') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Cut),
        KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Copy),
        KeyCode::Char('v') if modifiers == KeyModifiers::CONTROL => Action::UtilAction(UtilAction::Paste),
        KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => Action::UtilAction(UtilAction::InsertChar(c)),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::InsertChar(c)),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Exit),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Backspace),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Delete),
        KeyCode::Enter if modifiers == KeyModifiers::NONE => Action::UtilAction(UtilAction::Accept),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

// error mode does not have fallthrough, and so, does not need to mode pop until Insert mode
// if warning/notify/info mode entered from error mode, user can fallthrough to insert, or press esc to go back to error mode, 
// and esc again to go back to the mode before error. so, the mode stack concept is still viable
pub fn handle_error_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    //no op keypresses here should probably display in error mode, regardless of UNHANDLED_KEYPRESS_DISPLAY_MODE config.
    //otherwise they could end up in a fallthrough mode, and repeated keypresses would escape error mode(possibly unintentionally)
    match keycode{
        KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => Action::EditorAction(EditorAction::Quit),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::EditorAction(EditorAction::ModePop),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

//TODO: may need to do app.mode_pop() until app.mode() == Mode::Insert, to ensure we are in a good state
// because this mode falls through to insert mode, and will crash if we pop to any other mode than insert
pub fn handle_warning_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        //handle warning specific key presses, if any
        //only pop once, to return to previous mode
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::EditorAction(EditorAction::ModePop),
        //else, have key presses fall through to insert mode
        _ => {
            while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                app.action(Action::EditorAction(EditorAction::ModePop));
            }
            handle_insert_mode_keypress(keycode, modifiers)
            //alternatively, check success of insert mode action
            //if success, mode pop until app.mode() is insert, and continue as normal
            //if error, push error to warning mode, to stack warning messages
            //this would have to be done on all insert mode actions...

            //handle_key_event could take a mode as arg, instead of app, so that we could create a new action
            //EditorAction::RerunKeypressEventAsInsertMode(event)
            //so we can spoof our mode as insert, to handle fall through when mode specific handle functions are removed
        }
    }
}

//TODO: may need to do app.mode_pop() until app.mode() == Mode::Insert, to ensure we are in a good state
// because this mode falls through to insert mode, and will crash if we pop to any other mode than insert
pub fn handle_notify_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        //handle notify specific key presses, if any
        //only pop once, to return to previous mode
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::EditorAction(EditorAction::ModePop),
        //else, have key presses fall through to insert mode
        _ => {
            while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                app.action(Action::EditorAction(EditorAction::ModePop));
            }
            handle_insert_mode_keypress(keycode, modifiers)
        }
    }
}

//TODO: may need to do app.mode_pop() until app.mode() == Mode::Insert, to ensure we are in a good state
// because this mode falls through to insert mode, and will crash if we pop to any other mode than insert
pub fn handle_info_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
        //handle info specific key presses, if any
        //only pop once, to return to previous mode
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::EditorAction(EditorAction::ModePop),
        //else, have key presses fall through to insert mode
        _ => {
            while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                app.action(Action::EditorAction(EditorAction::ModePop));
            }
            handle_insert_mode_keypress(keycode, modifiers)
        }
    }
}

pub fn handle_object_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
            //KeyCode::Char('w') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::Word),
            //KeyCode::Char('s') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::Sentence),
            //KeyCode::Char('p') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::Paragraph),
        KeyCode::Char('b') if modifiers == KeyModifiers::NONE => Action::SelectionAction(SelectionAction::SurroundingPair, 1),
            //KeyCode::Char('q') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::QuotePair),
            //KeyCode::Char('e') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::ExclusiveSurroundingPair),
            //KeyCode::Char('i') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::InclusiveSurroundingPair),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::EditorAction(EditorAction::ModePop),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

pub fn handle_add_surround_mode_keypress(keycode: KeyCode, modifiers: KeyModifiers) -> Action{
    match keycode{
            //<  //TODO: why is this not working?... says unbound keypress
            //KeyCode::Char(',') if modifiers == KeyModifiers::SHIFT => app.edit_action(&EditAction::AddSurround('<', '>')),
        KeyCode::Char('[') if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::AddSurround('[', ']')),
        KeyCode::Char('{') if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::AddSurround('{', '}')),
        KeyCode::Char('(') if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::AddSurround('(', ')')),
        KeyCode::Char('<') if modifiers == KeyModifiers::NONE => Action::EditAction(EditAction::AddSurround('<', '>')),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => Action::EditorAction(EditorAction::ModePop),
        _ => Action::EditorAction(EditorAction::NoOpKeypress),
    }
}

//pub fn handle_suggestion_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
//    match (keycode, modifiers){
//        //handle suggestion specific key presses, if any
//        (KeyCode::Esc, modifiers) => {
//            if modifiers == KeyModifiers::NONE{app.mode_pop();}
//            //else, have key presses fall through to insert mode
//            else{
//                app.mode_pop();
//                handle_insert_mode_keypress(app, keycode, modifiers);
//            }
//        }
//        (KeyCode::Tab, modifiers) => {
//            if modifiers == KeyModifiers::SHIFT{/* move backwards through suggestions list */}
//            else if modifiers == KeyModifiers::NONE{/* move forwards through suggestions list */}
//            //else, have key presses fall through to insert mode
//            else{
//                app.mode_pop();
//                handle_insert_mode_keypress(app, keycode, modifiers);
//            }
//        }
//        //else, have key presses fall through to insert mode
//        _ => {
//            app.mode_pop();
//            handle_insert_mode_keypress(app, keycode, modifiers);
//        }
//    }
//}

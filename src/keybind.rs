use crate::application::{Application, Mode, UtilAction, ViewAction, EditAction, SelectionAction};
use crossterm::event::{KeyCode, KeyModifiers};



pub fn handle_insert_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        KeyCode::Char('p') if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.selection_action(&SelectionAction::DecrementPrimarySelection, 1),
        KeyCode::Char('z') if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.edit_action(&EditAction::Redo),
            //this seems to be triggering some mode in my alacritty settings?...  yeah, doesn't seem to be a thing in GNOME terminal...
            //KeyCode::Char('b') if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.mode_push(Mode::AddSurround),
            //'|' is <shift+\>
            //KeyCode::Char('\\') if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.mode_push(Mode::Pipe),
        KeyCode::Char(' ') if modifiers == KeyModifiers::CONTROL => app.mode_push(Mode::View),
        KeyCode::Char('a') if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::SelectAll, 1),
        KeyCode::Char('b') if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::Surround, 1),
        KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => app.copy(),
        KeyCode::Char('d') if modifiers == KeyModifiers::CONTROL => app.mode_push(Mode::AddSurround),
        KeyCode::Char('f') if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::FlipDirection, 1),
        KeyCode::Char('g') if modifiers == KeyModifiers::CONTROL => app.mode_push(Mode::Goto),
        KeyCode::Char('l') if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::SelectLine, 1),
        KeyCode::Char('o') if modifiers == KeyModifiers::CONTROL => app.mode_push(Mode::Object),
        KeyCode::Char('p') if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::IncrementPrimarySelection, 1),
        KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => app.quit(),
        KeyCode::Char('r') if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::RemovePrimarySelection, 1),
        KeyCode::Char('s') if modifiers == KeyModifiers::CONTROL => app.save(),
        KeyCode::Char('t') if modifiers == KeyModifiers::CONTROL => Application::open_new_terminal_window(),
        KeyCode::Char('v') if modifiers == KeyModifiers::CONTROL => app.edit_action(&EditAction::Paste),
        KeyCode::Char('x') if modifiers == KeyModifiers::CONTROL => app.edit_action(&EditAction::Cut),
        KeyCode::Char('z') if modifiers == KeyModifiers::CONTROL => app.edit_action(&EditAction::Undo),
        KeyCode::Char('/') if modifiers == KeyModifiers::CONTROL => app.mode_push(Mode::Find),
        KeyCode::Char(',') if modifiers == KeyModifiers::CONTROL => app.mode_push(Mode::Split),
        KeyCode::Char(';') if modifiers == KeyModifiers::CONTROL => app.mode_push(Mode::Command),
        KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => app.edit_action(&EditAction::InsertChar(c)),
        KeyCode::Char('v') if modifiers == KeyModifiers::ALT => app.view_action(&ViewAction::CenterVerticallyAroundCursor),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::InsertChar(c)),
            //KeyCode::PageDown if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionPageDown, 1),
        KeyCode::PageDown if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::MoveCursorPageDown, 1),
            //KeyCode::PageUp if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionPageUp, 1),
        KeyCode::PageUp if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::MoveCursorPageUp, 1),
        KeyCode::Up if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.selection_action(&SelectionAction::AddSelectionAbove, 1),
        KeyCode::Up if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionUp, 1),
        KeyCode::Up if modifiers == KeyModifiers::ALT => app.view_action(&ViewAction::ScrollUp),
        KeyCode::Up if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::MoveCursorUp, 1),
        KeyCode::Down if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.selection_action(&SelectionAction::AddSelectionBelow, 1),
        KeyCode::Down if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionDown, 1),
        KeyCode::Down if modifiers == KeyModifiers::ALT => app.view_action(&ViewAction::ScrollDown),
        KeyCode::Down if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::MoveCursorDown, 1),
        KeyCode::Home if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::MoveCursorBufferStart, 1),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionHome, 1),
        KeyCode::Home if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::MoveCursorHome, 1),
        KeyCode::End if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::MoveCursorBufferEnd, 1),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionLineEnd, 1),
        KeyCode::End if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::MoveCursorLineEnd, 1),
        KeyCode::Right if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.selection_action(&SelectionAction::ExtendSelectionWordBoundaryForward, 1),
        KeyCode::Right if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::MoveCursorWordBoundaryForward, 1),
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionRight, 1),
        KeyCode::Right if modifiers == KeyModifiers::ALT => app.view_action(&ViewAction::ScrollRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::MoveCursorRight, 1),
        KeyCode::Left if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.selection_action(&SelectionAction::ExtendSelectionWordBoundaryBackward, 1),
        KeyCode::Left if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::MoveCursorWordBoundaryBackward, 1),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::ExtendSelectionLeft, 1),
        KeyCode::Left if modifiers == KeyModifiers::ALT => app.view_action(&ViewAction::ScrollLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::MoveCursorLeft, 1),
        KeyCode::Tab if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::InsertTab),
        KeyCode::Enter if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::InsertNewline),
            //KeyCode::Delete if modifiers == KeyModifiers::CONTROL => app.delete_to_next_word_boundary(),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::Delete),
            //KeyCode::Backspace if modifiers == KeyModifiers::CONTROL => app.delete_to_revious_word_boundary(),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::Backspace),
        KeyCode::Esc if modifiers == KeyModifiers::CONTROL => app.selection_action(&SelectionAction::ClearNonPrimarySelections, 1),
        KeyCode::Esc if modifiers == KeyModifiers::SHIFT => app.selection_action(&SelectionAction::CollapseSelectionToAnchor, 1),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::CollapseSelectionToCursor, 1),
            //or use this for automatic esc behavior    //how can this be disambiguated as custom behavior vs builtin fn?
            //KeyCode::Esc if modifiers == KeyModifiers::NONE => app.esc_handle(),
        _ => app.no_op_keypress(),
    }
}

pub fn handle_view_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),
        KeyCode::Char('v') if modifiers == KeyModifiers::NONE => app.view_action(&ViewAction::CenterVerticallyAroundCursor),
        KeyCode::Up if modifiers == KeyModifiers::NONE => app.view_action(&ViewAction::ScrollUp),
        KeyCode::Down if modifiers == KeyModifiers::NONE => app.view_action(&ViewAction::ScrollDown),
        KeyCode::Left if modifiers == KeyModifiers::NONE => app.view_action(&ViewAction::ScrollLeft),
        KeyCode::Right if modifiers == KeyModifiers::NONE => app.view_action(&ViewAction::ScrollRight),
        _ => app.no_op_keypress(),
    }
}

pub fn handle_goto_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        KeyCode::PageUp if modifiers == KeyModifiers::NONE => app.goto_mode_selection_action(&SelectionAction::MoveCursorPageUp),
        KeyCode::PageDown if modifiers == KeyModifiers::NONE => app.goto_mode_selection_action(&SelectionAction::MoveCursorPageDown),
        KeyCode::Up if modifiers == KeyModifiers::SHIFT => app.goto_mode_selection_action(&SelectionAction::ExtendSelectionUp),
        KeyCode::Up if modifiers == KeyModifiers::NONE => app.goto_mode_selection_action(&SelectionAction::MoveCursorUp),
        KeyCode::Down if modifiers == KeyModifiers::SHIFT => app.goto_mode_selection_action(&SelectionAction::ExtendSelectionDown),
        KeyCode::Down if modifiers == KeyModifiers::NONE => app.goto_mode_selection_action(&SelectionAction::MoveCursorDown),
            //KeyCode::Right if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendRight),
            //KeyCode::Right if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveRight),
        KeyCode::Right if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.goto_mode_selection_action(&SelectionAction::ExtendSelectionWordBoundaryForward),
        KeyCode::Right if modifiers == KeyModifiers::CONTROL => app.goto_mode_selection_action(&SelectionAction::MoveCursorWordBoundaryForward),
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => app.goto_mode_selection_action(&SelectionAction::ExtendSelectionRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => app.goto_mode_selection_action(&SelectionAction::MoveCursorRight),
            //KeyCode::Left if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendLeft),
            //KeyCode::Left if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveLeft),
        KeyCode::Left if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => app.goto_mode_selection_action(&SelectionAction::ExtendSelectionWordBoundaryBackward),
        KeyCode::Left if modifiers == KeyModifiers::CONTROL => app.goto_mode_selection_action(&SelectionAction::MoveCursorWordBoundaryBackward),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => app.goto_mode_selection_action(&SelectionAction::ExtendSelectionLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => app.goto_mode_selection_action(&SelectionAction::MoveCursorLeft),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendHome),
        KeyCode::Home if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveHome),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendEnd),
        KeyCode::End if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveEnd),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),
        KeyCode::Enter if modifiers == KeyModifiers::NONE => app.goto_mode_accept(),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::Backspace),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::Delete),
            //KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::InsertChar(c)),
            //KeyCode::Char(c) if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::InsertChar(c)),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE && c.is_numeric() => app.util_action(&UtilAction::InsertChar(c)),
        _ => app.no_op_keypress(),
    }
}

pub fn handle_find_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveRight),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveLeft),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendHome),
        KeyCode::Home if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveHome),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendEnd),
        KeyCode::End if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveEnd),
        KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::InsertChar(c)),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::InsertChar(c)),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.restore_selections_and_exit(),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::Backspace),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::Delete),
        KeyCode::Enter if modifiers == KeyModifiers::NONE => app.mode_pop(),    //TODO: set warning if util text invalid
        _ => app.no_op_keypress(),
    }
}

pub fn handle_split_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveRight),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveLeft),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendHome),
        KeyCode::Home if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveHome),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendEnd),
        KeyCode::End if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveEnd),
        KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::InsertChar(c)),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::InsertChar(c)),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.restore_selections_and_exit(),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::Backspace),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::Delete),
        KeyCode::Enter if modifiers == KeyModifiers::NONE => app.mode_pop(),    //TODO: set warning if util text invalid
        _ => app.no_op_keypress(),
    }
}

pub fn handle_command_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        KeyCode::Right if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendRight),
        KeyCode::Right if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveRight),
        KeyCode::Left if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendLeft),
        KeyCode::Left if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveLeft),
        KeyCode::Home if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendHome),
        KeyCode::Home if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveHome),
        KeyCode::End if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::ExtendEnd),
        KeyCode::End if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::MoveEnd),
        KeyCode::Char(c) if modifiers == KeyModifiers::SHIFT => app.util_action(&UtilAction::InsertChar(c)),
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::InsertChar(c)),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),
        KeyCode::Backspace if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::Backspace),
        KeyCode::Delete if modifiers == KeyModifiers::NONE => app.util_action(&UtilAction::Delete),
        KeyCode::Enter if modifiers == KeyModifiers::NONE => app.command_mode_accept(),
        _ => app.no_op_keypress(),
    }
}

// error mode does not have fallthrough, and so, does not need to mode pop until Insert mode
// if warning/notify/info mode entered from error mode, user can fallthrough to insert, or press esc to go back to error mode, 
// and esc again to go back to the mode before error. so, the mode stack concept is still viable
pub fn handle_error_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    //no op keypresses here need to also display in error mode, regardless of UNHANDLED_KEYPRESS_DISPLAY_MODE config.
    //otherwise they could end up in a fallthrough mode, and repeated keypresses would escape error mode(possibly unintentionally)
    match keycode{
        KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL && app.mode() == Mode::Error(crate::config::FILE_MODIFIED.to_string()) => {
            app.quit_ignoring_changes()
        },
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),
        _ => app.handle_notification(crate::config::DisplayMode::Error, crate::config::UNHANDLED_KEYPRESS),
    }
}

//TODO: may need to do app.mode_pop() until app.mode() == Mode::Insert, to ensure we are in a good state
// because this mode falls through to insert mode, and will crash if we pop to any other mode than insert
pub fn handle_warning_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        //handle warning specific key presses, if any
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),  //only pop once, to return to previous mode
        //else, have key presses fall through to insert mode
        _ => {
            while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                app.mode_pop();
            }
            handle_insert_mode_keypress(app, keycode, modifiers);
        }
    }
}

//TODO: may need to do app.mode_pop() until app.mode() == Mode::Insert, to ensure we are in a good state
// because this mode falls through to insert mode, and will crash if we pop to any other mode than insert
pub fn handle_notify_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        //handle notify specific key presses, if any
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),  //only pop once, to return to previous mode
        //else, have key presses fall through to insert mode
        _ => {
            while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                app.mode_pop();
            }
            handle_insert_mode_keypress(app, keycode, modifiers);
        }
    }
}

//TODO: may need to do app.mode_pop() until app.mode() == Mode::Insert, to ensure we are in a good state
// because this mode falls through to insert mode, and will crash if we pop to any other mode than insert
pub fn handle_info_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
        //handle info specific key presses, if any
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),  //only pop once, to return to previous mode
        //else, have key presses fall through to insert mode
        _ => {
            while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                app.mode_pop();
            }
            handle_insert_mode_keypress(app, keycode, modifiers);
        }
    }
}

pub fn handle_object_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
            //KeyCode::Char('w') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::Word),
            //KeyCode::Char('s') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::Sentence),
            //KeyCode::Char('p') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::Paragraph),
        KeyCode::Char('b') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::SurroundingPair, 1),
            //KeyCode::Char('q') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::QuotePair),
            //KeyCode::Char('e') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::ExclusiveSurroundingPair),
            //KeyCode::Char('i') if modifiers == KeyModifiers::NONE => app.selection_action(&SelectionAction::InclusiveSurroundingPair),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),
        _ => app.no_op_keypress(),
    }
}

pub fn handle_add_surround_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match keycode{
            //<  //TODO: why is this not working?... says unbound keypress
            //KeyCode::Char(',') if modifiers == KeyModifiers::SHIFT => app.edit_action(&EditAction::AddSurround('<', '>')),
        KeyCode::Char('[') if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::AddSurround('[', ']')),
        KeyCode::Char('{') if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::AddSurround('{', '}')),
        KeyCode::Char('(') if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::AddSurround('(', ')')),
        KeyCode::Char('<') if modifiers == KeyModifiers::NONE => app.edit_action(&EditAction::AddSurround('<', '>')),
        KeyCode::Esc if modifiers == KeyModifiers::NONE => app.mode_pop(),
        _ => app.no_op_keypress(),
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

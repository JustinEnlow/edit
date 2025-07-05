use crate::application::{Application, Mode, UtilAction, ViewAction, EditAction, SelectionAction};
use crossterm::event::{KeyCode, KeyModifiers};



pub fn handle_insert_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){
                if c == 'p'{app.selection_action(&SelectionAction::DecrementPrimarySelection, 1);}
                else if c == 'z'{app.edit_action(&EditAction::Redo);}
                    //else if c == 'b'{app.mode_push(Mode::AddSurround);}  //this seems to be triggering some mode in my alacritty settings?...  yeah, doesn't seem to be a thing in GNOME terminal...
                    //else if c == '\\'{app.set_mode(Mode::Pipe);}   //'|' is <shift+\>
                else{app.no_op_keypress();}
            }
            else if modifiers == KeyModifiers::CONTROL{
                if c == ' '{app.mode_push(Mode::View);}
                else if c == 'a'{app.selection_action(&SelectionAction::SelectAll, 1);}
                else if c == 'b'{app.selection_action(&SelectionAction::Surround, 1);}
                else if c == 'c'{app.copy();}
                else if c == 'd'{app.mode_push(Mode::AddSurround);}
                else if c == 'f'{app.selection_action(&SelectionAction::FlipDirection, 1);}
                else if c == 'g'{app.mode_push(Mode::Goto);}
                else if c == 'l'{app.selection_action(&SelectionAction::SelectLine, 1);}
                else if c == 'o'{app.mode_push(Mode::Object);}
                else if c == 'p'{app.selection_action(&SelectionAction::IncrementPrimarySelection, 1);}
                else if c == 'q'{app.quit();}
                else if c == 'r'{app.selection_action(&SelectionAction::RemovePrimarySelection, 1);}
                else if c == 's'{app.save();}
                else if c == 't'{Application::open_new_terminal_window();}
                else if c == 'v'{app.edit_action(&EditAction::Paste);}
                else if c == 'x'{app.edit_action(&EditAction::Cut);}
                else if c == 'z'{app.edit_action(&EditAction::Undo);}
                else if c == '/'{app.mode_push(Mode::Find);}
                else if c == ','{app.mode_push(Mode::Split);}
                else if c == ';'{app.mode_push(Mode::Command);}
                else{app.no_op_keypress();}
            }
            else if modifiers == KeyModifiers::SHIFT{app.edit_action(&EditAction::InsertChar(c));}
            else if modifiers == KeyModifiers::ALT{
                if c == 'v'{app.view_action(&ViewAction::CenterVerticallyAroundCursor);}
                else{app.no_op_keypress();}
            }
            else if modifiers == KeyModifiers::NONE{app.edit_action(&EditAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        (KeyCode::PageDown, modifiers) => {
                //if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::ExtendSelectionPageDown);}
            /*else */if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::MoveCursorPageDown, 1);}
            else{app.no_op_keypress();}
        }
        (KeyCode::PageUp, modifiers) => {
                //if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::ExtendSelectionPageUp);}
            /*else */if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::MoveCursorPageUp, 1);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Up, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.selection_action(&SelectionAction::AddSelectionAbove, 1);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::ExtendSelectionUp, 1);}
            else if modifiers == KeyModifiers::ALT{app.view_action(&ViewAction::ScrollUp);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::MoveCursorUp, 1);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Down, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.selection_action(&SelectionAction::AddSelectionBelow, 1);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::ExtendSelectionDown, 1);}
            else if modifiers == KeyModifiers::ALT{app.view_action(&ViewAction::ScrollDown);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::MoveCursorDown, 1);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::CONTROL{app.selection_action(&SelectionAction::MoveCursorBufferStart, 1);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::ExtendSelectionHome, 1);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::MoveCursorHome, 1);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::CONTROL{app.selection_action(&SelectionAction::MoveCursorBufferEnd, 1);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::ExtendSelectionLineEnd, 1);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::MoveCursorLineEnd, 1);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.selection_action(&SelectionAction::ExtendSelectionWordBoundaryForward, 1);}
            else if modifiers == KeyModifiers::CONTROL{app.selection_action(&SelectionAction::MoveCursorWordBoundaryForward, 1);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::ExtendSelectionRight, 1);}
            else if modifiers == KeyModifiers::ALT{app.view_action(&ViewAction::ScrollRight);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::MoveCursorRight, 1);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.selection_action(&SelectionAction::ExtendSelectionWordBoundaryBackward, 1);}
            else if modifiers == KeyModifiers::CONTROL{app.selection_action(&SelectionAction::MoveCursorWordBoundaryBackward, 1);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::ExtendSelectionLeft, 1);}
            else if modifiers == KeyModifiers::ALT{app.view_action(&ViewAction::ScrollLeft);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::MoveCursorLeft, 1);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Tab, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.edit_action(&EditAction::InsertTab);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.edit_action(&EditAction::InsertNewline);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
                //if modifiers == KeyModifiers::CONTROL{app.delete_to_next_word_boundary();}
            /*else */if modifiers == KeyModifiers::NONE{app.edit_action(&EditAction::Delete);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
                //if modifiers == KeyModifiers::CONTROL{app.delete_to_previous_word_boundary();}
            /*else */if modifiers == KeyModifiers::NONE{app.edit_action(&EditAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::CONTROL{app.selection_action(&SelectionAction::ClearNonPrimarySelections, 1);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(&SelectionAction::CollapseSelectionToAnchor, 1);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(&SelectionAction::CollapseSelectionToCursor, 1);}
                //or use this for automatic esc behavior
                //if modifiers == KeyModifiers::NONE{app.esc_handle();} //how can this be disambiguated as custom behavior vs builtin fn?
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_view_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char('v'), modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(&ViewAction::CenterVerticallyAroundCursor);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Up, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(&ViewAction::ScrollUp);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Down, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(&ViewAction::ScrollDown);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(&ViewAction::ScrollLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(&ViewAction::ScrollRight);}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_goto_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::PageUp, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_selection_action(&SelectionAction::MoveCursorPageUp);}
            else{app.no_op_keypress();}
        }
        (KeyCode::PageDown, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_selection_action(&SelectionAction::MoveCursorPageDown);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Up, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.goto_mode_selection_action(&SelectionAction::ExtendSelectionUp);}
            else if modifiers == KeyModifiers::NONE{app.goto_mode_selection_action(&SelectionAction::MoveCursorUp);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Down, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.goto_mode_selection_action(&SelectionAction::ExtendSelectionDown);}
            else if modifiers == KeyModifiers::NONE{app.goto_mode_selection_action(&SelectionAction::MoveCursorDown);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
                //if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendRight);}
                //else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveRight);}
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.goto_mode_selection_action(&SelectionAction::ExtendSelectionWordBoundaryForward);}
            else if modifiers == KeyModifiers::CONTROL{app.goto_mode_selection_action(&SelectionAction::MoveCursorWordBoundaryForward);}
            else if modifiers == KeyModifiers::SHIFT{app.goto_mode_selection_action(&SelectionAction::ExtendSelectionRight);}
            else if modifiers == KeyModifiers::NONE{app.goto_mode_selection_action(&SelectionAction::MoveCursorRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers)  => {
                //if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendLeft);}
                //else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveLeft);}
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.goto_mode_selection_action(&SelectionAction::ExtendSelectionWordBoundaryBackward);}
            else if modifiers == KeyModifiers::CONTROL{app.goto_mode_selection_action(&SelectionAction::MoveCursorWordBoundaryBackward);}
            else if modifiers == KeyModifiers::SHIFT{app.goto_mode_selection_action(&SelectionAction::ExtendSelectionLeft);}
            else if modifiers == KeyModifiers::NONE{app.goto_mode_selection_action(&SelectionAction::MoveCursorLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers)  => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers)   => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_accept();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
                //if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::InsertChar(c));}
                //else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::InsertChar(c));}
            if modifiers == KeyModifiers::NONE{
                if c.is_numeric(){app.util_action(&UtilAction::InsertChar(c));}
                else{app.no_op_keypress();}
            }
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_find_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
                //if modifiers == KeyModifiers::NONE{app.set_mode(Mode::Insert);}//{app.find_mode_exit();}
            if modifiers == KeyModifiers::NONE{app.restore_selections_and_exit();}
            else{app.no_op_keypress();}
        }
            //(KeyCode::Up, _modifiers) => {
            //    //if modifiers == KeyModifiers::NONE{app.find_replace_mode_previous_instance();}
            //    /*else{*/app.no_op();//}
            //}
            //(KeyCode::Down, _modifiers) => {
            //    //if modifiers == KeyModifiers::NONE{app.find_replace_mode_next_instance();}
            //    /*else{*/app.no_op();//}
            //}
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();}  //TODO: set warning if util text invalid
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_split_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
                //if modifiers == KeyModifiers::NONE{app.set_mode(Mode::Insert);}//{app.split_mode_exit();}
            if modifiers == KeyModifiers::NONE{app.restore_selections_and_exit();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();}  //TODO: set warning if util text invalid
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_command_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(&UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.command_mode_accept();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(&UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

// error mode does not have fallthrough, and so, does not need to mode pop until Insert mode
// if warning/notify/info mode entered from error mode, user can fallthrough to insert, or press esc to go back to error mode, 
// and esc again to go back to the mode before error. so, the mode stack concept is still viable
pub fn handle_error_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    //no op keypresses here need to also display in error mode, regardless of UNHANDLED_KEYPRESS_DISPLAY_MODE config.
    //otherwise they could end up in a fallthrough mode, and repeated keypresses would escape error mode(possibly unintentionally)
    match (keycode, modifiers){
        (KeyCode::Char('q'), modifiers) => {
            if modifiers == KeyModifiers::CONTROL{
                //TODO: should this logic be in its own fn in application.rs?...
                if app.mode() == Mode::Error(crate::config::FILE_MODIFIED.to_string()){
                    app.quit_ignoring_changes();
                }
                else{app.handle_notification(crate::config::DisplayMode::Error, crate::config::UNHANDLED_KEYPRESS);/*app.no_op_keypress();*/}
            }
            else{app.handle_notification(crate::config::DisplayMode::Error, crate::config::UNHANDLED_KEYPRESS);/*app.no_op_keypress();*/}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();}
            else{app.handle_notification(crate::config::DisplayMode::Error, crate::config::UNHANDLED_KEYPRESS);/*app.no_op_keypress();*/}
        }
        _ => {app.handle_notification(crate::config::DisplayMode::Error, crate::config::UNHANDLED_KEYPRESS);/*app.no_op_keypress();*/}
    }
}

//TODO: may need to do app.mode_pop() until app.mode() == Mode::Insert, to ensure we are in a good state
// because this mode falls through to insert mode, and will crash if we pop to any other mode than insert
pub fn handle_warning_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        //handle warning specific key presses, if any
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();} //only pop once, to return to previous mode
            //else, have key presses fall through to insert mode
            else{
                while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                    app.mode_pop();
                }
                handle_insert_mode_keypress(app, keycode, modifiers);
            }
        }
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
    match (keycode, modifiers){
        //handle warning specific key presses, if any
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();} //only pop once, to return to previous mode
            //else, have key presses fall through to insert mode
            else{
                while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                    app.mode_pop();
                }
                handle_insert_mode_keypress(app, keycode, modifiers);
            }
        }
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
    match (keycode, modifiers){
        //handle warning specific key presses, if any
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();} //only pop once, to return to previous mode
            //else, have key presses fall through to insert mode
            else{
                while app.mode() != Mode::Insert{   //pop until insert mode, because of fallthrough
                    app.mode_pop();
                }
                handle_insert_mode_keypress(app, keycode, modifiers);
            }
        }
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
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::NONE{
                if c == 'w'{/*app.selection_action(SelectionAction::Word);*/}
                else if c == 's'{/*app.selection_action(SelectionAction::Sentence*/}
                else if c == 'p'{/*app.selection_action(SelectionAction::Paragraph*/}
                else if c == 'b'{app.selection_action(&SelectionAction::SurroundingPair, 1)}
                    //else if c == 'q'{/*app.selection_action(SelectionAction::QuotePair)*/}
                else if c == 'e'{/*app.selection_action(SelectionAction::ExclusiveSurroundingPair*/}
                else if c == 'i'{/*app.selection_action(SelectionAction::InclusiveSurroundingPair*/}
                else{app.no_op_keypress();}
            }
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{
                app.mode_pop();
            }
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_add_surround_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            //if modifiers == KeyModifiers::SHIFT{
            //    if c == ','{app.edit_action(EditAction::AddSurround('<', '>'));}   //<  //TODO: why is this not working?... says unbound keypress
            //    else{app.no_op_keypress();}
            //}
            /*else */if modifiers == KeyModifiers::NONE{
                if c == '['{app.edit_action(&EditAction::AddSurround('[', ']'));}
                else if c == '{'{app.edit_action(&EditAction::AddSurround('{', '}'));}
                else if c == '('{app.edit_action(&EditAction::AddSurround('(', ')'));}
                else if c == '<'{app.edit_action(&EditAction::AddSurround('<', '>'));}
                else{app.no_op_keypress();}
            }
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{
                app.mode_pop();
            }
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
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

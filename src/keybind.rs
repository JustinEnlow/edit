use crate::application::{Application, Mode, UtilAction, ViewAction, EditAction, SelectionAction};
use crossterm::event::{KeyCode, KeyModifiers};



pub fn handle_insert_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){
                if c == 'p'{app.selection_action(SelectionAction::DecrementPrimarySelection);}
                else if c == 'z'{app.edit_action(EditAction::Redo);}
                //else if c == '\\'{app.set_mode(Mode::Pipe);}   //'|' is <shift+\>
                else{app.no_op_keypress();}
            }
            else if modifiers == KeyModifiers::CONTROL{
                if c == ' '{/*app.set_mode(Mode::View);*/app.mode_push(Mode::View);}
                else if c == 'a'{app.selection_action(SelectionAction::SelectAll);}
                else if c == 'c'{app.copy();}
                else if c == 'g'{/*app.set_mode(Mode::Goto);*/app.mode_push(Mode::Goto);}
                else if c == 'l'{app.selection_action(SelectionAction::SelectLine);}
                else if c == 'p'{app.selection_action(SelectionAction::IncrementPrimarySelection);}
                else if c == 'q'{app.quit();}
                else if c == 'r'{app.selection_action(SelectionAction::RemovePrimarySelection);}
                else if c == 's'{app.save();}
                else if c == 't'{app.open_new_terminal_window();}
                else if c == 'v'{app.edit_action(EditAction::Paste);}
                else if c == 'x'{app.edit_action(EditAction::Cut);}
                else if c == 'z'{app.edit_action(EditAction::Undo);}
                else if c == '/'{/*app.set_mode(Mode::Find);*/app.mode_push(Mode::Find);}
                else if c == ','{/*app.set_mode(Mode::Split);*/app.mode_push(Mode::Split);}
                else if c == ';'{/*app.set_mode(Mode::Command);*/app.mode_push(Mode::Command);}
                else{app.no_op_keypress();}
            }
            else if modifiers == KeyModifiers::SHIFT{app.edit_action(EditAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        (KeyCode::PageDown, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.selection_action(SelectionAction::ExtendSelectionPageDown);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(SelectionAction::MoveCursorPageDown);}
            else{app.no_op_keypress();}
        }
        (KeyCode::PageUp, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.selection_action(SelectionAction::ExtendSelectionPageUp);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(SelectionAction::MoveCursorPageUp);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Up, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.selection_action(SelectionAction::AddSelectionAbove);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(SelectionAction::ExtendSelectionUp);}
            //else if modifiers == KeyModifiers::ALT{app.view_action(ViewAction::ScrollUp);}//{app.scroll_view_up(VIEW_SCROLL_AMOUNT);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(SelectionAction::MoveCursorUp);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Down, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.selection_action(SelectionAction::AddSelectionBelow);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(SelectionAction::ExtendSelectionDown);}
            //else if modifiers == KeyModifiers::ALT{app.view_action(ViewAction::ScrollDown);}//{app.scroll_view_down(VIEW_SCROLL_AMOUNT);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(SelectionAction::MoveCursorDown);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::CONTROL{app.selection_action(SelectionAction::MoveCursorDocumentStart);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(SelectionAction::ExtendSelectionHome);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(SelectionAction::MoveCursorHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::CONTROL{app.selection_action(SelectionAction::MoveCursorDocumentEnd);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(SelectionAction::ExtendSelectionLineEnd);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(SelectionAction::MoveCursorLineEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.selection_action(SelectionAction::ExtendSelectionWordBoundaryForward);}
            else if modifiers == KeyModifiers::CONTROL{app.selection_action(SelectionAction::MoveCursorWordBoundaryForward);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(SelectionAction::ExtendSelectionRight);}
            //else if modifiers == KeyModifiers::ALT{app.view_action(ViewAction::ScrollRight);}//{app.scroll_view_right(VIEW_SCROLL_AMOUNT);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(SelectionAction::MoveCursorRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.selection_action(SelectionAction::ExtendSelectionWordBoundaryBackward);}
            else if modifiers == KeyModifiers::CONTROL{app.selection_action(SelectionAction::MoveCursorWordBoundaryBackward);}
            else if modifiers == KeyModifiers::SHIFT{app.selection_action(SelectionAction::ExtendSelectionLeft);}
            //else if modifiers == KeyModifiers::ALT{app.view_action(ViewAction::ScrollLeft);}//{app.scroll_view_left(VIEW_SCROLL_AMOUNT);}
            else if modifiers == KeyModifiers::NONE{app.selection_action(SelectionAction::MoveCursorLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Tab, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::InsertTab);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::InsertNewline);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            //if modifiers == KeyModifiers::CONTROL{app.delete_to_next_word_boundary();}
            /*else */if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::Delete);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            //if modifiers == KeyModifiers::CONTROL{app.delete_to_previous_word_boundary();}
            /*else */if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.esc_handle();}  //how can this be disambiguated as custom behavior vs builtin fn?
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_view_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{/*app.set_mode(Mode::Insert);*/app.mode_pop();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char('v'), modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(ViewAction::CenterVerticallyAroundCursor);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Up, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(ViewAction::ScrollUp);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Down, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(ViewAction::ScrollDown);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(ViewAction::ScrollLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(ViewAction::ScrollRight);}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_warning_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char('q'), modifiers) => {
            if modifiers == KeyModifiers::CONTROL{
                //TODO: should this logic be in its own fn in application.rs?...
                if app.mode() == Mode::Warning(crate::application::WarningKind::FileIsModified){
                    app.quit_ignoring_changes();
                }
                else{app.no_op_keypress();}
            }
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{/*app.set_mode(Mode::Insert);*/app.mode_pop();}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_goto_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Up, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_selection_action(SelectionAction::MoveCursorUp);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Down, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_selection_action(SelectionAction::MoveCursorDown);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers)  => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers)  => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers)   => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{/*app.set_mode(Mode::Insert);*/app.mode_pop();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_accept();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_find_replace_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::InsertChar(c));}
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
            if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{/*app.set_mode(Mode::Insert);*/app.mode_pop();}  //TODO: set warning if util text invalid
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_split_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            //if modifiers == KeyModifiers::NONE{app.set_mode(Mode::Insert);}//{app.split_mode_exit();}
            if modifiers == KeyModifiers::NONE{app.restore_selections_and_exit();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{/*app.set_mode(Mode::Insert);*/app.mode_pop();}  //TODO: set warning if util text invalid
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_command_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.util_action(UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{/*app.set_mode(Mode::Insert);*/app.mode_pop();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.command_mode_accept();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.util_action(UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_notify_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        //handle notify specific key presses, if any
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.mode_pop();}
            //else, have key presses fall through to insert mode
            else{
                app.mode_pop();
                handle_insert_mode_keypress(app, keycode, modifiers);
            }
        }
        //else, have key presses fall through to insert mode
        _ => {
            app.mode_pop();
            handle_insert_mode_keypress(app, keycode, modifiers);
        }
    }
}

pub fn handle_select_inside_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::NONE{
                if c == '{'{/*select inside pair('{', '}')*/} //forwards          {idk}
                if c == '}'{/*select inside pair('}', '{')*/} //but backwards     }idk{
                if c == 'w'{/*select inside word*/} //maybe <shift+w> for long word?
                if c == 'p'{/*select inside paragraph*/}    //although, this would prevent us from selecting inside instances of 'p'...
                else{/*select inside instances of single char*/}
            }
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

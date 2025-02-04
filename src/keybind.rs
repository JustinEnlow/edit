use crate::application::{Application, Mode, UtilAction, ViewAction, EditAction};
use crossterm::event::{KeyCode, KeyModifiers};



pub fn handle_insert_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){
                if c == 'p'{app.decrement_primary_selection();}
                else if c == 'z'{app.edit_action(EditAction::Redo);}//{app.redo();}
                //if c == 'l'{app.toggle_line_numbers();}
                else{app.no_op_keypress();}
            }
            else if modifiers == KeyModifiers::CONTROL{
                if c == ' '{app.set_mode(Mode::Space);}//{app.set_mode_space();}
                else if c == 'q'{app.quit();}
                else if c == 's'{app.save();}
                else if c == 'g'{app.set_mode(Mode::Goto);}//{app.set_mode_goto();}
                else if c == 'f'{app.set_mode(Mode::Find);}//{app.set_mode_find();}
                else if c == 'y'{app.set_mode(Mode::Split);}//{app.set_mode_split();}
                else if c == 'l'{app.select_line();} //conflicts with display_line_numbers
                //if c == 'k'{app.toggle_status_bar();}
                else if c == 'o'{app.set_mode(Mode::Command);}//{app.set_mode_command();}
                else if c == 't'{app.open_new_terminal_window();}
                else if c == 'a'{app.select_all();}
                else if c == 'x'{app.edit_action(EditAction::Cut);}//{app.cut();}
                else if c == 'c'{app.copy();}
                else if c == 'v'{app.edit_action(EditAction::Paste);}//{app.paste();}
                else if c == 'p'{app.increment_primary_selection();}
                else if c == 'z'{app.edit_action(EditAction::Undo);}//{app.undo();}
                else if c == 'r'{app.remove_primary_selection();}
                else{app.no_op_keypress();}
            }
            else if modifiers == KeyModifiers::SHIFT{app.edit_action(EditAction::InsertChar(c));}//{app.insert_char(c);}
            else if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::InsertChar(c));}//{app.insert_char(c);}
            else{app.no_op_keypress();}
        }
        (KeyCode::PageDown, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.extend_selection_page_down();}
            else if modifiers == KeyModifiers::NONE{app.move_cursor_page_down();}
            else{app.no_op_keypress();}
        }
        (KeyCode::PageUp, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.extend_selection_page_up();}
            else if modifiers == KeyModifiers::NONE{app.move_cursor_page_up();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Up, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.add_selection_above();}
            else if modifiers == KeyModifiers::SHIFT{app.extend_selection_up();}
            else if modifiers == KeyModifiers::ALT{app.view_action(ViewAction::ScrollUp);}//{app.scroll_view_up(VIEW_SCROLL_AMOUNT);}
            else if modifiers == KeyModifiers::NONE{app.move_cursor_up();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Down, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.add_selection_below();}
            else if modifiers == KeyModifiers::SHIFT{app.extend_selection_down();}
            else if modifiers == KeyModifiers::ALT{app.view_action(ViewAction::ScrollDown);}//{app.scroll_view_down(VIEW_SCROLL_AMOUNT);}
            else if modifiers == KeyModifiers::NONE{app.move_cursor_down();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::CONTROL{app.move_cursor_document_start();}
            else if modifiers == KeyModifiers::SHIFT{app.extend_selection_home();}
            else if modifiers == KeyModifiers::NONE{app.move_cursor_line_start();}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::CONTROL{app.move_cursor_document_end();}
            else if modifiers == KeyModifiers::SHIFT{app.extend_selection_end();}
            else if modifiers == KeyModifiers::NONE{app.move_cursor_line_end();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.extend_selection_word_boundary_forward();}
            else if modifiers == KeyModifiers::CONTROL{app.move_cursor_word_boundary_forward();}
            else if modifiers == KeyModifiers::SHIFT{app.extend_selection_right();}
            else if modifiers == KeyModifiers::ALT{app.view_action(ViewAction::ScrollRight);}//{app.scroll_view_right(VIEW_SCROLL_AMOUNT);}
            else if modifiers == KeyModifiers::NONE{app.move_cursor_right();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT){app.extend_selection_word_boundary_backward();}
            else if modifiers == KeyModifiers::CONTROL{app.move_cursor_word_boundary_backward();}
            else if modifiers == KeyModifiers::SHIFT{app.extend_selection_left();}
            else if modifiers == KeyModifiers::ALT{app.view_action(ViewAction::ScrollLeft);}//{app.scroll_view_left(VIEW_SCROLL_AMOUNT);}
            else if modifiers == KeyModifiers::NONE{app.move_cursor_left();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Tab, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::InsertTab);}//{app.insert_tab();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::InsertNewline);}//{app.insert_newline();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            //if modifiers == KeyModifiers::CONTROL{app.delete_to_next_word_boundary();}
            /*else */if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::Delete);}//{app.delete();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            //if modifiers == KeyModifiers::CONTROL{app.delete_to_previous_word_boundary();}
            /*else */if modifiers == KeyModifiers::NONE{app.edit_action(EditAction::Backspace);}//{app.backspace();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.esc_handle();}  //how can this be disambiguated as custom behavior vs builtin fn?
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_space_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.set_mode(Mode::Insert);}//{app.space_mode_exit();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char('c'), modifiers) => {
            if modifiers == KeyModifiers::NONE{app.view_action(ViewAction::CenterVerticallyAroundCursor);}//{app.center_view_vertically_around_cursor();}    //this still needs be made to exit space mode
            else{app.no_op_keypress();}
        }
        (KeyCode::Char('p'), modifiers) => {
            if modifiers == KeyModifiers::NONE{app.increment_primary_selection();}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_warning_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char('q'), modifiers) => {
            if modifiers == KeyModifiers::CONTROL{app.quit_ignoring_changes();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.set_mode(Mode::Insert);}//{app.warning_mode_exit();}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_goto_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Goto, UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Goto, UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers)  => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Goto, UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Goto, UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers)  => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Goto, UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Goto, UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers)   => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Goto, UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Goto, UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_exit();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_accept();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_backspace();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_delete();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::NONE{app.goto_mode_insert_char(c);}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_find_replace_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Find, UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Find, UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Find, UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Find, UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Find, UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Find, UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Find, UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Find, UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.find_mode_insert_char(c);}
            else if modifiers == KeyModifiers::NONE{app.find_mode_insert_char(c);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.find_mode_exit();}
            else{app.no_op_keypress();}
        }
        //(KeyCode::Tab, _modifiers) => {
        //    //if modifiers == KeyModifiers::NONE{app.find_replace_mode_switch_util_bar_focus();}
        //    /*else{*/app.no_op();//}
        //}
        //(KeyCode::Up, _modifiers) => {
        //    //if modifiers == KeyModifiers::NONE{app.find_replace_mode_previous_instance();}
        //    /*else{*/app.no_op();//}
        //}
        //(KeyCode::Down, _modifiers) => {
        //    //if modifiers == KeyModifiers::NONE{app.find_replace_mode_next_instance();}
        //    /*else{*/app.no_op();//}
        //}
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.find_mode_backspace();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.find_mode_delete();}
            else{app.no_op_keypress();}
        }
        //(KeyCode::Enter, modifiers) => {
        //    if modifiers == KeyModifiers::NONE{app.find_mode_accept();}
        //    else{app.no_op();}
        //}
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_split_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Split, UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Split, UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Split, UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Split, UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Split, UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Split, UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Split, UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Split, UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.split_mode_insert_char(c);}
            else if modifiers == KeyModifiers::NONE{app.split_mode_insert_char(c);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.split_mode_exit();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.split_mode_backspace();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.split_mode_delete();}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

pub fn handle_command_mode_keypress(app: &mut Application, keycode: KeyCode, modifiers: KeyModifiers){
    match (keycode, modifiers){
        (KeyCode::Char(c), modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Command, UtilAction::InsertChar(c));}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Command, UtilAction::InsertChar(c));}
            else{app.no_op_keypress();}
        }
        (KeyCode::Right, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Command, UtilAction::ExtendRight);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Command, UtilAction::MoveRight);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Left, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Command, UtilAction::ExtendLeft);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Command, UtilAction::MoveLeft);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Home, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Command, UtilAction::ExtendHome);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Command, UtilAction::MoveHome);}
            else{app.no_op_keypress();}
        }
        (KeyCode::End, modifiers) => {
            if modifiers == KeyModifiers::SHIFT{app.generic_util_action(Mode::Command, UtilAction::ExtendEnd);}
            else if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Command, UtilAction::MoveEnd);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Esc, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.command_mode_exit();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Enter, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.command_mode_accept();}
            else{app.no_op_keypress();}
        }
        (KeyCode::Backspace, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Command, UtilAction::Backspace);}
            else{app.no_op_keypress();}
        }
        (KeyCode::Delete, modifiers) => {
            if modifiers == KeyModifiers::NONE{app.generic_util_action(Mode::Command, UtilAction::Delete);}
            else{app.no_op_keypress();}
        }
        _ => {app.no_op_keypress();}
    }
}

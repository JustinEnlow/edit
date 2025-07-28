use crate::{
    mode::Mode,
    action::{Action, EditorAction, SelectionAction, EditAction, ViewAction, UtilAction}
};
use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
use std::collections::HashMap;



//TODO: deserialize user config keybinds to hashmap{KeyBind, Action}
//then call app.action(action_map.get(KeyBind))
//would action have to be a string for user to define their own?...
//maybe define EditorAction::Custom(user_action_string) that performs a user defined set of commands

pub fn default_keybinds() -> HashMap<(Mode, KeyEvent), Action>{
    let mut keybinds = HashMap::new();
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::DecrementPrimarySelection, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Action::EditAction(EditAction::Redo));
        //this seems to be triggering some mode in my alacritty settings?...  yeah, doesn't seem to be a thing in GNOME terminal...
        //keybinds.insert((Mode::Insert, KeyEvent::new_with_kind(KeyCode::Char('b'), KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyEventKind::Press)), Action::EditorAction(EditorAction::ModePush(Mode::AddSurround)));
        //'|' is <shift+\>
        //keybinds.insert((Mode::Insert, KeyEvent::new_with_kind(KeyCode::Char('\\'), KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyEventKind::Press)), Action::EditorAction(EditorAction::Pipe));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::ModePush(Mode::View)));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::SelectAll, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::Surround, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::Copy));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::ModePush(Mode::AddSurround)));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::FlipDirection, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::ModePush(Mode::Goto)));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::SelectLine, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::ModePush(Mode::Object)));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::IncrementPrimarySelection, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::Quit));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::RemovePrimarySelection, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::Save));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::OpenNewTerminalWindow));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL)), Action::EditAction(EditAction::Paste));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL)), Action::EditAction(EditAction::Cut));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL)), Action::EditAction(EditAction::Undo));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('/'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::ModePush(Mode::Find)));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char(','), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::ModePush(Mode::Split)));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char(';'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::ModePush(Mode::Command)));
        //handled in Application::handle_event()
        //keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)), Action::EditAction(EditAction::InsertChar(c)));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT)), Action::ViewAction(ViewAction::CenterVerticallyAroundCursor));
        //handled in Application::handle_event()
        //keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)), Action::EditAction(EditAction::InsertChar(c)));
            //keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::PageDown, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionPageDown, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::MoveCursorPageDown, 1));
            //keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::PageUp, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionPageUp, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::MoveCursorPageUp, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::AddSelectionAbove, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionUp, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Up, KeyModifiers::ALT)), Action::ViewAction(ViewAction::ScrollUp));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::MoveCursorUp, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::AddSelectionBelow, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionDown, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Down, KeyModifiers::ALT)), Action::ViewAction(ViewAction::ScrollDown));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::MoveCursorDown, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Home, KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::MoveCursorBufferStart, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Home, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionHome, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::MoveCursorHome, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::MoveCursorBufferEnd, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::End, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionLineEnd, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::End, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::MoveCursorLineEnd, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionWordBoundaryForward, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::MoveCursorWordBoundaryForward, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionRight, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Right, KeyModifiers::ALT)), Action::ViewAction(ViewAction::ScrollRight));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::MoveCursorRight, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionWordBoundaryBackward, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::MoveCursorWordBoundaryBackward, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::ExtendSelectionLeft, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Left, KeyModifiers::ALT)), Action::ViewAction(ViewAction::ScrollLeft));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::MoveCursorLeft, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)), Action::EditAction(EditAction::InsertTab));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)), Action::EditAction(EditAction::InsertNewline));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE)), Action::EditAction(EditAction::Delete));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)), Action::EditAction(EditAction::Backspace));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Esc, KeyModifiers::CONTROL)), Action::SelectionAction(SelectionAction::ClearNonPrimarySelections, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT)), Action::SelectionAction(SelectionAction::CollapseSelectionToAnchor, 1));
    keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::CollapseSelectionToCursor, 1));
            //or use this for automatic esc behavior    //how can this be disambiguated as custom behavior vs builtin fn?
            //keybinds.insert((Mode::Insert, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::AutoEsc, 1));   //app.esc_handle()

    keybinds.insert((Mode::View, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::EditorAction(EditorAction::ModePop));
    keybinds.insert((Mode::View, KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE)), Action::ViewAction(ViewAction::CenterVerticallyAroundCursor));
    keybinds.insert((Mode::View, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)), Action::ViewAction(ViewAction::ScrollUp));
    keybinds.insert((Mode::View, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)), Action::ViewAction(ViewAction::ScrollDown));
    keybinds.insert((Mode::View, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), Action::ViewAction(ViewAction::ScrollLeft));
    keybinds.insert((Mode::View, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)), Action::ViewAction(ViewAction::ScrollRight));

    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorPageUp)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorPageDown)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionUp)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorUp)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionDown)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorDown)));
            //keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendRight));
            //keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveRight));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionWordBoundaryForward)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorWordBoundaryForward)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionRight)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorRight)));
            //keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendLeft));
            //keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveLeft));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionWordBoundaryBackward)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorWordBoundaryBackward)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::ExtendSelectionLeft)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), Action::UtilAction(UtilAction::GotoModeSelectionAction(SelectionAction::MoveCursorLeft)));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Home, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendHome));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveHome));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::End, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendEnd));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::End, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveEnd));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Exit));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Accept));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Backspace));
    keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Delete));
        //handled in Application::handle_event()
        //keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::InsertChar(c)));
        //keybinds.insert((Mode::Goto, KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)), Action::UtilAction(UtilAction::InsertChar(c)));

    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendRight));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveRight));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendLeft));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveLeft));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Home, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendHome));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveHome));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::End, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendEnd));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::End, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveEnd));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Cut));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Copy));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Paste));
        //handled in Application::handle_event()
        //keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::InsertChar(c)));
        //keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)), Action::UtilAction(UtilAction::InsertChar(c)));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Exit));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Backspace));
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Delete));
    //TODO: set warning if util text invalid
    keybinds.insert((Mode::Find, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Accept));

    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendRight));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveRight));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendLeft));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveLeft));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Home, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendHome));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveHome));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::End, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendEnd));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::End, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveEnd));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Cut));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Copy));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Paste));
        //handled in Application::handle_event()
        //keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::InsertChar(c)));
        //keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)), Action::UtilAction(UtilAction::InsertChar(c)));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Exit));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Backspace));
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Delete));
    //TODO: set warning if util text invalid
    keybinds.insert((Mode::Split, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Accept));

    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendRight));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveRight));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendLeft));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveLeft));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Home, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendHome));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveHome));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::End, KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::ExtendEnd));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::End, KeyModifiers::NONE)), Action::UtilAction(UtilAction::MoveEnd));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Cut));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Copy));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL)), Action::UtilAction(UtilAction::Paste));
        //handled in Application::handle_event()
        //keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)), Action::UtilAction(UtilAction::InsertChar(c)));
        //keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)), Action::UtilAction(UtilAction::InsertChar(c)));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Exit));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Backspace));
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Delete));
    //TODO: set warning if util text invalid
    keybinds.insert((Mode::Command, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)), Action::UtilAction(UtilAction::Accept));

    keybinds.insert((Mode::Error, KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)), Action::EditorAction(EditorAction::Quit));
    keybinds.insert((Mode::Error, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::EditorAction(EditorAction::ModePop));

    keybinds.insert((Mode::Warning, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::EditorAction(EditorAction::ModePop));

    keybinds.insert((Mode::Notify, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::EditorAction(EditorAction::ModePop));

    keybinds.insert((Mode::Info, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::EditorAction(EditorAction::ModePop));

    keybinds.insert((Mode::Object, KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)), Action::SelectionAction(SelectionAction::SurroundingPair, 1));
    keybinds.insert((Mode::Object, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::EditorAction(EditorAction::ModePop));

    keybinds.insert((Mode::AddSurround, KeyEvent::new(KeyCode::Char('['), KeyModifiers::NONE)), Action::EditAction(EditAction::AddSurround('[', ']')));
    keybinds.insert((Mode::AddSurround, KeyEvent::new(KeyCode::Char('{'), KeyModifiers::NONE)), Action::EditAction(EditAction::AddSurround('{', '}')));
    keybinds.insert((Mode::AddSurround, KeyEvent::new(KeyCode::Char('('), KeyModifiers::NONE)), Action::EditAction(EditAction::AddSurround('(', ')')));
    keybinds.insert((Mode::AddSurround, KeyEvent::new(KeyCode::Char('<'), KeyModifiers::NONE)), Action::EditAction(EditAction::AddSurround('<', '>')));
    keybinds.insert((Mode::AddSurround, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::EditorAction(EditorAction::ModePop));

    //for when suggestion mode impled
    //keybinds.insert((Mode::Suggestion, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), Action::EditorAction(EditorAction::ModePop));
    //keybinds.insert((Mode::Suggestion, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)), Action::EditAction(EditAction::AcceptSuggestion));
    //keybinds.insert((Mode::Suggestion, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)), Action::EditorAction(EditorAction::NextSuggestion));
    //keybinds.insert((Mode::Suggestion, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)), Action::EditorAction(EditorAction::PreviousSuggestion));
    //

    keybinds
}

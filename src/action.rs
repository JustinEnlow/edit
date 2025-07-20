pub enum EditorAction{
    ModePop,
    ModePush(crate::mode::Mode),
    Resize(u16, u16),
    NoOpKeypress,
    NoOpEvent,
    Quit,
    QuitIgnoringChanges,
    Save,
    Copy,
    //ToggleLineNumbers,
    //ToggleStatusBar,
    //OpenNewTerminalWindow,
}
pub enum Action{
    EditorAction(EditorAction),
    SelectionAction(crate::application::SelectionAction, usize),
    EditAction(crate::application::EditAction),
    ViewAction(crate::application::ViewAction),
    UtilAction(crate::application::UtilAction)
}

fn perform_action(app: &mut crate::application::Application, action: Action){
    match action{
        Action::EditorAction(editor_action) => {
            match editor_action{
                EditorAction::ModePop => {app.mode_pop();}
                EditorAction::ModePush(to_mode) => {app.mode_push(to_mode);}
                EditorAction::Resize(width, height) => {app.resize(width, height);}
                EditorAction::NoOpKeypress => {}
                EditorAction::NoOpEvent => {}
                EditorAction::Quit => {}
                EditorAction::QuitIgnoringChanges => {}
                EditorAction::Save => {}
                EditorAction::Copy => {}
            }
        }
        Action::SelectionAction(selection_action, count) => {
            app.selection_action(&selection_action, count);
        }
        Action::EditAction(edit_action) => {
            app.edit_action(&edit_action);
        }
        Action::ViewAction(view_action) => {
            app.view_action(&view_action);
        }
        Action::UtilAction(util_action) => {
            app.util_action(&util_action);
        }
    }
}

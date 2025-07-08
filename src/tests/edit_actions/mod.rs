use crate::{
    config::Config,
    application::{EditAction, Mode},
    buffer::Buffer,
    selection::Selection,
    display_area::DisplayArea,
};
use crate::tests::common::{
    set_up_test_application,
    generate_selections
};



mod insert_char;
mod insert_newline;
mod insert_tab;
mod delete;
mod backspace;
mod cut;
mod paste;
mod undo;   //TODO: impl tests
mod redo;   //TODO: impl tests
mod add_surround;



pub fn test_edit_action(
    config: Config,
    edit_action: EditAction,
    //semantics: CursorSemantics,
    render_line_numbers: bool,
    render_status_bar: bool,
    read_only: bool,
    terminal_display_area: DisplayArea,
    buffer_text: &str,
    //tuple_selections: Vec<(usize, usize, Option<usize>)>,
    selections: Vec<Selection>,
    primary: usize,
    clipboard: &str,
    //expected_buffer_display_area: DisplayArea,
    //expected_buffer_display_area_text: &str,
    expected_buffer_text: &str,
    expected_mode: Mode,
    //tuple_expected_selections: Vec<(usize, usize, Option<usize>)>,
    expected_selections: Vec<Selection>,
    expected_primary: usize,
    expected_clipboard: &str
){
    match set_up_test_application(config.clone(), terminal_display_area, buffer_text, read_only, render_line_numbers, render_status_bar){
        Ok(mut app) => {
            let selections = generate_selections(selections, primary, &app.buffer, config.semantics.clone());
            let expected_buffer = Buffer::new(expected_buffer_text, None, read_only);
            let expected_selections = generate_selections(expected_selections, expected_primary, &expected_buffer, config.semantics.clone());

            app.clipboard = clipboard.to_string();
            app.selections = selections;
            //call action specific test(selection/view/edit/etc)
            app.edit_action(&edit_action);

            assert_eq!(expected_mode, app.mode());
            assert_eq!(expected_selections, app.selections);
            assert_eq!(expected_primary, app.selections.primary_selection_index);
            assert_eq!(expected_buffer, app.buffer);
            assert_eq!(expected_clipboard, app.clipboard);

            //assert!(app.buffer.is_modified());    //doesn't work in tests because we can't compare to persistent file
        }
        Err(e) => assert!(false, "{}", e)
    }
}

use crate::{
    config::Config,
    action::EditAction,
    mode::Mode,
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
    _render_line_numbers: bool,
    _render_status_bar: bool,
    read_only: bool,
    terminal_display_area: DisplayArea,
    buffer_text: &str,
    selections: Vec<Selection>,
    primary: usize,
    clipboard: &str,
    expected_mode: Mode,
    expected_buffer_text: &str,
    expected_selections: Vec<Selection>,
    expected_primary: usize,
    expected_clipboard: &str,
    //debug: bool,      //or could pass a bitmap indicating which components to debug: 
                                        //0001  buffer text (real, expected)
                                        //0010  selections (real, expected)
                                        //0100  others...
                                        //1000
){
    match set_up_test_application(
        config.clone(), 
        terminal_display_area.clone(), 
        buffer_text, 
        read_only
        /*, render_line_numbers, render_status_bar*/
    ){
        Ok(mut app) => {
            println!(
"inputs: 
    cursor semantics: {:?}
    edit action: {:?}
    read only: {:?}
    display area: {:?}
    buffer text: {:?}
    selections: {:?}
    primary selection: {:?}
    clipboard: {:?}
    expected mode: {:?}
    expected buffer text: {:?}
    expected selections: {:?}
    expected primary selection: {:?}
    expected clipboard: {:?}
",
                config.semantics,
                edit_action,
                //_render_line_numbers,
                //_render_status_bar,
                read_only,
                terminal_display_area,
                buffer_text,
                selections,
                primary,
                clipboard,
                expected_mode,
                expected_buffer_text,
                expected_selections,
                expected_primary,
                expected_clipboard,
            );

            let selections = generate_selections(selections, primary, &app.buffer, config.semantics.clone());
            let expected_buffer = Buffer::new(expected_buffer_text, None, read_only);
            let expected_selections = generate_selections(expected_selections, expected_primary, &expected_buffer, config.semantics.clone());

            app.clipboard = clipboard.to_string();
            app.selections = selections;
                print!("before edit: ");
                crate::tests::common::debug_buffer_selections(&app.buffer, &app.selections, config.semantics);
            //call action specific test(selection/view/edit/etc)
            //app.edit_action(&edit_action);
            app.update(crate::action::Action::EditAction(edit_action.clone()));
                print!("after edit:  ");
                crate::tests::common::debug_buffer_selections(&app.buffer, &app.selections, config.semantics);
                print!("expected:    ");
                crate::tests::common::debug_buffer_selections(&expected_buffer, &expected_selections, config.semantics);
                println!("");

            //if debug{
                println!("buffer text:");
                crate::tests::common::string_stats(buffer_text);
                println!("");
                println!("expected buffer text:");
                crate::tests::common::string_stats(expected_buffer_text);
                println!("");
            //}

            assert_eq!(expected_mode, app.mode());
            assert_eq!(expected_selections, app.selections);
            assert_eq!(expected_primary, app.selections.primary_selection_index());
            assert_eq!(expected_buffer, app.buffer);
            assert_eq!(expected_clipboard, app.clipboard);

            //assert!(app.buffer.is_modified());    //doesn't work in tests because we can't compare to persistent file
        }
        Err(e) => assert!(false, "{}", e)
    }
}

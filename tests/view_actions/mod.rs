use edit::{
    config::Config,
    application::{ViewAction, Mode},
    //selection::CursorSemantics,
    display_area::DisplayArea,
};
use crate::common::{
    set_up_test_application,
    generate_selections
};



mod center_view_vertically_around_cursor;
mod scroll_up;
mod scroll_down;
mod scroll_left;
mod scroll_right;


//TODO: could take expected display area selections
pub fn test_view_action(
    config: Config,
    view_action: ViewAction,
    //semantics: CursorSemantics,
    render_line_numbers: bool,
    render_status_bar: bool,
    //This may differ from buffer_display_area if line numbers or status bar are shown
    terminal_display_area: DisplayArea,
    starting_mode: Mode,
    buffer_text: &str,
    tuple_selections: Vec<(usize, usize, Option<usize>)>,
    primary: usize,
    expected_mode: Mode,
    expected_buffer_display_area_text: &str,
    expected_buffer_display_area: DisplayArea,
){
    //set up app
    match set_up_test_application(config.clone(), terminal_display_area.clone(), buffer_text, false, render_line_numbers, render_status_bar){
        Ok(mut app) => {
            let selections = generate_selections(tuple_selections, primary, &app.buffer, config.semantics.clone());
            
            app.selections = selections;
            //app.mode_push(Mode::View);
            if starting_mode != Mode::Insert{
                app.mode_push(starting_mode);
            }

            //call action specific test(selection/view/edit/etc)
            app.view_action(&view_action);
            
            assert_eq!(expected_mode, app.mode());
            assert_eq!(expected_buffer_display_area_text, app.buffer_display_area().text(&app.buffer));
            assert_eq!(expected_buffer_display_area, app.buffer_display_area());
            //TODO?: could test if we get the correct selection2ds in display area
            assert!(!app.buffer.is_modified());
        }
        Err(e) => assert!(false, "{}", e)
    }
}

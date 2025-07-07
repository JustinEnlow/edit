use edit::{
    config::Config,
    application::{SelectionAction, Mode},
    selection::Selection,
    display_area::DisplayArea,
};
use crate::common::{
    set_up_test_application,
    generate_selections
};



mod move_cursor_up;
mod move_cursor_down;
mod move_cursor_left;
mod move_cursor_right;
mod move_cursor_word_boundary_forward;
mod move_cursor_word_boundary_backward;
mod move_cursor_line_end;
mod move_cursor_home;
mod move_cursor_buffer_start;
mod move_cursor_buffer_end;
mod move_cursor_page_up;    //TODO: impl tests
mod move_cursor_page_down;  //TODO: impl tests
mod extend_selection_up;
mod extend_selection_down;
mod extend_selection_left;
mod extend_selection_right;
mod extend_selection_word_boundary_backward;
mod extend_selection_word_boundary_forward;
mod extend_selection_line_end;
mod extend_selection_home;
mod select_line;
mod select_all;
mod collapse_selection_to_anchor;
mod collapse_selection_to_cursor;
mod clear_non_primary_selections;
mod add_selection_above;
mod add_selection_below;
mod remove_primary_selection;
mod increment_primary_selection;
mod decrement_primary_selection;
mod surround;
mod flip_direction;
mod surrounding_pair;



pub fn test_selection_action(
    config: Config,
    selection_action: SelectionAction,
    //semantics: CursorSemantics,
    render_line_numbers: bool,
    render_status_bar: bool,
    terminal_display_area: DisplayArea,
    buffer_text: &str,
    //TODO: starting_expected_buffer_display_area: DisplayArea,
    //TODO?: starting_expected_buffer_display_area_text: &str,
    //tuple_selections: Vec<(usize, usize, Option<usize>)>,
    selections: Vec<Selection>,
    primary: usize,
    count: usize, 
    expected_mode: Mode,
    //tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, 
    expected_selections: Vec<Selection>,
    expected_primary: usize,
    //TODO: ending_expected_buffer_display_area: DisplayArea,
    //TODO: expected_buffer_display_area_text: &str,
){
    //set up app
    match set_up_test_application(config.clone(), terminal_display_area, buffer_text, false, render_line_numbers, render_status_bar/* TODO: , starting_expected_buffer_display_area */){
        Ok(mut app) => {
            let expected_selections = generate_selections(expected_selections, expected_primary, &app.buffer, config.semantics.clone()); 
            let selections = generate_selections(selections, primary, &app.buffer, config.semantics.clone());
            
            app.selections = selections;
            //call action specific test(selection/view/edit/etc)
            app.selection_action(&selection_action, count);
            
            assert_eq!(expected_mode, app.mode());
            assert_eq!(expected_selections, app.selections);
            //TODO: assert_eq!(expected_buffer_display_area_text, app.display_area().text(&app.buffer));
            //TODO: assert_eq!(expected_buffer_display_area, app.display_area());
            assert!(!app.buffer.is_modified());
        }
        Err(e) => assert!(false, "{}", e)
    }
}

use crate::{
    application::{Application, ApplicationError, Mode, SelectionAction, ViewAction},
    selections::Selections,
    selection::{Selection, CursorSemantics},
    view::DisplayArea,
    buffer::Buffer,
    config::SAME_STATE,
};



//TODO?: could call app.handle_whatever_mode_keypress instead of calling f. this would allow us to test our reaction code as well...
//TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
pub fn selection_movement_with_count<F>(
    f: F, 
    semantics: CursorSemantics, 
    text: &str, 
    tuple_selections: Vec<(usize, usize, Option<usize>)>, 
    primary: usize, 
    count: usize, 
    display_area: Option<&DisplayArea>, 
    tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, 
    expected_primary: usize
)
    where F: Fn(&mut Application, usize, Option<&DisplayArea>, CursorSemantics) -> Result<(), ApplicationError>
{
    let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
    let expected_selections = generate_selections(tuple_expected_selections, expected_primary, &app.buffer, semantics.clone());
    let selections = generate_selections(tuple_selections, primary, &app.buffer, semantics.clone());
    app.selections = selections;

    //TODO: this only tests our application_impl, and not any response code to it...
    let result = f(&mut app, count, display_area, semantics.clone());
    assert!(!result.is_err());
    
    assert_eq!(expected_selections, app.selections);
    assert!(!app.buffer.is_modified());
}

//could this be used for tests that should panic as well?...
pub fn error_selection_movement_with_count<F>(
    f: F, 
    semantics: CursorSemantics, 
    text: &str, 
    tuple_selections: Vec<(usize, usize, Option<usize>)>, 
    primary: usize, 
    count: usize, 
    display_area: Option<&DisplayArea>
)
    where F: Fn(&mut Application, usize, Option<&DisplayArea>, CursorSemantics) -> Result<(), ApplicationError>
{
    let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
    let selections = generate_selections(tuple_selections, primary, &app.buffer, semantics.clone());
    app.selections = selections;
    
    assert!(f(&mut app, count, display_area, semantics).is_err());
    assert!(!app.buffer.is_modified());
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn set_up_test_application(
    terminal_display_area: DisplayArea, //this represents our full terminal, not just the buffer viewport.
    buffer_text: &str, 
    read_only: bool,
    //TODO: show_line_numbers: bool,
    //TODO: show_status_bar: bool,
    //TODO: expected_buffer_display_area: DisplayArea,
) -> Result<Application, String>{
    /* could we derive terminal_display_area from given buffer_display_area?...
    let terminal_width = if show_line_numbers{
        //should this include padding too?...
        buffer_display_area.width + crate::ui::document_viewport::count_digits(buffer_text.len_lines());   //width of line numbers
    }else{buffer_display_area.width}
    let terminal_height = if show_status_bar{
        buffer_display_area.height + 2
    }else{buffer_display_area.height}
    */
    let backend = ratatui::backend::TestBackend::new(
        terminal_display_area.width as u16, 
        terminal_display_area.height as u16
    );
    match ratatui::Terminal::new(backend){
        Ok(terminal) => {
            match Application::new(buffer_text, None, read_only,&terminal){
                Ok(mut app) => {
                    app.buffer_horizontal_start = terminal_display_area.horizontal_start;
                    app.buffer_vertical_start = terminal_display_area.vertical_start;

                    //TODO: assert_eq!(expected_buffer_display_area, app.buffer_display_area());
                    Ok(app)
                }
                Err(_) => Err("could not create test application instance".to_string()),
            }
        }
        Err(_) => Err("could not create test terminal instance".to_string()),
    }
}

fn generate_selections(tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, buffer: &Buffer, semantics: CursorSemantics) -> Selections{
    let mut selections = Vec::new();
    for tuple in tuple_selections{
        selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, buffer, semantics.clone()));
    }
    Selections::new(selections, primary, buffer, semantics.clone())
}

//TODO: create
    //test_edit_action
    //test_view_action
    //test_mode_specific_actions
    //etc...
fn test_selection_action(
    selection_action: SelectionAction,
    semantics: CursorSemantics,
    terminal_display_area: DisplayArea,
    buffer_text: &str,
    //TODO: starting_expected_buffer_display_area: DisplayArea,
    //TODO?: starting_expected_buffer_display_area_text: &str,
    tuple_selections: Vec<(usize, usize, Option<usize>)>,
    primary: usize,
    count: usize, 
    expected_mode: Mode,
    tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, 
    expected_primary: usize,
    //TODO: ending_expected_buffer_display_area: DisplayArea,
    //TODO: expected_buffer_display_area_text: &str,
){
    //set up app
    match set_up_test_application(terminal_display_area, buffer_text, false/* TODO: , starting_expected_buffer_display_area */){
        Ok(mut app) => {
            let expected_selections = generate_selections(tuple_expected_selections, expected_primary, &app.buffer, semantics.clone()); 
            let selections = generate_selections(tuple_selections, primary, &app.buffer, semantics.clone());
            
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
#[test] fn example_selection_action_test(){
    test_selection_action(
        SelectionAction::MoveCursorRight, 
        CursorSemantics::Block, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50},
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None)
        ], 
        0, 
        1, 
        Mode::Insert,
        vec![
            (1, 2, Some(1))
        ], 
        0
    );
}

#[test] fn example_error_selection_action_test(){
    test_selection_action(
        SelectionAction::MoveCursorRight, 
        CursorSemantics::Block, 
        DisplayArea{horizontal_start: 0, vertical_start: 0, width: 80, height: 50},
        "idk\nsome\nshit\n", 
        vec![
            (14, 15, None)
        ], 
        0, 
        1, 
        Mode::Warning(SAME_STATE.to_string()),
        vec![
            (14, 15, None)
        ], 
        0
    );
}


pub fn test_view_action(
    view_action: ViewAction,
    semantics: CursorSemantics,
    //This may differ from buffer_display_area if line numbers or status bar are shown
    terminal_display_area: DisplayArea,
    buffer_text: &str,
    tuple_selections: Vec<(usize, usize, Option<usize>)>,
    primary: usize,
    expected_mode: Mode,
    expected_buffer_display_area_text: &str,
    expected_buffer_display_area: DisplayArea,
){
    //set up app
    match set_up_test_application(terminal_display_area.clone(), buffer_text, false){
        Ok(mut app) => {
            let selections = generate_selections(tuple_selections, primary, &app.buffer, semantics.clone());
            
            app.selections = selections;
            app.mode_push(Mode::View);

            //call action specific test(selection/view/edit/etc)
            app.view_action(&view_action);
            
            assert_eq!(expected_mode, app.mode());
            assert_eq!(expected_buffer_display_area_text, app.buffer_display_area().text(&app.buffer));
            assert_eq!(expected_buffer_display_area, app.buffer_display_area());
            assert!(!app.buffer.is_modified());
        }
        Err(e) => assert!(false, "{}", e)
    }
}

#[test] fn example_view_action_test(){
    test_view_action(
        ViewAction::ScrollRight, 
        CursorSemantics::Block, 
        DisplayArea{
            horizontal_start: 0, 
            vertical_start: 0, 
            width: 3,   //buffer_display_area.width + len lines + padding?(for line numbers)   //why does set up test application not seem to layout line numbers rn?
            height: 5   //buffer_display_area.height + 2(for status bar)
        },
        "idk\nsome\nshit\n", 
        vec![
            (0, 1, None)
        ], 
        0, 
        Mode::View, 
        "dk\nome\nhit\n", 
        DisplayArea::new(1, 0, 3, 3)
    );
}

use edit::{
    application::Application,
    buffer::Buffer,
    selection::{Selection, CursorSemantics},
    selections::Selections,
    display_area::DisplayArea
};

pub fn set_up_test_application(
    terminal_display_area: DisplayArea, //this represents our full terminal, not just the buffer viewport.
    buffer_text: &str, 
    read_only: bool,
    render_line_numbers: bool,
    render_status_bar: bool,
    //TODO: expected_buffer_display_area: DisplayArea,
) -> Result<Application, String>{
        // i don't think we want to do this. there are some advantages to passing in a manual terminal_display_area, and checking against an expected buffer_display_area
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

                    //this could disable extra widgets, and make terminal_display_area and buffer_display_area equivalent
                    app.ui.document_viewport.line_number_widget.show = render_line_numbers;
                    app.ui.status_bar.show = render_status_bar;
                    app.update_layouts();
                    //TODO: figure out how to print terminal buffer for debugging...
                    //TODO: assert_eq!(expected_buffer_display_area, app.buffer_display_area());
                    Ok(app)
                }
                Err(_) => Err("could not create test application instance".to_string()),
            }
        }
        Err(_) => Err("could not create test terminal instance".to_string()),
    }
}

pub fn generate_selections(tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, buffer: &Buffer, semantics: CursorSemantics) -> Selections{
    let mut selections = Vec::new();
    for tuple in tuple_selections{
        selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, buffer, semantics.clone()));
    }
    Selections::new(selections, primary, buffer, semantics.clone())
}

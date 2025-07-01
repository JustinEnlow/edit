use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, ExtensionDirection/*, Movement */},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
    }
    Ok(())
}

/// Returns a new instance of [`Selection`] with cursor moved down.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let mut selection = selection.clone();
    selection.assert_invariants(buffer, semantics.clone());
    
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_width = buffer.line_width(line_number, false);
    let line_start = buffer.line_to_char(line_number);
    let line_end = line_start.saturating_add(line_width);   //nth_next_grapheme_index(line_start, line_width, text)?

    if selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
    //selection.put_cursor(line_end, text, Movement::Move, semantics, true)
    
    selection.range.start = line_end;
    selection.range.end = match semantics.clone(){
        CursorSemantics::Bar => line_end.min(buffer.len_chars()),
        CursorSemantics::Block => buffer.next_grapheme_boundary_index(line_end).min(buffer.len_chars().saturating_add(1))
    };
    selection.direction = ExtensionDirection::None;
    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
    
    selection.assert_invariants(buffer, semantics.clone());

    Ok(selection)
}

#[cfg(test)]
mod tests{
    use crate::utilities::move_cursor_line_end;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        display_area::DisplayArea,
    };

    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));

        let mut vec_expected_selections = Vec::new();
        for tuple in tuple_expected_selections{
            vec_expected_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let expected_selections = Selections::new(vec_expected_selections, expected_primary, &app.buffer, semantics.clone());
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        let result = move_cursor_line_end::application_impl(&mut app, semantics.clone());
        assert!(!result.is_err());
        
        assert_eq!(expected_selections, app.selections);
        assert!(!app.buffer.is_modified());
    }
    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        assert!(move_cursor_line_end::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_mixed_valid_and_invalid_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),   //common use
                (6, 6, None),   //from middle of line
                (14, 14, None)  //invalid. already at line end
            ], 0, 
            vec![
                (3, 3, Some(3)),
                (8, 8, Some(4)),
                (14, 14, None)
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),   //common use
                (6, 7, None),   //from middle of line
                (14, 15, None)  //invalid. already at line end
            ], 0, 
            vec![
                (3, 4, Some(3)),
                (8, 9, Some(4)),
                (14, 15, None)
            ], 0
        );
    }
    
    #[test] fn errors_when_single_selection_at_line_end_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 3, None)
            ], 0
        );
    }
    #[test] fn errors_when_single_selection_at_line_end_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 4, None)
            ], 0
        );
    }
}

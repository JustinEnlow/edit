use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, Movement},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
    }
    Ok(())
}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended to the end of the current line.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{    //TODO: ensure this can't extend past doc text end
    selection.assert_invariants(buffer, semantics.clone());
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_width = buffer.line_width(line_number, false);    //doesn't include newline
    let line_start = buffer.line_to_char(line_number);
    let line_end = line_start.saturating_add(line_width);   //index at end of line text, not including newline  //nth_next_grapheme_index(line_start, line_width, text)?

    match semantics{
        CursorSemantics::Bar => {
            if selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
            selection.put_cursor(line_end, buffer, Movement::Extend, semantics, true)
        }
        CursorSemantics::Block => {
            //if self.cursor(semantics) == line_end.saturating_sub(1)
            if selection.cursor(buffer, semantics.clone()) == buffer.previous_grapheme_boundary_index(line_end)
            || selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
            let start_line = buffer.char_to_line(selection.range.start);
            let end_line = buffer.char_to_line(selection.range.end);
            if selection.cursor(buffer, semantics.clone()) == selection.range.start && end_line > start_line{
                selection.put_cursor(line_end, buffer, Movement::Extend, semantics, true)  //put cursor over newline, if extending from a line below
            }else{
                //self.put_cursor(line_end.saturating_sub(1), text, Movement::Extend, semantics, true)
                selection.put_cursor(buffer.previous_grapheme_boundary_index(line_end), buffer, Movement::Extend, semantics, true)
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::utilities::extend_selection_line_end;
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
        
        let result = extend_selection_line_end::application_impl(&mut app, semantics.clone());
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
        
        assert!(extend_selection_line_end::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn moves_to_line_text_end_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),
                (4, 4, None)
            ], 0, 
            vec![
                (0, 3, Some(3)),
                (4, 8, Some(4))
            ], 0
        );
    }
    #[test] fn moves_to_line_text_end_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
            vec![
                (0, 3, Some(2)),
                (4, 8, Some(3))
            ], 0
        );
    }
    
    #[test] fn with_mixed_valid_and_invalid_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 3, None),
                (5, 5, None)
            ], 0, 
            vec![
                (3, 3, None),
                (5, 8, Some(4))
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (2, 3, None),
                (4, 5, None)
            ], 0, 
            vec![
                (2, 3, None),
                (4, 8, Some(3))
            ], 0
        );
    }
    
    #[test] fn errors_if_already_at_line_text_end_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 3, None),
                (8, 8, None)
            ], 0
        );
    }
    #[test] fn errors_if_already_at_line_text_end_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (2, 3, None),
                (7, 8, None)
            ], 0
        );
    }

    //Only applies to block cursor semantics
    #[test] fn error_if_already_at_line_end(){  //with cursor over newline char
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 4, None),
                (8, 9, None)
            ], 0
        );
    }
}

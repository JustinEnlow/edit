use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
    }
    Ok(())
}

/// Returns a new instance of [`Selection`] encompassing the current line.
//TODO: make pub fn select_line //should this include newline at end of line? //should this include indentation at start of line? //vscode includes both, as does kakoune
//TODO: if called on empty last line, this moves the selection to second to last line end, instead it should error
fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    //vs code selects all spanned lines...  maybe caller can make that determination...
    if selection.spans_multiple_lines(buffer){return Err(SelectionError::SpansMultipleLines);}    //make specific error. SpansMultipleLines or something...
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}

    let line = buffer.char_to_line(selection.range.start);
    let line_start = buffer.line_to_char(line);
    let line_end = line_start + buffer.line_width(line, true);

    if selection.range.start == line_start && selection.range.end == line_end{Err(SelectionError::ResultsInSameState)}
    else{
        let mut selection = selection.clone();
        selection.range.start = line_start;
        selection.range.end = line_end;
        selection.direction = crate::selection::ExtensionDirection::Forward;
        //TODO?: maybe update stored line offset?...
        Ok(selection)
    }
}

#[cfg(test)]
mod tests{
    use crate::utilities::select_line;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::View,
    };

    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));

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
        
        let result = select_line::application_impl(&mut app, semantics.clone());
        assert!(!result.is_err());
        
        assert_eq!(expected_selections, app.selections);
        assert!(!app.buffer.is_modified());
    }
    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        assert!(select_line::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn normal_use_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),
                (4, 4, None)
            ], 0, 
            vec![
                (0, 4, None),
                (4, 9, None)
            ], 0
        );
    }
    #[test] fn normal_use_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
            vec![
                (0, 4, None),
                (4, 9, None)
            ], 0
        );
    }
    #[test] fn should_succeed_if_mixed_selection_spanning_multiple_lines_and_valid_selection(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 12, None)
            ], 0, 
            vec![
                (0, 4, None),
                (4, 12, None)
            ], 0
        );
    }

    #[test] fn errors_if_selection_spans_multiple_lines_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 12, None)
            ], 0
        );
    }
    #[test] fn errors_if_selection_spans_multiple_lines_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 12, None)
            ], 0
        );
    }

    //TODO: have test with mixed new state and same state selections. should succeed...
    #[test] fn errors_if_results_in_same_state_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 4, None)
            ], 0
        );
    }
    #[test] fn errors_if_results_in_same_state_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 4, None)
            ], 0
        );
    }

    //TODO: have test with mixed valid selection and selection at doc end and line empty. should succeed...
    #[test] fn errors_if_at_doc_end_and_line_empty_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 14, None)
            ], 0
        );
    }
    #[test] fn errors_if_at_doc_end_and_line_empty_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 15, None)
            ], 0
        );
    }
}

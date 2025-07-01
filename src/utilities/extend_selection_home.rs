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

/// Returns a new instance of [`Selection`] with the [`Selection`] extended to absolute start of line, or line text start, depending on [`Selection`] `head` position.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    use crate::utilities::extend_selection_line_text_start;
    use crate::utilities::extend_selection_line_start;
    
    selection.assert_invariants(buffer, semantics.clone());
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_start = buffer.line_to_char(line_number);
    let text_start_offset = buffer.first_non_whitespace_character_offset(line_number);
    let text_start = line_start.saturating_add(text_start_offset);  //nth_next_grapheme_index(line_start, text_start_offset, text)?

    if selection.cursor(buffer, semantics.clone()) == text_start{extend_selection_line_start::selection_impl(selection, buffer, semantics.clone())}
    else{extend_selection_line_text_start::selection_impl(selection, buffer, semantics)}
}

#[cfg(test)]
mod tests{
    use crate::utilities::extend_selection_home;
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
        
        let result = extend_selection_home::application_impl(&mut app, semantics.clone());
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
        
        assert!(extend_selection_home::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn when_cursor_past_line_text_start_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "    idk\n    something\n", 
            vec![
                (6, 6, None),
                (16, 16, None)
            ], 0, 
            vec![
                (6, 4, Some(4)),
                (16, 12, Some(4))
            ], 0
        );
    }
    #[test] fn when_cursor_past_line_text_start_block_semantics(){
        test(
            CursorSemantics::Block, 
            "    idk\n    something\n", 
            vec![
                (6, 7, None),
                (16, 17, None)
            ], 0, 
            vec![
                (7, 4, Some(4)),
                (17, 12, Some(4))
            ], 0
        );
    }
    #[test] fn when_cursor_at_line_text_start_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "    idk\n    something\n", 
            vec![
                (4, 4, None),
                (12, 12, None)
            ], 0, 
            vec![
                (4, 0, Some(0)),
                (12, 8, Some(0))
            ], 0
        );
    }
    #[test] fn when_cursor_at_line_text_start_block_semantics(){
        test(
            CursorSemantics::Block, 
            "    idk\n    something\n", 
            vec![
                (4, 5, None),
                (12, 13, None)
            ], 0, 
            vec![
                (5, 0, Some(0)),
                (13, 8, Some(0))
            ], 0
        );
    }
    #[test] fn when_cursor_before_line_text_start_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "    idk\n    something\n", 
            vec![
                (2, 2, None),
                (10, 10, None)
            ], 0, 
            vec![
                (2, 4, Some(4)),
                (10, 12, Some(4))
            ], 0
        );
    }
    #[test] fn when_cursor_before_line_text_start_block_semantics(){
        test(
            CursorSemantics::Block, 
            "    idk\n    something\n", 
            vec![
                (2, 3, None),
                (10, 11, None)
            ], 0, 
            vec![
                (2, 5, Some(4)),
                (10, 13, Some(4))
            ], 0
        );
    }

    #[test] fn with_mixed_valid_and_invalid_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),
                (6, 6, None)
            ], 0, 
            vec![
                (0, 0, None),
                (6, 4, Some(0))
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (6, 7, None)
            ], 0, 
            vec![
                (0, 1, None),
                (7, 4, Some(0))
            ], 0
        );
    }
    
    #[test] fn errors_when_line_start_and_line_text_start_and_cursor_position_all_equal_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),
                (4, 4, None)
            ], 0
        );
    }
    #[test] fn errors_when_line_start_and_line_text_start_and_cursor_position_all_equal_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0
        );
    }
}

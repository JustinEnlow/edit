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
fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let mut selection = selection.clone();
    selection.assert_invariants(buffer, semantics.clone());
    
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == 0{
        return Err(SelectionError::ResultsInSameState);
    }
    
    let current_line = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let goal_line_number = current_line.saturating_sub(1);
    let start_of_line = buffer.line_to_char(goal_line_number);
    let line_width = buffer.line_width(goal_line_number, false);
    
    // Use current stored line offset or calculate it if None
    let stored_line_offset = selection.stored_line_offset.unwrap_or_else(|| {
        buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone()))
    });

    // Calculate the new position based on line width
    let mut new_position = if stored_line_offset < line_width{
        start_of_line + stored_line_offset
    }else{
        start_of_line + line_width
    };
    new_position = new_position.min(buffer.len_chars());
    
    selection.range.start = new_position;
    selection.range.end = match semantics{
        CursorSemantics::Bar => new_position,
        CursorSemantics::Block => buffer.next_grapheme_boundary_index(new_position)
    };
    selection.direction = ExtensionDirection::None;
    selection.stored_line_offset = Some(stored_line_offset);

    selection.assert_invariants(buffer, semantics);
    Ok(selection)
}

#[cfg(test)]
mod tests{
    use crate::utilities::move_cursor_up;
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
        
        let result = move_cursor_up::application_impl(&mut app, semantics.clone());
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
        
        assert!(move_cursor_up::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn to_shorter_line_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshits\n", 
            vec![
                (8, 8, None),
                (14, 14, None)
            ], 0, 
            vec![
                (3, 3, Some(4)),    //notice this maintains stored line position of selection before operation
                (8, 8, Some(5))     //notice this maintains stored line position of selection before operation
            ], 0
        );
    }
    #[test] fn to_shorter_line_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshits\n", 
            vec![
                (8, 9, None),
                (14, 15, None)
            ], 0, 
            vec![
                (3, 4, Some(4)),
                (8, 9, Some(5))
            ], 0
        );
    }
    
    #[test] fn to_line_with_equal_len_or_more_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idfk\nsome\nshit\n", 
            vec![
                (9, 9, None),
                (14, 14, None)
            ], 0, 
            vec![
                (4, 4, Some(4)),
                (9, 9, Some(4))
            ], 0
        );
    }
    #[test] fn to_line_with_equal_len_or_more_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idfk\nsome\nshit\n", 
            vec![
                (9, 10, None),
                (14, 15, None)
            ], 0, 
            vec![
                (4, 5, Some(4)),
                (9, 10, Some(4))
            ], 0
        );
    }

    //with mixed valid and invalid selections   //one on top line, one not
    #[test] fn with_mixed_valid_and_invalid_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),
                (9, 9, None)
            ], 0, 
            vec![
                (0, 0, None),
                (4, 4, Some(0))
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (9, 10, None)
            ], 0, 
            vec![
                (0, 1, None),
                (4, 5, Some(0))
            ], 0
        );
    }

    //merges overlapping resultant selections   //one on top line, one on second
    #[test] fn merges_overlapping_resultant_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),
                (4, 4, None)
            ], 0, 
            vec![
                (0, 0, Some(0))
            ], 0
        );
    }
    #[test] fn merges_overlapping_resultant_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
            vec![
                (0, 1, Some(0))
            ], 0
        );
    }
    
    //with extended selections collapses
    #[test] fn with_extended_selection_collapses_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 8, None),
                (9, 13, None)
            ], 0, 
            vec![
                (3, 3, Some(4)),
                (8, 8, Some(4))
            ], 0
        );
    }
    #[test] fn with_extended_selection_collapses_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 9, None),
                (9, 14, None)
            ], 0, 
            vec![
                (3, 4, Some(4)),
                (8, 9, Some(4))
            ], 0
        );
    }
    
    #[test] fn errors_if_single_selection_on_topmost_line_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None)
            ], 0
        );
    }
    #[test] fn errors_if_single_selection_on_topmost_line_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 0
        );
    }
}

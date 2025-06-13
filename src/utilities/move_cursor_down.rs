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
    
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){
        return Err(SelectionError::ResultsInSameState);
    }
    
    let current_line = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let goal_line_number = core::cmp::Ord::min(current_line.saturating_add(1), buffer.len_lines().saturating_sub(1));
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
    use crate::utilities::move_cursor_down;
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
        
        let result = move_cursor_down::application_impl(&mut app, semantics.clone());
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
        
        assert!(move_cursor_down::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    //to shorter line
    #[test] fn to_shorter_line_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "shits\nsome\nidk", 
            vec![
                (5, 5, None),
                (10, 10, None)
            ], 0, 
            vec![
                (10, 10, Some(5)),  //notice this maintains stored line position of selection before operation
                (14, 14, Some(4))   //notice this maintains stored line position of selection before operation
            ], 0
        );
    }
    #[test] fn to_shorter_line_block_semantics(){
        test(
            CursorSemantics::Block, 
            "shits\nsome\nidk", 
            vec![
                (5, 6, None),
                (10, 11, None)
            ], 0, 
            vec![
                (10, 11, Some(5)),  //notice this maintains stored line position of selection before operation
                (14, 15, Some(4))   //notice this maintains stored line position of selection before operation
            ], 0
        );
    }

    //to line with equal len or more
    #[test] fn to_line_with_equal_len_or_more_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "some\nshit\nidfk\n", 
            vec![
                (4, 4, None),
                (9, 9, None)
            ], 0, 
            vec![
                (9, 9, Some(4)),
                (14, 14, Some(4))
            ], 0
        );
    }
    #[test] fn to_line_with_equal_len_or_more_block_semantics(){
        test(
            CursorSemantics::Block, 
            "some\nshit\nidfk\n", 
            vec![
                (4, 5, None),
                (9, 10, None)
            ], 0, 
            vec![
                (9, 10, Some(4)),
                (14, 15, Some(4))
            ], 0
        );
    }
    
    //with mixed valid and invalid selections   //one on bottom line, one not
    #[test] fn with_mixed_valid_and_invalid_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 4, None),
                (14, 14, None)
            ], 0, 
            vec![
                (9, 9, Some(0)),
                (14, 14, None)
            ], 0
        );
    }
    #[test] fn with_mixed_valid_and_invalid_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 5, None),
                (14, 15, None)
            ], 0, 
            vec![
                (9, 10, Some(0)),
                (14, 15, None)
            ], 0
        );
    }
    
    //merges overlapping resultant selections   //one on bottom line, one on second
    #[test] fn merges_overlapping_resultant_selections_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 9, None),
                (14, 14, None)
            ], 0, 
            vec![
                (14, 14, Some(0))
            ], 0
        );
    }
    #[test] fn merges_overlapping_resultant_selections_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 10, None),
                (14, 15, None)
            ], 0, 
            vec![
                (14, 15, Some(0))
            ], 0
        );
    }
    
    //with extended selections collapses
    #[test] fn with_extended_selection_collapses_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 3, None),
                (4, 8, None)
            ], 0, 
            vec![
                (7, 7, Some(3)),
                (13, 13, Some(4))
            ], 0
        );
    }
    #[test] fn with_extended_selection_collapses_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 4, None),
                (4, 9, None)
            ], 0, 
            vec![
                (7, 8, Some(3)),
                (13, 14, Some(4))
            ], 0
        );
    }
    
    //errors if single selection on bottom-most line
    #[test] fn errors_if_single_selection_on_bottommost_line_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 14, None)
            ], 0
        );
    }
    #[test] fn errors_if_single_selection_on_bottommost_line_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 15, None)
            ], 0
        );
    }
}

use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, Movement},
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match app.selections.move_cursor_clearing_non_primary(&app.buffer, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
    }
    Ok(())
}

/// Returns a new instance of [`Selection`] with [`Selection`] extended to encompass all text.
fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{  //TODO: ensure this can't extend past doc text end
    selection.assert_invariants(buffer, semantics.clone());
    if selection.range.start == 0 
    && (
        selection.range.end == buffer.len_chars() || 
        selection.range.end == buffer.len_chars().saturating_add(1)
    ){return Err(SelectionError::ResultsInSameState);}
    
    let selection = selection.put_cursor(0, buffer, Movement::Move, semantics.clone(), true)?;
    selection.put_cursor(
        match semantics{
            CursorSemantics::Bar => buffer.len_chars(), 
            CursorSemantics::Block => buffer.previous_grapheme_boundary_index(buffer.len_chars())
        }, 
        buffer, 
        Movement::Extend, 
        semantics, 
        true
    )
}

#[cfg(test)]
mod tests{
    use crate::utilities::select_all;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::DisplayArea,
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
        
        let result = select_all::application_impl(&mut app, semantics.clone());
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
        
        assert!(select_all::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }
    
    //TODO: should this really be returning a selection with stored_line_position?...
    
    #[test] fn selects_all_and_clears_non_primary_selections(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
            vec![
                (0, 14, Some(4))
            ], 0
        );
    }
    #[test] fn ensure_cannot_past_text_len(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 15, None)
            ], 0, 
            vec![
                (0, 14, Some(4))
            ], 0
        );
    }
    
    #[test] fn errors_if_all_already_selected(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 14, None)
            ], 0
        );
    }
}

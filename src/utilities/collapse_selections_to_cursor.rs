use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, Movement}
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match app.selections.move_cursor_non_overlapping(&app.buffer, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
    }
    Ok(())
}

//TODO: we should allow collapsing to anchor, or collapse to anchor collapse(&self, text: &Rope, semantics: CursorSemantics, collapse_target: Anchor)
/// Returns a new instance of [`Selection`] with `anchor` aligned with cursor.
fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    selection.assert_invariants(buffer, semantics.clone());
    if !selection.is_extended(){return Err(SelectionError::ResultsInSameState);}
    selection.put_cursor(selection.cursor(buffer, semantics.clone()), buffer, Movement::Move, semantics, true)
    //if we want collapse to anchor:
    //self.put_cursor(self.anchor, text, Movement::Move, semantics, true)
}

#[cfg(test)]
mod tests{
    use crate::utilities::collapse_selections_to_cursor;
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
        
        let result = collapse_selections_to_cursor::application_impl(&mut app, semantics.clone());
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
        
        assert!(collapse_selections_to_cursor::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    //TODO: should these functions really result in selections with a stored line position?...
    
    #[test] fn collapses_to_cursor_with_multiple_selections_with_selection_forward(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 3, None),
                (4, 8, None)
            ], 0, 
            vec![
                (2, 3, Some(2)),
                (7, 8, Some(3))
            ], 0
        );
    }
    #[test] fn collapses_to_cursor_with_multiple_selections_with_selection_backward(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 0, None),
                (8, 4, None)
            ], 0, 
            vec![
                (0, 1, Some(0)),
                (4, 5, Some(0))
            ], 0
        );
    }
    
    #[test] fn collapses_to_cursor_with_mixed_extension(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 8, None)
            ], 0, 
            vec![
                (0, 1, None),
                (7, 8, Some(3))
            ], 0
        );
    }
    
    #[test] fn errors_if_already_collapsed(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0
        );
    }
    //maybe test above with single selection too...idk
}

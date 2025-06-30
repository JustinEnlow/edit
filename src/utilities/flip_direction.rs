use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics},
};

//TODO: rename to collapse_selections_to_cursor
pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match app.selections.move_cursor_non_overlapping(&app.buffer, semantics, selection_impl){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
    }
    Ok(())
}

pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    use crate::selection::ExtensionDirection;
    selection.assert_invariants(buffer, semantics.clone());
    if !selection.is_extended(){return Err(SelectionError::ResultsInSameState)}
    //Ok(
    //    Selection::new(
    //        selection.range.clone(), 
    //        match selection.direction{
    //            Direction::Forward => {Direction::Backward}
    //            Direction::Backward => {Direction::Forward}
    //        }
    //    )
    //)
    let mut new_selection = selection.clone();
    new_selection.direction = match selection.direction{
        ExtensionDirection::None => return Err(SelectionError::ResultsInSameState),
        ExtensionDirection::Forward => ExtensionDirection::Backward,
        ExtensionDirection::Backward => ExtensionDirection::Forward
    };
    Ok(new_selection)
}

#[cfg(test)]
mod tests{
    use crate::utilities::flip_direction;
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
        
        let result = flip_direction::application_impl(&mut app, semantics.clone());
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
        
        assert!(flip_direction::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn forward_selections_flip_backwards_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 3, None),
                (4, 8, None)
            ], 0, 
            vec![
                (3, 0, None),
                (8, 4, None)
            ], 0
        );
    }
    #[test] fn forward_selections_flip_backwards_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 4, None),
                (4, 9, None)
            ], 0, 
            vec![
                (4, 0, None),
                (9, 4, None)
            ], 0
        );
    }

    #[test] fn backward_selections_flip_forwards_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (3, 0, None),
                (8, 4, None)
            ], 0, 
            vec![
                (0, 3, None),
                (4, 8, None)
            ], 0
        );
    }
    #[test] fn backward_selections_flip_forwards_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 0, None),
                (9, 4, None)
            ], 0, 
            vec![
                (0, 4, None),
                (4, 9, None)
            ], 0
        );
    }

    #[test] fn non_extended_return_error_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None)
            ], 
            0
        );
    }
    #[test] fn non_extended_return_error_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 
            0
        );
    }

    //TODO: what about mixed directions? should they even be allowed?...
}

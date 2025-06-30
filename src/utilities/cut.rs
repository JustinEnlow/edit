use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
};

/// Cut single selection.
/// Copies text to clipboard and removes selected text from document.
/// Ensure single selection when calling this function.
pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    if app.selections.count() > 1{return Err(ApplicationError::SelectionsError(crate::selections::SelectionsError::MultipleSelections))}

    let selection = app.selections.primary_mut();
    // Copy the selected text to the clipboard
    app.clipboard = app.buffer.slice(selection.range.start, selection.range.end).to_string();
    crate::utilities::delete::application_impl(app, semantics)   //notice this is returning the result from delete
}

#[cfg(test)]
mod tests{
    use crate::utilities::cut;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::DisplayArea,
    };

    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, expected_text: &str, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize, expected_clipboard: &str){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));

        let expected_buffer = crate::buffer::Buffer::new(expected_text, None, false);
        let mut vec_expected_selections = Vec::new();
        for tuple in tuple_expected_selections{
            vec_expected_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &expected_buffer, semantics.clone()));
        }
        let expected_selections = Selections::new(vec_expected_selections, expected_primary, &expected_buffer, semantics.clone());
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        let result = cut::application_impl(&mut app, semantics);
        assert!(!result.is_err());
        
        assert_eq!(expected_buffer, app.buffer);
        assert_eq!(expected_selections, app.selections);
        assert_eq!(expected_clipboard, app.clipboard);
        //println!("expected: {:?}\ngot: {:?}", expected_buffer, app.buffer);
        //assert!(app.buffer.is_modified());    //is modified doesn't work with tests, because it now checks against a persistent file, which tests don't have
    }
    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        assert!(cut::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn cut_with_selection_direction_forward_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 9, None)
            ], 0, 
            "idk\nshit\n", 
            vec![
                (4, 5, Some(0))
            ], 0, 
            "some\n"
        );
    }
    #[test] fn cut_with_selection_direction_forward_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 9, None)
            ], 0, 
            "idk\nshit\n", 
            vec![
                (4, 4, Some(0))
            ], 0, 
            "some\n"
        );
    }

    #[test] fn cut_with_selection_direction_backward_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 4, None)
            ], 0, 
            "idk\nshit\n", 
            vec![
                (4, 5, Some(0))
            ], 0, 
            "some\n"
        );
    }
    #[test] fn cut_with_selection_direction_backward_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 4, None)
            ], 0, 
            "idk\nshit\n", 
            vec![
                (4, 4, Some(0))
            ], 0, 
            "some\n"
        );
    }

    #[test] fn cut_with_multiple_selections_returns_error(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 3, None),
                (4, 7, None)
            ], 0
        );
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 3, None),
                (4, 7, None)
            ], 0
        );
    }
}

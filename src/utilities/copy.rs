use crate::application::{Application, ApplicationError};
use crate::selections::SelectionsError;

/// Copy single selection to clipboard.
/// Ensure single selection when calling this function.
pub fn application_impl(app: &mut Application) -> Result<(), ApplicationError>{
    if app.selections.count() > 1{return Err(ApplicationError::SelectionsError(SelectionsError::MultipleSelections));}
    
    let selection = app.selections.primary().clone();
    // Copy the selected text to the clipboard
    app.clipboard = app.buffer.slice(selection.range.start, selection.range.end).to_string();

    Ok(())
}

#[cfg(test)]
mod tests{
    use crate::utilities::copy;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::DisplayArea,
    };

    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, expected_clipboard: &str){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        let result = copy::application_impl(&mut app);
        assert!(!result.is_err());
        
        assert_eq!(expected_clipboard, app.clipboard);
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
        
        assert!(copy::application_impl(&mut app).is_err());
        assert!(!app.buffer.is_modified());
    }

    //TODO: copy with no selection extension
        //should fail with bar semantics?...
        //should copy single char with block semantics

    #[test] fn copy_with_selection_direction_forward_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 9, None)
            ], 0, 
            "some\n"
        );
    }
    #[test] fn copy_with_selection_direction_forward_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (4, 9, None)
            ], 0, 
            "some\n"
        );
    }

    #[test] fn copy_with_selection_direction_backward_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 4, None)
            ], 0, 
            "some\n"
        );
    }
    #[test] fn copy_with_selection_direction_backward_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 4, None)
            ], 0, 
            "some\n"
        );
    }

    #[test] fn copy_with_multiple_selections_should_error(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0
        );
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None),
                (4, 4, None)
            ], 0
        );
    }
}

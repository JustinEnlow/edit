use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
};

/// Insert clipboard contents at cursor position(s).
pub fn application_impl(app: &mut Application, use_hard_tab: bool, tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    crate::utilities::insert_string::application_impl(app, &app.clipboard.clone(), use_hard_tab, tab_width, semantics)
}

#[cfg(test)]
mod tests{
    use crate::utilities::paste;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        display_area::DisplayArea,
    };

    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, clipboard: &str, expected_text: &str, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
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
        app.clipboard = clipboard.to_string();
        
        let result = paste::application_impl(&mut app, false, 4, semantics);
        assert!(!result.is_err());
        
        assert_eq!(expected_buffer, app.buffer);
        assert_eq!(expected_selections, app.selections);
        //println!("expected: {:?}\ngot: {:?}", expected_buffer, app.buffer);
        //assert!(app.buffer.is_modified());    //is modified doesn't work with tests, because it now checks against a persistent file, which tests don't have
    }
    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, clipboard: &str){
        let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        app.clipboard = clipboard.to_string();
        
        assert!(paste::application_impl(&mut app, false, 4, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn paste_single_selection_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 10, None)
            ], 0, 
            "other\n", 
            "idk\nsome\nother\nshit\n", 
            vec![
                (15, 16, Some(0))
            ], 0
        );
    }
    #[test] fn paste_single_selection_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (9, 9, None)
            ], 0, 
            "other\n", 
            "idk\nsome\nother\nshit\n", 
            vec![
                (15, 15, Some(0))
            ], 0
        );
    }
    //TODO: paste_multi_selection_block_semantics
    //TODO: paste_multi_selection_bar_semantics

    #[test] fn errors_if_empty_clipboard(){
        test_error(
            CursorSemantics::Block, 
            "idk\nshit\n", 
            vec![
                (4, 5, None)
            ], 0, 
            ""
        );
    }

}

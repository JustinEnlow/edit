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

pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    let mut selection = selection.clone();
    selection.assert_invariants(buffer, semantics.clone());
    
    if selection.cursor(buffer, semantics.clone()) == 0{return Err(SelectionError::ResultsInSameState);}
    //if selection.cursor(buffer, semantics.clone()) == buffer.len_chars(){     //possible fix for extend left from buffer end getting stuck bug logged in todo.rs
    //    return crate::utilities::move_cursor_left::selection_impl(&selection, buffer, semantics);
    //}

    let new_position = buffer.previous_grapheme_boundary_index(selection.cursor(buffer, semantics.clone()));
    
    match semantics.clone(){
        CursorSemantics::Bar => {
            let to = Ord::min(new_position, buffer.len_chars());
            let (start, end, direction) = if to < selection.anchor(){
                (to, selection.anchor(), ExtensionDirection::Backward)
            }else{
                (selection.anchor(), to, ExtensionDirection::Forward)
            };
            selection.range.start = start;
            selection.range.end = end;
            selection.direction = direction;
        }
        CursorSemantics::Block => {
            let to = Ord::min(new_position, buffer.previous_grapheme_boundary_index(buffer.len_chars()));
            let new_anchor = match selection.direction{
                ExtensionDirection::None |
                ExtensionDirection::Forward => {
                    if to < selection.anchor(){  //could also do self.range.start
                        if let Some(char_at_cursor) = buffer.get_char(selection.cursor(buffer, semantics.clone())){
                            if char_at_cursor == '\n'{selection.anchor()}
                            else{buffer.next_grapheme_boundary_index(selection.anchor()).min(buffer.len_chars())}
                        }else{buffer.next_grapheme_boundary_index(selection.anchor()).min(buffer.len_chars())}
                    }else{selection.anchor()}
                }
                ExtensionDirection::Backward => {
                    if to >= selection.anchor(){buffer.previous_grapheme_boundary_index(selection.anchor())} //could also do self.range.end
                    else{selection.anchor()}
                }
            };

            if new_anchor <= to{    //allowing one more char past text.len_chars() for block cursor
                selection.range.start = new_anchor;
                selection.range.end = Ord::min(buffer.next_grapheme_boundary_index(to), buffer.len_chars().saturating_add(1));
                selection.direction = ExtensionDirection::Forward;
            }else{
                selection.range.start = to;
                selection.range.end = new_anchor;
                selection.direction = ExtensionDirection::Backward;
            }
        }
    }

    selection.stored_line_offset = Some(buffer.offset_from_line_start(selection.cursor(buffer, semantics.clone())));
    
    selection.assert_invariants(buffer, semantics.clone());

    Ok(selection)
}

#[cfg(test)]
mod tests{
    use crate::utilities::extend_selection_left;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::View,
    };

    //TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
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
        
        let result = extend_selection_left::application_impl(&mut app, semantics.clone());
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
        
        assert!(extend_selection_left::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    //TODO: updates stored line position on line change

    #[test] fn normal_use_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 14, None)
            ], 0, 
            vec![
                (14, 13, Some(4))
            ], 0
        );
    }
    #[test] fn normal_use_block_semantics(){    //+ trims newline from selection
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (13, 14, None),
            ], 0, 
            vec![
                (13, 12, Some(3))
            ], 0
        );
    }

    #[test] fn extends_to_doc_start_bar_semantics(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (1, 1, None)
            ], 0, 
            vec![
                (1, 0, Some(0))
            ], 0
        );
    }
    #[test] fn extends_to_doc_start_block_semantics(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (1, 2, None)
            ], 0, 
            vec![
                (2, 0, Some(0))
            ], 0
        );
    }

    #[test] fn with_previously_forward_extended_selection(){
        test(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 14, None)
            ], 0, 
            vec![
                (0, 13, Some(4))
            ], 0
        );
    }

    #[test] fn errors_if_cursor_at_doc_start_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 0, None)
            ], 0
        );
    }
    #[test] fn errors_if_cursor_at_doc_start_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 0
        );
    }

    #[test] fn errors_if_already_extended_backward_at_doc_start_bar_semantics(){
        test_error(
            CursorSemantics::Bar, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 0, None)
            ], 0
        );
    }
    #[test] fn errors_if_already_extended_backward_at_doc_start_block_semantics(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 0, None)
            ], 0
        );
    }
}

use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    selections::{Selections, SelectionsError}
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match selections_impl(&app.selections, &app.buffer, semantics){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
    }
    Ok(())
}

//TODO: add selection above/below fns don't work as expected when multiple selections on same line. only adds primary selection range above/below

/// Adds a new [`Selection`] directly above the top-most [`Selection`], with the same start and end offsets from line start, if possible.
fn selections_impl(selections: &Selections, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selections, SelectionsError>{
    assert!(selections.count() > 0);  //ensure at least one selection in selections

    let top_selection = selections.first();
    let top_selection_line = buffer.char_to_line(top_selection.range.start);
    if top_selection_line == 0{return Err(SelectionsError::CannotAddSelectionAbove);}
    // should error if any selection spans multiple lines. //callee can determine appropriate response behavior in this case        //vscode behavior is to extend topmost selection up one line if any selection spans multiple lines
    for selection in &selections.inner{  //self.selections.iter(){   //change suggested by clippy lint
        if selection.spans_multiple_lines(buffer){return Err(SelectionsError::SpansMultipleLines);}
    }

    // using primary selection here, because that is the selection we want our added selection to emulate, if possible with the available text
    let start_offset = buffer.offset_from_line_start(selections.primary().range.start);
    let end_offset = start_offset.saturating_add(selections.primary().range.end.saturating_sub(selections.primary().range.start));  //start_offset + (end char index - start char index)
    let line_above = top_selection_line.saturating_sub(1);
    let line_start = buffer.line_to_char(line_above);
    let line_text = buffer.inner.line(line_above);
    let line_width = buffer.line_width(line_above, false);
    let line_width_including_newline = buffer.line_width(line_above, true);
    let (start, end) = if line_text.to_string().is_empty() || line_text == "\n"{    //should be impossible for the text in the line above first selection to be empty. is_empty() check is redundant here...
        match semantics{
            CursorSemantics::Bar => (line_start, line_start),
            CursorSemantics::Block => (line_start, buffer.next_grapheme_boundary_index(line_start))
        }
    }
    else if selections.primary().is_extended(){
        if start_offset < line_width{   //should we exclusively handle start_offset < line_width && end_offset < line_width as well?
            (line_start.saturating_add(start_offset), line_start.saturating_add(end_offset.min(line_width_including_newline))) //start offset already verified within line text bounds
        }
        else{
            // currently same as non extended. this might change...
            match semantics{    //ensure adding the offsets doesn't make this go past line width
                CursorSemantics::Bar => (line_start.saturating_add(start_offset.min(line_width)), line_start.saturating_add(start_offset.min(line_width))),
                CursorSemantics::Block => (line_start.saturating_add(start_offset.min(line_width)), buffer.next_grapheme_boundary_index(line_start.saturating_add(start_offset.min(line_width))))
            }
        }
    }
    else{  //not extended
        match semantics{    //ensure adding the offsets doesn't make this go past line width
            CursorSemantics::Bar => (line_start.saturating_add(start_offset.min(line_width)), line_start.saturating_add(start_offset.min(line_width))),
            CursorSemantics::Block => (line_start.saturating_add(start_offset.min(line_width)), buffer.next_grapheme_boundary_index(line_start.saturating_add(start_offset.min(line_width))))
        }
    };

    let mut selection = selections.primary().clone();
    selection.range.start = start;
    selection.range.end = end;
    Ok(selections.push_front(selection, false))
}

#[cfg(test)]
mod tests{
    use crate::utilities::add_selection_above;
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
        
        let result = add_selection_above::application_impl(&mut app, semantics.clone());
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
        
        assert!(add_selection_above::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    //to line with same len or more
        //non extended
            //bar
                //selection direction forward
                #[test] fn to_line_with_same_len_or_more_with_non_extended_selection_with_direction_forward_bar_semantics(){
                    test(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (9, 9, None),
                        ], 0, 
                        vec![
                            (4, 4, None),
                            (9, 9, None)
                        ], 1
                    );
                }
                //selection direction backward
                #[test] fn to_line_with_same_len_or_more_with_non_extended_selection_with_direction_backward_bar_semantics(){
                    test(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (9, 9, None)
                        ], 0, 
                        vec![
                            (4, 4, None),
                            (9, 9, None)
                        ], 1
                    );
                }
            //block
                //selection direction forward
                //selection direction backward
        //extended
            //bar
                //selection direction forward
                #[test] fn to_line_with_same_len_or_more_with_extended_selection_with_direction_forward_bar_semantics(){
                    test(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (9, 13, None)
                        ], 0, 
                        vec![
                            (4, 8, None),
                            (9, 13, None)
                        ], 1
                    );
                }
                //selection direction backward
                #[test] fn to_line_with_same_len_or_more_with_extended_selection_with_direction_backward_bar_semantics(){
                    test(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (13, 9, None)
                        ], 0, 
                        vec![
                            (8, 4, None),
                            (13, 9, None)
                        ], 1
                    );
                }
            //block
                //selection direction forward
                //selection direction backward
    //to shorter line
        //non extended
            //bar
                //selection direction forward
                #[test] fn to_shorter_line_with_non_extended_selection_with_direction_forward_bar_semantics(){
                    test(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (4, 4, None)
                        ], 0, 
                        vec![
                            (0, 0, None),
                            (4, 4, None)
                        ], 1
                    );
                }
                //selection direction backward
                #[test] fn to_shorter_line_with_non_extended_selection_with_direction_backward_bar_semantics(){
                    test(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (4, 4, None)
                        ], 0, 
                        vec![
                            (0, 0, None),
                            (4, 4, None)
                        ], 1
                    );
                }
            //block
                //selection direction forward
                //selection direction backward
        //extended
            //bar
                //selection direction forward
                #[test] fn to_shorter_line_with_extended_selection_with_direction_forward_bar_semantics(){
                    test(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (4, 8, None)
                        ], 0, 
                        vec![
                            (0, 4, None),
                            (4, 8, None)
                        ], 1
                    );
                }
                //selection direction backward
                #[test] fn to_shorter_line_with_extended_selection_with_direction_backward_bar_semantics(){
                    test(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (8, 4, None)
                        ], 0, 
                        vec![
                            (4, 0, None),
                            (8, 4, None)
                        ], 1
                    );
                }
            //block
                //selection direction forward
                //selection direction backward
    //to line with only newline char
        //non extended
            //bar
                //selection direction forward
                //selection direction backward
            //block
                //selection direction forward
                //selection direction backward
        //extended
            //bar
                //selection direction forward
                //selection direction backward
            //block
                //selection direction forward
                //selection direction backward
    //with multiple selections on same line (should merge overlapping if needed)
        //non extended
            //bar
                //selection direction forward
                //selection direction backward
            //block
                //selection direction forward
                //selection direction backward
        //extended
            //bar
                //selection direction forward
                //selection direction backward
            //block
                //selection direction forward
                //selection direction backward
    //should error if on top line
        //non extended
            //bar
                //selection direction forward
                #[test] fn should_error_if_on_top_line_with_non_extended_selection_with_direction_forward_bar_semantics(){
                    test_error(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (0, 0, None)
                        ], 0
                    );
                }
                //selection direction backward
                #[test] fn should_error_if_on_top_line_with_non_extended_selection_with_direction_backward_bar_semantics(){
                    test_error(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (0, 0, None)
                        ], 0
                    );
                }
            //block
                //selection direction forward
                //selection direction backward
        //extended
            //bar
                //selection direction forward
                #[test] fn should_error_if_on_top_line_with_extended_selection_with_direction_forward_bar_semantics(){
                    test_error(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (0, 3, None)
                        ], 0
                    );
                }
                //selection direction backward
                #[test] fn should_error_if_on_top_line_with_extended_selection_with_direction_backward_bar_semantics(){
                    test_error(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (3, 0, None)
                        ], 0
                    );
                }
            //block
                //selection direction forward
                //selection direction backward
    //should error if any selection is multiline
        //non extended
            //bar
                //selection direction forward
                //selection direction backward
            //block
                //selection direction forward
                //selection direction backward
        //extended
            //bar
                //selection direction forward
                #[test] fn should_error_if_any_selection_is_multiline_with_direction_forward_bar_semantics(){
                    test_error(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (0, 9, None)
                        ], 0
                    );
                }
                //selection direction backward
                #[test] fn should_error_if_any_selection_is_multiline_with_direction_backward_bar_semantics(){
                    test_error(
                        CursorSemantics::Bar, 
                        "idk\nsome\nshit\n", 
                        vec![
                            (9, 0, None)
                        ], 0
                    );
                }
            //block
                //selection direction forward
                //selection direction backward
}

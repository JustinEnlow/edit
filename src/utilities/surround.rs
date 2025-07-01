use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, CursorSemantics},
    selections::{Selections, SelectionsError}
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match selections_impl(&app.selections, &app.buffer, semantics){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
    }
    Ok(())
}

pub fn selections_impl(selections: &Selections, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selections, SelectionsError>{
    let mut new_selections = Vec::with_capacity(2 * selections.count());
    let mut num_pushed: usize = 0;
    let primary_selection = selections.primary();
    let mut primary_selection_index = selections.primary_selection_index;
    for selection in &selections.inner{
        let surrounds = selection_impl(selection, buffer);
        //if selection == primary_selection{
        //    primary_selection_index = num_pushed;//.saturating_sub(1);
        //}
        //for surround in surrounds{
        //    new_selections.push(surround);
        //    num_pushed = num_pushed + 1;
        //}
        if surrounds.is_empty(){    //needed to handle mixed valid and invalid selections
            if selections.count() == 1{return Err(SelectionsError::ResultsInSameState);}
            if selection == primary_selection{
                primary_selection_index = num_pushed;//.saturating_sub(1);
            }
            new_selections.push(selection.clone());
            num_pushed = num_pushed + 1;
        }
        else{
            if selection == primary_selection{
                primary_selection_index = num_pushed;//.saturating_sub(1);
            }
            for surround in surrounds{
                new_selections.push(surround);
                num_pushed = num_pushed + 1;
            }
        }
    }
    assert!(!new_selections.is_empty());
    //if new_selections.is_empty(){Err(SelectionsError::ResultsInSameState)} //TODO: create better error?...
    //else{
        Ok(Selections::new(new_selections, primary_selection_index, buffer, semantics))
    //}
}

#[must_use] pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer) -> Vec<Selection>{
    //TODO: selection.assert_invariants(text, semantics);
    let mut surround_selections = Vec::new();
    if selection.range.start == buffer.len_chars(){return surround_selections;}
    //let first_selection = Selection::new(Range::new(selection.range.start, text_util::next_grapheme_index(selection.range.start, text)), Direction::Forward);
    let mut first_selection = selection.clone();
    first_selection.range.start = selection.range.start;
    first_selection.range.end = buffer.next_grapheme_boundary_index(selection.range.start);
    first_selection.direction = crate::selection::ExtensionDirection::None;
    //let second_selection = Selection::new(Range::new(selection.range.end, text_util::next_grapheme_index(selection.range.end, text)), Direction::Forward);
    let mut second_selection = selection.clone();
    second_selection.range.start = selection.range.end;
    second_selection.range.end = buffer.next_grapheme_boundary_index(selection.range.end);
    second_selection.direction = crate::selection::ExtensionDirection::None;

    surround_selections.push(first_selection);
    surround_selections.push(second_selection);
    surround_selections
}

#[cfg(test)]
mod tests{
    use crate::utilities::surround;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        display_area::DisplayArea,
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
        
        let result = surround::application_impl(&mut app, semantics.clone());
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
        
        assert!(surround::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_non_extended_selection(){   //also ensures primary updates properly
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 1, 
            vec![
                (0, 1, None),
                (1, 2, None),
                (4, 5, None),
                (5, 6, None)
            ], 2
        );
    }
    //TODO: need to handle bar semantics
    //#[test] fn with_non_extended_selection_bar_semantics(){   //also ensures primary updates properly
    //    test(
    //        CursorSemantics::Bar, 
    //        "idk\nsome\nshit\n", 
    //        vec![
    //            Selection::new(Range::new(0, 0), Direction::Forward),
    //            Selection::new(Range::new(4, 4), Direction::Forward),
    //        ], 1, 
    //        vec![
    //            Selection::new(Range::new(0, 0), Direction::Forward),
    //            Selection::new(Range::new(1, 1), Direction::Forward),
    //            Selection::new(Range::new(4, 4), Direction::Forward),
    //            Selection::new(Range::new(5, 5), Direction::Forward),
    //        ], 2
    //    );
    //}
    
    #[test] fn with_extended_selection(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 3, None),
                (4, 8, None)
            ], 0, 
            vec![
                (0, 1, None),
                (3, 4, None),
                (4, 5, None),
                (8, 9, None)
            ], 0
        );
    }

    //mixed valid and invalid selections  //one at doc end, one not
    #[test] fn mixed_valid_and_invalid_selections(){    //also ensures primary updates properly
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (14, 15, None)
            ], 1, 
            vec![
                (0, 1, None),
                (1, 2, None),
                (14, 15, None)
            ], 2
        );
    }
    
    #[test] fn errors_if_single_selection_at_doc_end(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (14, 15, None)
            ], 0
        );
    }
}

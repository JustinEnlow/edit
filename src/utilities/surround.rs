use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, CursorSemantics},
    selections::{Selections, SelectionsError}
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match selections_impl(&app.selections, &app.buffer, semantics){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
//    }
//    Ok(())
//}

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

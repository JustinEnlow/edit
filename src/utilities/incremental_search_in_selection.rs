use crate::{
    application::{Application, ApplicationError},
    selections::{Selections, SelectionsError},
    selection::{Selection, ExtensionDirection, CursorSemantics},
};
use regex::Regex;

pub fn application_impl(app: &mut Application, search_text: &str, selections_before_search: &Selections, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match selections_impl(selections_before_search, search_text, &app.buffer, semantics){
        Ok(new_selections) => {
            app.selections = new_selections;
            Ok(())
        }
        Err(_) => {
            app.selections = selections_before_search.clone();
            Err(ApplicationError::InvalidInput)
        }
    }
}

//TODO: maybe. if no selection extended, search whole text
/// 
/// # Errors
/// when no matches.
pub fn selections_impl(selections: &Selections, input: &str, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selections, SelectionsError>{
    if input.is_empty(){return Err(SelectionsError::NoSearchMatches);}
    let mut new_selections = Vec::new();
    let mut num_pushed: usize = 0;
    let primary_selection = selections.primary();
    //let mut primary_selection_index = self.primary_selection_index;
    let mut primary_selection_index = 0;
    
    for selection in &selections.inner{  //self.selections.iter(){   //change suggested by clippy lint
        let matches = selection_impl(selection, input, buffer);
        if selection == primary_selection{
            primary_selection_index = num_pushed.saturating_sub(1);
        }
        for search_match in matches{
            new_selections.push(search_match);
            num_pushed = num_pushed + 1;
        }
    }

    if new_selections.is_empty(){Err(SelectionsError::NoSearchMatches)}
    else{
        Ok(Selections::new(new_selections, primary_selection_index, buffer, semantics))
    }
}

/// Returns a [`Vec`] of [`Selection`]s where the underlying text is a match for the `input` search string.
#[must_use] pub fn selection_impl(selection: &Selection, input: &str, buffer: &crate::buffer::Buffer) -> Vec<Selection>{   //text should be the text within a selection, not the whole document text       //TODO: -> Result<Vec<Selection>>
    let mut selections = Vec::new();
    let start = selection.range.start;

    //match Regex::new(input){
    //    Ok(regex) => {
    //        for search_match in regex.find_iter(&text.to_string()[start..self.range.end.min(text.len_chars())]){
    //            selections.push(Selection::new(search_match.start().saturating_add(start), search_match.end().saturating_add(start)));
    //        }
    //    }
    //    Err(_) => {}    //return error FailedToParseRegex
    //}
    if let Ok(regex) = Regex::new(input){
        for search_match in regex.find_iter(&buffer.inner.to_string()[start..selection.range.end.min(buffer.len_chars())]){
            //selections.push(Selection::new(search_match.start().saturating_add(start), search_match.end().saturating_add(start)));
            //selections.push(Selection::new(Range::new(search_match.start().saturating_add(start), search_match.end().saturating_add(start)), Direction::Forward));
            let mut new_selection = selection.clone();
            new_selection.range.start = search_match.start().saturating_add(start);
            new_selection.range.end = search_match.end().saturating_add(start);
            new_selection.direction = ExtensionDirection::Forward;
            selections.push(new_selection);
        }
    }
    //else{/*return error FailedToParseRegex*/}

    if selections.is_empty(){
        //return NoMatch error      //this error is not strictly necessary since caller can just check for an empty return vec
    }
    selections
}

#[cfg(test)]
mod tests{
    #[ignore] #[test] fn implement_tests(){
        unimplemented!()
    }
}

use crate::{
    application::{Application, ApplicationError},
    selections::{Selections, SelectionsError},
    selection::{Selection, /*Extension*/Direction, CursorSemantics},
    range::Range,
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

//TODO: impl tests in src/selections_tests
fn selections_impl(selections: &Selections, input: &str, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selections, SelectionsError>{
    if input.is_empty(){return Err(SelectionsError::NoSearchMatches);}
    let mut new_selections = Vec::new();
    let mut num_pushed: usize = 0;
    let primary_selection = selections.primary();
    let mut primary_selection_index = 0;
    
    for selection in &selections.inner{
        let matches = selection_impl(selection, input, buffer);
        if matches.is_empty(){
            if selections.count() == 1{return Err(SelectionsError::NoSearchMatches);}
            if selection == primary_selection{
                primary_selection_index = num_pushed.saturating_sub(1);
            }
            new_selections.push(selection.clone());
            num_pushed = num_pushed + 1;
        }
        else{
            if selection == primary_selection{
                primary_selection_index = num_pushed.saturating_sub(1);
            }
            for search_match in matches{
                new_selections.push(search_match);
                num_pushed = num_pushed + 1;
            }
        }
    }

    let new_selections = Selections::new(new_selections, primary_selection_index, buffer, semantics.clone());
    if new_selections == *selections{return Err(SelectionsError::ResultsInSameState);}

    Ok(new_selections)
}

/// Returns a [`Vec`] of [`Selection`]s containing each part of the current selection except the split pattern.
#[must_use] fn selection_impl(selection: &Selection, pattern: &str, buffer: &crate::buffer::Buffer) -> Vec<Selection>{
    let mut selections = Vec::new();
    if let Ok(regex) = Regex::new(pattern){
        let mut start = selection.range.start; //0;
        let mut found_split = false;
        // Iter over each split, and push the retained selection before it, if any...       TODO: test split at start of selection
        for split in regex.find_iter(&buffer./*inner.*/to_string()[selection.range.start..selection.range.end.min(buffer.len_chars())]){
            found_split = true;
            let selection_range = Range::new(start, split.start().saturating_add(selection.range.start));
            if selection_range.start < selection_range.end{
                let mut new_selection = selection.clone();
                new_selection.range.start = selection_range.start;
                new_selection.range.end = selection_range.end;
                new_selection.extension_direction = Some(Direction::Forward);
                selections.push(new_selection);
            }
            start = split.end().saturating_add(selection.range.start);
        }
        // Handle any remaining text after the last split
        //if split found and end of last split < selection end
        if found_split && start < selection.range.end.min(buffer.len_chars()){
            let mut new_selection = selection.clone();
            new_selection.range.start = start;
            new_selection.range.end = selection.range.end.min(buffer.len_chars());
            new_selection.extension_direction = Some(Direction::Forward);
            selections.push(new_selection);
        }
    }
    selections
}

#[cfg(test)]
mod tests{
    #[ignore] #[test] fn implement_tests(){
        todo!()
    }
}

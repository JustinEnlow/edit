use crate::{
    application::{Application, ApplicationError},
    selections::{Selections, SelectionsError},
    selection::{Selection, /*Extension*/Direction, CursorSemantics},
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
    if let Ok(regex) = Regex::new(input){
        //regex returns byte indices, and the current Selection impl uses char indices...
        for search_match in regex.find_iter(&buffer.to_string()[start..selection.range.end.min(buffer.len_chars())]){
            let mut new_selection = selection.clone();
            //if we used char indexing instead of byte indexing, we could use buffer.byte_to_char(search_match.start()).saturating_add(start)
            //new_selection.range.start = search_match.start().saturating_add(start);
            new_selection.range.start = buffer.byte_to_char(search_match.start()).saturating_add(start);
            //new_selection.range.end = search_match.end().saturating_add(start);
            new_selection.range.end = buffer.byte_to_char(search_match.end()).saturating_add(start);
            //new_selection.extension_direction = Some(Direction::Forward);//ExtensionDirection::Forward;
            new_selection.extension_direction = if buffer.next_grapheme_char_index(new_selection.range.start) == new_selection.range.end{None}
            else{Some(Direction::Forward)};
            selections.push(new_selection);
        }
    }
    //else{/*return error FailedToParseRegex*/} //no match found if regex parse fails
    selections  //if selections empty, no match found
}

#[cfg(test)]
mod tests{
    use crate::{
        selection::{Selection, Direction},
        range::Range,
        buffer::Buffer,
        utilities::incremental_search_in_selection
    };

    #[test] fn search_hard_tab(){
        let buffer_text = "\tidk\nsome\nshit\n";
        let buffer = Buffer::new(buffer_text, None, false);
        let selection = Selection::new_unchecked(Range::new(0, buffer.chars().count()), Some(Direction::Forward), None);
        let expected_selections = vec![
            //Selection::new_unchecked(Range::new(0, 1), None, None)
            Selection::new_unchecked(Range::new(0, "\t".chars().count()), None, None)
        ];
        assert_eq!(expected_selections, incremental_search_in_selection::selection_impl(&selection, "\t", &buffer));
    }

    #[test] fn search_multibyte_grapheme(){
        let buffer_text = "a̐éö̲\r\n";
        let buffer = Buffer::new(buffer_text, None, false);
        let selection = Selection::new_unchecked(Range::new(0, buffer_text.chars().count()), Some(Direction::Forward), None);
        let expected_selections = vec![
            //Selection::new_unchecked(Range::new(0, 2), None, None)    //a̐ is 2 chars(unicode code points)
            Selection::new_unchecked(Range::new(0, "a̐".chars().count()), None, None)
        ];
        assert_eq!(expected_selections, incremental_search_in_selection::selection_impl(&selection, "a̐", &buffer));
    }
}

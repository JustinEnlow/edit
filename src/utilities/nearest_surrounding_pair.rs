use crate::{
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

//TODO: for some reason, repeated calls after successfully selecting bracket pair do not return same state error...
pub fn selections_impl(selections: &Selections, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selections, SelectionsError>{
    let mut new_selections = Vec::with_capacity(2 * selections.count());
    let mut num_pushed: usize = 0;
    let primary_selection = selections.primary();
    let mut primary_selection_index = selections.primary_selection_index;
    for selection in &selections.inner{
        let surrounds = selection_impl(selection, buffer);
        if selection == primary_selection{
            primary_selection_index = num_pushed;
        }
        if surrounds.is_empty(){//push selection
            new_selections.push(selection.clone());
            num_pushed = num_pushed + 1;
        }
        else{//push surrounds
            for surround in surrounds{
                new_selections.push(surround);
                num_pushed = num_pushed + 1;
            }
        }
    }
    if new_selections.is_empty() || new_selections == selections.inner{Err(SelectionsError::ResultsInSameState)}
    else{
        //Ok(Selections::new(new_selections, primary_selection_index, text))
        Selections::new(new_selections, primary_selection_index, buffer, semantics.clone()).sort().merge_overlapping(buffer, semantics)
    }
}

//TODO: maybe this should be implemented with treesitter, so irrelevant pairs(like ' characters inside words(like don't)) aren't matched
//TODO: maybe front end should pass in their view of what is a valid surrounding pair, then we can match those...to make this as flexible as possible
//TODO: think about how surrounding quotation pairs should be handled
/// Returns a new pair of [`Selection`]s with each selection over the nearest surrounding grapheme pair, if possible
/// valid pairs:    //maybe add ':', '*'
/// { }
/// ( )
/// [ ]
/// < >
/// ' '
/// " "
#[must_use] fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer) -> Vec<Selection>{
    let mut rev_search_index = selection.range.start;
    'outer: loop{
        let current_char = buffer./*inner.*/char(rev_search_index);
        if is_opening_bracket(current_char){
            let opening_char = current_char;
            let closing_char = get_matching_closing_bracket(opening_char);
            let mut match_stack = Vec::new();
            let mut search_index = rev_search_index;
            'inner: loop{
                let current_char = buffer./*inner.*/char(search_index);
                if opening_char == closing_char{  //search before cursor for previous instance of char, then after cursor for next instance. ignore hierarchy because i'm not sure we can parse that...
                    if current_char == closing_char{
                        if match_stack.is_empty(){
                            match_stack.push(current_char);
                        }
                        else{
                            let mut first_selection = selection.clone();
                            first_selection.range.start = rev_search_index;
                            first_selection.range.end = buffer.next_grapheme_char_index(rev_search_index);
                            first_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;

                            let mut second_selection = selection.clone();
                            second_selection.range.start = search_index;
                            second_selection.range.end = buffer.next_grapheme_char_index(search_index);
                            second_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;
                            return vec![
                                //Selection::new(Range::new(rev_search_index, text_util::next_grapheme_index(rev_search_index, text)), Direction::Forward),
                                first_selection,
                                //Selection::new(Range::new(search_index, text_util::next_grapheme_index(search_index, text)), Direction::Forward)
                                second_selection
                            ];
                        }
                    }
                    else{/*do nothing. index will be incremented below...*/}
                }
                else{
                    if current_char == opening_char{
                        match_stack.push(current_char);
                    }
                    else if current_char == closing_char{
                        match_stack.pop();
                        if match_stack.is_empty(){
                            if search_index >= selection.range.start{
                                let mut first_selection = selection.clone();
                                first_selection.range.start = rev_search_index;
                                first_selection.range.end = buffer.next_grapheme_char_index(rev_search_index);
                                first_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;

                                let mut second_selection = selection.clone();
                                second_selection.range.start = search_index;
                                second_selection.range.end = buffer.next_grapheme_char_index(search_index);
                                second_selection.extension_direction = None;//crate::selection::ExtensionDirection::None;
                                return vec![
                                    //Selection::new(Range::new(rev_search_index, text_util::next_grapheme_index(rev_search_index, text)), Direction::Forward),
                                    first_selection,
                                    //Selection::new(Range::new(search_index, text_util::next_grapheme_index(search_index, text)), Direction::Forward)
                                    second_selection
                                ];
                            }
                            else{break 'inner;}
                        }
                        else{/*do nothing. index will be incremented below...*/}
                    }
                }
                    
                search_index = search_index + 1;

                if search_index >= buffer.len_chars(){break 'outer;}
            }
        }
        //else{ //is else really needed here?...
            rev_search_index = rev_search_index.saturating_sub(1);
        //}

        if rev_search_index == 0{break 'outer;}
    }

    Vec::new()
}
fn is_opening_bracket(char: char) -> bool{  //TODO: this should prob be in text_util.rs
    char == '{'
    || char == '('
    || char == '['
    || char == '<'
    || char == '\''
    || char == '"'
}
fn get_matching_closing_bracket(char: char) -> char{    //TODO: this should prob be in text_util.rs
    if char == '{'{'}'}
    else if char == '('{')'}
    else if char == '['{']'}
    else if char == '<'{'>'}
    else if char == '\''{'\''}
    else if char == '"'{'"'}
    else{panic!();} //TODO: maybe return None, or an error?...
}

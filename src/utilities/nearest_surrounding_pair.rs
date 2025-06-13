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

//TODO: for some reason, repeated calls after successfully selecting bracket pair do not return same state error...
fn selections_impl(selections: &Selections, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selections, SelectionsError>{
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
        let current_char = buffer.inner.char(rev_search_index);
        if is_opening_bracket(current_char){
            let opening_char = current_char;
            let closing_char = get_matching_closing_bracket(opening_char);
            let mut match_stack = Vec::new();
            let mut search_index = rev_search_index;
            'inner: loop{
                let current_char = buffer.inner.char(search_index);
                if opening_char == closing_char{  //search before cursor for previous instance of char, then after cursor for next instance. ignore hierarchy because i'm not sure we can parse that...
                    if current_char == closing_char{
                        if match_stack.is_empty(){
                            match_stack.push(current_char);
                        }
                        else{
                            let mut first_selection = selection.clone();
                            first_selection.range.start = rev_search_index;
                            first_selection.range.end = buffer.next_grapheme_boundary_index(rev_search_index);
                            first_selection.direction = crate::selection::ExtensionDirection::None;

                            let mut second_selection = selection.clone();
                            second_selection.range.start = search_index;
                            second_selection.range.end = buffer.next_grapheme_boundary_index(search_index);
                            second_selection.direction = crate::selection::ExtensionDirection::None;
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
                                first_selection.range.end = buffer.next_grapheme_boundary_index(rev_search_index);
                                first_selection.direction = crate::selection::ExtensionDirection::None;

                                let mut second_selection = selection.clone();
                                second_selection.range.start = search_index;
                                second_selection.range.end = buffer.next_grapheme_boundary_index(search_index);
                                second_selection.direction = crate::selection::ExtensionDirection::None;
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

#[cfg(test)]
mod tests{
    use crate::utilities::nearest_surrounding_pair;
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
        
        let result = nearest_surrounding_pair::application_impl(&mut app, semantics.clone());
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
        
        assert!(nearest_surrounding_pair::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_multiple_selections(){
        //                     1                   2
        // 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7
        //|i>d k ( s|o>m e|[>] _ t h|i>n g _ {|}>e l s e ) _ i|d>k
        //|i>d k|(>s o m e|[>]>_ t h i n g _|{>}>e l s e|)>_ i|d>k
        test(
            CursorSemantics::Block, 
            "idk(some[] thing {}else) idk", 
            vec![
                (0, 1, None),   //no pair
                (5, 6, None),   //pair
                (8, 9, None),   //pair
                (13, 14, None), //pair
                (18, 19, None), //pair
                (26, 27, None)  //no pair
            ], 0, 
            vec![
                (0, 1, None),
                (3, 4, Some(3)),    //idk why these have stored line position and others don't
                (8, 9, None),
                (9, 10, None),
                (17, 18, None),
                (18, 19, None),
                (23, 24, Some(23)), //idk why these have stored line position and others don't
                (26, 27, None)
                //TODO: merge overlapping in selection.rs causing the stored line position. only the overlapping selections have it
                //if so, this should def be fixed in merge_overlapping impl
                //or more correctly, every movement fn should update the stored line position...
                    //the only reason we have a None variant is so that we don't need to take a &Rope in Selection::new()
            ], 0
        );
    }

    ////|i>d k ( s o m e [ ] _ t h i n g _ { } e l s e ) _ i d k     //no surrounding pair with cursor at this location
    #[test] fn at_start_with_no_surrounding_pair(){
        test_error(
            CursorSemantics::Block, 
            "idk(some[] thing {}else) idk", 
            vec![
                (0, 1, None)
            ], 0
        );
    }

    //// i d k ( s|o>m e [ ] _ t h i n g _ { } e l s e ) _ i d k     //paren surrounding pair with cursor at this location
    #[test] fn normal_case(){
        test(
            CursorSemantics::Block, 
            "idk(some[] thing {}else) idk", 
            vec![
                (5, 6, None)
            ], 0, 
            vec![
                (3, 4, None),
                (23, 24, None)
            ], 0
        );
    }

    //// i d k ( s o m e|[>] _ t h i n g _ { } e l s e ) _ i d k     //square bracket surrounding pair with cursor at this location
    #[test] fn with_cursor_over_surrounding_pair_opening(){
        test(
            CursorSemantics::Block, 
            "idk(some[] thing {}else) idk", 
            vec![
                (8, 9, None)
            ], 0, 
            vec![
                (8, 9, None),
                (9, 10, None)
            ], 0
        );
    }

    //// i d k ( s o m e [ ] _ t h|i>n g _ { } e l s e ) _ i d k     //paren surrounding pair with cursor at this location
    #[test] fn with_other_pairs_inside_surrounding_pair(){
        test(
            CursorSemantics::Block, 
            "idk(some[] thing {}else) idk", 
            vec![
                (13, 14, None)
            ], 0, 
            vec![
                (3, 4, None),
                (23, 24, None)
            ], 0
        );
    }

    //// i d k ( s o m e [ ] _ t h i n g _ {|}>e l s e ) _ i d k     //curly bracket surrounding pair with cursor at this location
    #[test] fn with_cursor_over_surrounding_pair_closing(){
        test(
            CursorSemantics::Block, 
            "idk(some[] thing {}else) idk", 
            vec![
                (18, 19, None)
            ], 0, 
            vec![
                (17, 18, None),
                (18, 19, None)
            ], 0
        );
    }

    //// i d k ( s o m e [ ] _ t h i n g _ { } e l s e ) _ i|d>k     //no surrounding pair with cursor at this location
    #[test] fn at_end_with_no_surrounding_pair(){
        test_error(
            CursorSemantics::Block, 
            "idk(some[] thing {}else) idk", 
            vec![
                (26, 27, None)
            ], 0
        );
    }

    //These two seem redundant given previous tests...
    #[test] fn no_opening_bracket_pair_returns_empty_vec(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsomething)\n", 
            vec![
                (3, 4, None)
            ], 0
        );
    }
    #[test] fn no_closing_bracket_pair_returns_empty_vec(){
        test_error(
            CursorSemantics::Block, 
            "(idk\nsomething\n", 
            vec![
                (3, 4, None)
            ], 0
        );
    }

    ////idk(some()t(h(i)n)g()else)    //test from multiple levels of same surrounding pair
    #[test] fn with_multiple_levels_of_same_surrounding_pair(){
        test(
            CursorSemantics::Block, 
            "idk(some()t(h(i)n)g()else", 
            vec![
                (12, 13, None)
            ], 0, 
            vec![
                (11, 12, None),
                (17, 18, None)
            ], 0
        );
    }

    //TODO: impl test with expected quote pair behavior
    //note: quote pairs may have to work differently than bracket pairs
    //#[test] fn with_same_surrounding_pair_opening_and_closing(){
    //    //idk"some""t"h"i"n"g""else"
    //    let text = Rope::from("idk\"some\"\"t\"h\"i\"n\"g\"\"else");
    //    let selection = Selection::new(Range::new(12, 13), Direction::Forward);
    //    assert_eq!(
    //        vec![
    //            Selection::new(Range::new(11, 12), Direction::Forward),
    //            //Selection::new(Range::new(17, 18), Direction::Forward)
    //            Selection::new(Range::new(13, 14), Direction::Forward)
    //        ],
    //        selection.nearest_surrounding_pair(&text)
    //    );
    //}
}

use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, CursorSemantics},
    view::{View, ViewError},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    match view_impl(&app.view, app.selections.primary(), &app.buffer, semantics){
        Ok(view) => {app.view = view}
        Err(e) => {
            match e{
                ViewError::InvalidInput => {return Err(ApplicationError::InvalidInput);}
                ViewError::ResultsInSameState => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
            }
        }
    }

    Ok(())
}

/// Returns an instance of [`View`] vertically centered around specified cursor.
/// # Errors
///     //if function output would return a `View` with the same state.
/// # Panics
///     //if `selection` is invalid.
///     //if `text` is invalid.
fn view_impl(view: &View, selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<View, ViewError>{
    assert!(selection.cursor(buffer, semantics.clone()) <= buffer.len_chars());    //ensure selection is valid
    assert!(buffer.len_lines() > 0);  //ensure text is not empty
        
    let current_line = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    //let view_is_even_numbered = self.height % 2 == 0;
    let half_view_height = view.height / 2; //current impl will be biased towards the bottom of the view, if view is even numbered

    //TODO: consider how even numbered view heights should be handled...
    // maybe < half_view_height.saturating_sub(1)
    if current_line <= half_view_height{return Err(ViewError::ResultsInSameState);} //maybe return error cursor before doc_start + half the view height
    if current_line >= buffer.len_lines().saturating_sub(half_view_height){return Err(ViewError::ResultsInSameState);}    //maybe return error cursor after doc_end - half the view height

    // Calculate the new vertical start position
    let new_vertical_start = if current_line > half_view_height{
        current_line.saturating_sub(half_view_height)
    }else{
        0
    }.min(buffer.len_lines().saturating_sub(view.height));    //should self.height be half_view_height?

    // if view_is_even_numbered && (current_line == new_vertical_start || current_line == new_vertical_start.saturating_sub(1)){return Err(ViewError::ResultsInSameState);}
    //if current_line == new_vertical_start{return Err(ViewError::ResultsInSameState);}   //maybe return error already centered   //TODO: and test
    //

    let new_view = View::new(view.horizontal_start, new_vertical_start, view.width, view.height);    
    if new_view == view.clone(){return Err(ViewError::ResultsInSameState);} //can we catch this condition any earlier?...
    Ok(new_view)
}

#[cfg(test)]
mod tests{
    use crate::utilities::center_view_vertically_around_cursor;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::View,
    };

    fn test(semantics: CursorSemantics, text: &str, view: View, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, expected_text: &str, expected_view: View){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));

        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        app.view = view;
        
        let result = center_view_vertically_around_cursor::application_impl(&mut app, semantics.clone());
        assert!(!result.is_err());
        
        assert_eq!(expected_text, app.view.text(&app.buffer));
        assert_eq!(expected_view, app.view);
        assert!(!app.buffer.is_modified());
    }
    fn test_error(semantics: CursorSemantics, text: &str, view: View, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
        
        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        app.view = view;
        
        assert!(center_view_vertically_around_cursor::application_impl(&mut app, semantics).is_err());
        assert!(!app.buffer.is_modified());
    }
    
    #[test] fn works_when_cursor_in_valid_position_before_center(){
        // i d k                                        // i d k
        // y e t                                        //|y e t|
        //|s o m|e      //<-- primary cursor here -->   //|s o m|e
        //|m o r|e                                      //|m o r|e
        //|o t h|e r                                    // o t h e r
        // r a n d o m                                  // r a n d o m
        // s h i t                                      // s h i t
        test(
            CursorSemantics::Block, 
            "idk\nyet\nsome\nmore\nother\nrandom\nshit\n", 
            View::new(0, 2, 3, 3), 
            vec![
                (8, 9, None)
            ], 0, 
            "yet\nsom\nmor\n", 
            View::new(0, 1, 3, 3)
        );
    }
    #[test] fn works_when_cursor_in_valid_position_after_center(){
        // i d k                                        // i d k
        // y e t                                        // y e t
        //|s o m|e                                      // s o m e
        //|m o r|e                                      //|m o r|e
        //|o t h|e r    //<-- primary cursor here -->   //|o t h|e r
        // r a n d o m                                  //|r a n|d o m
        // s h i t                                      // s h i t
        test(
            CursorSemantics::Block, 
            "idk\nyet\nsome\nmore\nother\nrandom\nshit\n", 
            View::new(0, 2, 3, 3), 
            vec![
                (18, 19, None)
            ], 0, 
            "mor\noth\nran\n", 
            View::new(0, 3, 3, 3)
        );
    }

    #[test] fn errors_when_cursor_before_half_view_height(){
        //|i d k|       //<-- primary cursor here -->   //|i d k|
        //|s o m|e                                      //|s o m|e
        //|m o r|e                                      //|m o r|e
        // o t h e r                                    // o t h e r
        // s h i t                                      // s h i t
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nmore\nother\nshit\n", 
            View::new(0, 0, 3, 3), 
            vec![
                (0, 1, None)
            ], 0
        );
    }
    
    #[test] fn errors_when_cursor_after_doc_end_minus_half_view_height(){
        // i d k                                        // i d k
        // s o m e                                      // s o m e
        //|m o r|e                                      //|m o r|e
        //|o t h|e r                                    //|o t h|e r
        //|s h i|t      //<-- primary cursor here -->   //|s h i|t
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nmore\nother\nshit\n", 
            View::new(0, 2, 3, 3), 
            vec![
                (25, 26, None)
            ], 0
        );
    }
    
    #[test] fn errors_when_cursor_already_centered_with_odd_num_lines(){
        // i d k                                        // i d k
        //|s o m|e                                      //|s o m|e
        //|m o r|e      //<-- primary cursor here -->   //|m o r|e
        //|o t h|e r                                    //|o t h|e r
        // s h i t                                      // s h i t
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nmore\nother\nshit\n", 
            View::new(0, 1, 3, 3), 
            vec![
                (9, 10, None)
            ], 0
        );
    }
    #[test] fn errors_when_cursor_on_first_middle_line_with_even_num_lines(){
        // i d k                                        // i d k
        //|y e t|                                       //|y e t|
        //|s o m|e      //<-- primary cursor here -->   //|s o m|e
        //|m o r|e                                      //|m o r|e
        //|o t h|e r                                    //|o t h|e r
        // s h i t                                      // s h i t
        test_error(
            CursorSemantics::Block, 
            "idk\nyet\nsome\nmore\nother\nshit\n", 
            View::new(0, 1, 3, 4), 
            vec![
                (8, 9, None)
            ], 0
        );
    }
    #[test] fn errors_when_cursor_on_other_middle_line_with_even_num_lines(){
        // i d k                                        // i d k
        //|y e t|                                       //|y e t|
        //|s o m|e                                      //|s o m|e
        //|m o r|e      //<-- primary cursor here -->   //|m o r|e
        //|o t h|e r                                    //|o t h|e r
        // s h i t                                      // s h i t
        test_error(
            CursorSemantics::Block, 
            "idk\nyet\nsome\nmore\nother\nshit\n", 
            View::new(0, 1, 3, 4), 
            vec![
                (13, 14, None)
            ], 0
        );
    }
}

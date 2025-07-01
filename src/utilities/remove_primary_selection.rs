use crate::{
    application::{Application, ApplicationError},
    selections::{Selections, SelectionsError}
};

pub fn application_impl(app: &mut Application) -> Result<(), ApplicationError>{
    match selections_impl(&app.selections){
        Ok(new_selections) => {app.selections = new_selections;}
        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
    }
    Ok(())
}

/// Returns a new instance of [`Selections`] with the current primary selection removed, if possible.
/// # Errors
/// errors if `self` containts only a single `Selection`.
pub fn selections_impl(selections: &Selections) -> Result<Selections, SelectionsError>{
    if selections.count() < 2{return Err(SelectionsError::SingleSelection);}
        
    let mut new_selections = Vec::new();
    for selection in &selections.inner{
        if selection != selections.primary(){
            new_selections.push(selection.clone());
        }
    }
    //keep the new primary selection relatively close by
    let new_primary_index = if selections.primary_selection_index > 0{
        selections.primary_selection_index.saturating_sub(1)
    }else{
        selections.primary_selection_index
    };

    Ok(Selections{inner: new_selections, primary_selection_index: new_primary_index})
}

#[cfg(test)]
mod tests{
    use crate::utilities::remove_primary_selection;
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
        
        let result = remove_primary_selection::application_impl(&mut app);
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
        
        assert!(remove_primary_selection::application_impl(&mut app).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn when_primary_is_first_next_becomes_new_primary(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
            vec![
                (4, 5, None)
            ], 0
        );
    }
    #[test] fn when_primary_not_first_previous_becomes_new_primary(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 1, 
            vec![
                (0, 1, None)
            ], 0
        );
    }
    
    #[test] fn errors_if_single_selection(){
        test_error(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None)
            ], 0
        );
    }
}

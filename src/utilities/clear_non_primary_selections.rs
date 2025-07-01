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

/// Removes all [`Selection`]s except [`Selection`] at `primary_selection_index`.
/// Errors if [`Selections`] has only 1 [`Selection`].
pub fn selections_impl(selections: &Selections) -> Result<Selections, SelectionsError>{ //left this as public, because it is used elsewhere in codebase...
    if selections.count() < 2{return Err(SelectionsError::SingleSelection);}
    
    let primary_as_vec = vec![selections.primary().clone()];
    assert!(primary_as_vec.len() == 1);
    
    Ok(Selections{
        inner: primary_as_vec,
        primary_selection_index: 0
    })
}

#[cfg(test)]
mod tests{
    use crate::utilities::clear_non_primary_selections;
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
        
        let result = clear_non_primary_selections::application_impl(&mut app);
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
        
        assert!(clear_non_primary_selections::application_impl(&mut app).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn clears_non_primary_with_multiple_selections(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 
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

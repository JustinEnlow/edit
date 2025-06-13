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

/// Decrements the primary selection index.
fn selections_impl(selections: &Selections) -> Result<Selections, SelectionsError>{
    if selections.count() < 2{return Err(SelectionsError::SingleSelection);}
    if selections.primary_selection_index > 0{
        Ok(Selections{inner: selections.inner.clone(), primary_selection_index: selections.primary_selection_index - 1})
    }else{
        Ok(Selections{inner: selections.inner.clone(), primary_selection_index: selections.count().saturating_sub(1)})
    }
}

#[cfg(test)]
mod tests{
    use crate::utilities::decrement_primary_selection;
    use crate::{
        application::Application,
        selections::Selections,
        selection::{Selection, CursorSemantics},
        view::View,
    };

    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, expected_primary: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));

        let mut vec_selections = Vec::new();
        for tuple in tuple_selections{
            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
        }
        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
        
        app.selections = selections;
        
        let result = decrement_primary_selection::application_impl(&mut app);
        assert!(!result.is_err());
        
        assert_eq!(expected_primary, app.selections.primary_selection_index);
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
        
        assert!(decrement_primary_selection::application_impl(&mut app).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn with_multiple_selections(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 1, 0
        );
    }
    #[test] fn wraps_if_primary_is_first(){
        test(
            CursorSemantics::Block, 
            "idk\nsome\nshit\n", 
            vec![
                (0, 1, None),
                (4, 5, None)
            ], 0, 1
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

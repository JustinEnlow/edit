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

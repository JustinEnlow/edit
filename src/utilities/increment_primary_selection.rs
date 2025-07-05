use crate::{
    application::{Application, ApplicationError},
    selections::{Selections, SelectionsError}
};

//pub fn application_impl(app: &mut Application) -> Result<(), ApplicationError>{
//    match selections_impl(&app.selections){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
//    }
//    Ok(())
//}

/// Increments `primary_selection_index`.
pub fn selections_impl(selections: &Selections) -> Result<Selections, SelectionsError>{
    if selections.count() < 2{return Err(SelectionsError::SingleSelection);}
    if selections.primary_selection_index.saturating_add(1) < selections.count(){
        Ok(Selections{inner: selections.inner.clone(), primary_selection_index: selections.primary_selection_index + 1})
    }else{
        Ok(Selections{inner: selections.inner.clone(), primary_selection_index: 0})
    }
}

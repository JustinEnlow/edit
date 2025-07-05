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

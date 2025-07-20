use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::{Change, ChangeSet, Operation}
};

//TODO: can this function and backspace be combined?...

/// Deletes text inside each [`Selection`] in [`Selections`], or if [`Selection`] not extended, the next character, and pushes changes to undo stack.
pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();
    let mut cannot_delete = false;
    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        //handles cursor at doc end
        if selection.anchor() == app.buffer.len_chars() && selection.cursor(&app.buffer, semantics.clone()) == app.buffer.len_chars(){
            cannot_delete = true; //don't modify text buffer here...
            let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
            changes.push(change);
        }
        else{   //apply the delete
            //let change = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
            let change = app.buffer.apply_delete(selection, semantics.clone());
            if let Operation::Insert{inserted_text} = change.inverse(){
                app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
            }
            changes.push(change);
        }
    }

    if app.selections.count() == 1 && cannot_delete{return Err(ApplicationError::SelectionAtDocBounds);}
    else{
        // push change set to undo stack
        app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

        // clear redo stack. new actions invalidate the redo history
        app.redo_stack.clear();
    }

    Ok(())
}

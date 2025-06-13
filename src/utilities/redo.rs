use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::Operation
};
use std::cmp::Ordering;

/// Re-applies the last undone changes to the document.
// Make sure to clear the redo stack in every edit fn. new actions invalidate the redo history
pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    // Check if there is something to redo
    if let Some(change_set) = app.redo_stack.pop(){
        let changes = change_set.changes();

        app.selections = change_set.clone().selections_before_changes();    //set selections to selections_before_changes to account for any selection movements that may have occurred since undo
        assert!(app.selections.count() == changes.len());   //num selections should match num changes

        for (i, change) in changes.iter().enumerate().take(app.selections.count()){
            let selection = app.selections.nth_mut(i);
            match change.operation(){
                Operation::Insert{inserted_text} => {
                    let _ = Application::apply_insert(&mut app.buffer, &inserted_text, selection, semantics.clone());
                    app.selections.shift_subsequent_selections_forward(i, inserted_text.len());
                }
                Operation::Delete => {
                    *selection = change.selection_before_change();
                    let change = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
                    if let Operation::Insert{inserted_text} = change.inverse(){
                        app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
                    }
                }
                Operation::Replace{replacement_text} => {
                    let inserted_text = replacement_text;
                    let change = Application::apply_replace(&mut app.buffer, &inserted_text, selection, semantics.clone());
                    if let Operation::Replace{replacement_text} = change.inverse(){   //destructure to get currently selected text
                        match replacement_text.len().cmp(&inserted_text.len()){    //old selected text vs new text
                            Ordering::Greater => {app.selections.shift_subsequent_selections_backward(i, replacement_text.len().saturating_sub(inserted_text.len()));}
                            Ordering::Less => {app.selections.shift_subsequent_selections_forward(i, inserted_text.len().saturating_sub(replacement_text.len()));}
                            Ordering::Equal => {}   // no change to subsequent selections
                        }
                    }
                }
                Operation::NoOp => {}
            }
        }
        assert!(app.selections == change_set.clone().selections_after_changes());

        // Push changes back onto the undo stack
        app.undo_stack.push(change_set);

        Ok(())
    }else{Err(ApplicationError::NoChangesToRedo)}
}


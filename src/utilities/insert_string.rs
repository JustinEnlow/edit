use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::{Change, ChangeSet},
};

/// Inserts provided string into text at each selection.
pub fn application_impl(app: &mut Application, string: &str, use_hard_tab: bool, tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    if app.buffer.read_only{return Err(ApplicationError::ReadOnlyBuffer);}
    if string.is_empty(){return Err(ApplicationError::InvalidInput);}
    
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();

    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        let change = match string{
            //"\n" => {}    //handle behavior specific to pressing "enter". auto-indent, etc... //TODO: create tests for newline behavior...
            "\t" => {   //handle behavior specific to pressing "tab".
                if use_hard_tab{
                    if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), "\t")}
                    else{handle_insert(app, "\t", i, semantics.clone())}
                }
                else{
                    let tab_distance = app.buffer.distance_to_next_multiple_of_tab_width(selection, semantics.clone(), tab_width);
                    let modified_tab_width = if tab_distance > 0 && tab_distance < tab_width{tab_distance}else{tab_width};
                    let soft_tab = " ".repeat(modified_tab_width);

                    if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), &soft_tab)}
                    else{handle_insert(app, &soft_tab, i, semantics.clone())}
                }
            }
            //handle any other inserted string
            _ => {
                if selection.is_extended(){handle_insert_replace(app, i, semantics.clone(), string)}
                else{handle_insert(app, string, i, semantics.clone())}
            }
        };

        changes.push(change);
    }

    // push change set to undo stack
    app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

    // clear redo stack. new actions invalidate the redo history
    app.redo_stack.clear();

    Ok(())
}
fn handle_insert_replace(app: &mut Application, current_selection_index: usize, semantics: CursorSemantics, new_text: &str) -> Change{
    use std::cmp::Ordering;
    let selection = app.selections.nth_mut(current_selection_index);
    let change = Application::apply_replace(&mut app.buffer, new_text, selection, semantics);
    if let crate::history::Operation::Replace{replacement_text} = change.inverse(){
        match replacement_text.len().cmp(&new_text.len()){    //old selected text vs new text
            Ordering::Greater => {app.selections.shift_subsequent_selections_backward(current_selection_index, replacement_text.len().saturating_sub(new_text.len()));}
            Ordering::Less => {app.selections.shift_subsequent_selections_forward(current_selection_index, new_text.len().saturating_sub(replacement_text.len()));}
            Ordering::Equal => {}   // no change to subsequent selections
        }
    }
    change
}
fn handle_insert(app: &mut Application, string: &str, current_selection_index: usize, semantics: CursorSemantics) -> Change{
    let selection = app.selections.nth_mut(current_selection_index);
    let change = Application::apply_insert(&mut app.buffer, string, selection, semantics);
    app.selections.shift_subsequent_selections_forward(current_selection_index, string.len());
    change
}

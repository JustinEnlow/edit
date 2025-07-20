use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::{Change, ChangeSet, Operation}
};

//TODO: combine backspace with delete (make delete take a direction::Forward/Backward)

/// Deletes the previous character, or deletes selection if extended.
/// #### Invariants:
/// - will not delete past start of doc
/// - at start of line, appends current line to end of previous line
/// - removes previous soft tab, if `TAB_WIDTH` spaces are before cursor
/// - deletes selection if selection extended
pub fn application_impl(app: &mut Application, use_hard_tab: bool, tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::with_capacity(app.selections.count());
    let mut cannot_delete = false;

    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        if selection.is_extended(){
            //let change = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
            let change = app.buffer.apply_delete(selection, semantics.clone());
            if let Operation::Insert{inserted_text} = change.inverse(){
                app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
            }
            changes.push(change);
        }else{
            if selection.anchor() == 0 && selection.cursor(&app.buffer, semantics.clone()) == 0{
                cannot_delete = true; //don't modify text buffer here...
                let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
                changes.push(change);
            }
            else{
                let offset_from_line_start = app.buffer.offset_from_line_start(selection.cursor(&app.buffer, semantics.clone()));
                //let line = app.buffer.inner.line(app.buffer.char_to_line(selection.cursor(&app.buffer, semantics.clone())));
                let is_deletable_soft_tab = !use_hard_tab && offset_from_line_start >= tab_width
                // handles case where user adds a space after a tab, and wants to delete only the space
                && offset_from_line_start % tab_width == 0
                // if previous 4 chars are spaces, delete 4. otherwise, use default behavior
                && app.buffer.slice_is_all_spaces(offset_from_line_start.saturating_sub(tab_width), offset_from_line_start);

                if is_deletable_soft_tab{
                    selection.shift_and_extend(tab_width, &app.buffer, semantics.clone());
                    //changes.push(Application::apply_delete(&mut app.buffer, selection, semantics.clone()));
                    changes.push(app.buffer.apply_delete(selection, semantics.clone()));
                    app.selections.shift_subsequent_selections_backward(i, tab_width);
                }
                else{
                        //if let Ok(new_selection) = selection.move_left(&document.text, semantics){
                    if let Ok(new_selection) = crate::utilities::move_cursor_left::selection_impl(selection, 1, &app.buffer, None, semantics.clone()){
                        *selection = new_selection;
                    }   //TODO: handle error    //first for loop guarantees no selection is at doc bounds, so this should be ok to ignore...
                    //changes.push(Application::apply_delete(&mut app.buffer, selection, semantics.clone()));
                    changes.push(app.buffer.apply_delete(selection, semantics.clone()));
                    app.selections.shift_subsequent_selections_backward(i, 1);
                }
            }
        }
    }

    if app.selections.count() == 1 && cannot_delete{return Err(ApplicationError::SelectionAtDocBounds);}
    else{
        // push changes to undo stack
        app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));

        // clear redo stack. new actions invalidate the redo history
        app.redo_stack.clear();
    }

    Ok(())
}

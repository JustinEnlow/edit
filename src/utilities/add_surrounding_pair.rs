use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::{Change, ChangeSet, Operation}
};

//TODO: i think all edit actions + apply replace/insert/delete should prob be made purely functional...

//had to make the following public
    //Document.text
    //Document.selections
    //Document.undo_stack
    //Document.redo_stack
    //Document::apply_replace
//is this easing of encapsulation acceptable?...
pub fn application_impl(app: &mut Application, leading_char: char, trailing_char: char, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    let selections_before_changes = app.selections.clone();
    let mut changes = Vec::new();
    let mut cannot_add_surrounding_pair = false;  //to handle cursor at doc end...
    for i in 0..app.selections.count(){
        let selection = app.selections.nth_mut(i);
        //handles cursor at doc end
        if selection.anchor() == app.buffer.len_chars() && selection.cursor(&app.buffer, semantics.clone()) == app.buffer.len_chars(){
            cannot_add_surrounding_pair = true; //don't modify text buffer here...
            let change = Change::new(Operation::NoOp, selection.clone(), selection.clone(), Operation::NoOp);
            changes.push(change);
        }
        else{   //replace each selection with its text contents + leading and trailing char added
            //let mut contents = selection.contents_as_string(&document.text);
            let mut contents = selection.to_string(&app.buffer);
            contents.insert(0, leading_char);
            contents.push(trailing_char);
            let change = Application::apply_replace(&mut app.buffer, &contents, selection, CursorSemantics::Block);
            changes.push(change);
            app.selections.shift_subsequent_selections_forward(i, 2);  //TODO: could this be handled inside apply_replace and similar functions?...
        }
    }

    if app.selections.count() == 1 && cannot_add_surrounding_pair{return Err(ApplicationError::InvalidInput);}
    else{
        // push change set to undo stack
        app.undo_stack.push(ChangeSet::new(changes, selections_before_changes, app.selections.clone()));
    
        // clear redo stack. new actions invalidate the redo history
        app.redo_stack.clear();
    }
    
    Ok(())
}

use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
};

/// Cut single selection.
/// Copies text to clipboard and removes selected text from document.
/// Ensure single selection when calling this function.
pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    if app.selections.count() > 1{return Err(ApplicationError::SelectionsError(crate::selections::SelectionsError::MultipleSelections))}

    let selection = app.selections.primary_mut();
    // Copy the selected text to the clipboard
    app.clipboard = app.buffer.slice(selection.range.start, selection.range.end).to_string();
    crate::utilities::delete::application_impl(app, semantics)   //notice this is returning the result from delete
}

use crate::{
    selection::{Selection, SelectionError, CursorSemantics, Direction},
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] encompassing the current line.
//TODO: make pub fn select_line //should this include newline at end of line? //should this include indentation at start of line? //vscode includes both, as does kakoune
//TODO: if called on empty last line, this moves the selection to second to last line end, instead it should error
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{
    //selection.assert_invariants(buffer, semantics.clone());
    assert_eq!(Ok(()), selection.invariants_hold(buffer, semantics.clone()));
    //vs code selects all spanned lines...  maybe caller can make that determination...
    if selection.spans_multiple_lines(buffer){return Err(SelectionError::SpansMultipleLines);}    //make specific error. SpansMultipleLines or something...
    if buffer.char_to_line(selection.cursor(buffer, semantics.clone())) == buffer.len_lines().saturating_sub(1){return Err(SelectionError::ResultsInSameState);}

    let line = buffer.char_to_line(selection.range.start);
    let line_start = buffer.line_to_char(line);
    let line_end = line_start + buffer.line_width_chars(line, true);

    if selection.range.start == line_start && selection.range.end == line_end{Err(SelectionError::ResultsInSameState)}
    else{
        let mut selection = selection.clone();
        selection.range.start = line_start;
        selection.range.end = line_end;
        selection.extension_direction = Some(Direction::Forward);//crate::selection::ExtensionDirection::Forward;
        //TODO?: maybe update stored line offset?...
        Ok(selection)
    }
}

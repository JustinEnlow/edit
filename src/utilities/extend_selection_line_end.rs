use crate::{
    application::{Application, ApplicationError},
    selection::{Selection, SelectionError, CursorSemantics, Movement},
    selections::SelectionsError
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match app.selections.move_cursor_potentially_overlapping(&app.buffer, semantics, selection_impl){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(_) => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState))}
//    }
//    Ok(())
//}

/// Returns a new instance of [`Selection`] with the [`Selection`] extended to the end of the current line.
pub fn selection_impl(selection: &Selection, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selection, SelectionError>{    //TODO: ensure this can't extend past doc text end
    selection.assert_invariants(buffer, semantics.clone());
    let line_number = buffer.char_to_line(selection.cursor(buffer, semantics.clone()));
    let line_width = buffer.line_width(line_number, false);    //doesn't include newline
    let line_start = buffer.line_to_char(line_number);
    let line_end = line_start.saturating_add(line_width);   //index at end of line text, not including newline  //nth_next_grapheme_index(line_start, line_width, text)?

    match semantics{
        CursorSemantics::Bar => {
            if selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
            selection.put_cursor(line_end, buffer, Movement::Extend, semantics, true)
        }
        CursorSemantics::Block => {
            //if self.cursor(semantics) == line_end.saturating_sub(1)
            if selection.cursor(buffer, semantics.clone()) == buffer.previous_grapheme_boundary_index(line_end)
            || selection.cursor(buffer, semantics.clone()) == line_end{return Err(SelectionError::ResultsInSameState);}
            let start_line = buffer.char_to_line(selection.range.start);
            let end_line = buffer.char_to_line(selection.range.end);
            if selection.cursor(buffer, semantics.clone()) == selection.range.start && end_line > start_line{
                selection.put_cursor(line_end, buffer, Movement::Extend, semantics, true)  //put cursor over newline, if extending from a line below
            }else{
                //self.put_cursor(line_end.saturating_sub(1), text, Movement::Extend, semantics, true)
                selection.put_cursor(buffer.previous_grapheme_boundary_index(line_end), buffer, Movement::Extend, semantics, true)
            }
        }
    }
}

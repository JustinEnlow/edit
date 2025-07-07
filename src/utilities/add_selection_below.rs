use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    selections::{Selections, SelectionsError}
};

//pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
//    match selections_impl(&app.selections, &app.buffer, semantics){
//        Ok(new_selections) => {app.selections = new_selections;}
//        Err(e) => {return Err(ApplicationError::SelectionsError(e))}
//    }
//    Ok(())
//}

//TODO: add selection above/below fns don't work as expected when multiple selections on same line. only adds primary selection range above/below

// TODO: selection added below at text end is not rendering on last line(this is a frontend issue though)
/// Adds a new [`Selection`] directly below bottom-most [`Selection`], with the same start and end offsets from line start, if possible.
pub fn selections_impl(selections: &Selections, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selections, SelectionsError>{
    assert!(selections.count() > 0);  //ensure at least one selection in selections

    let bottom_selection = selections.last();
    let bottom_selection_line = buffer.char_to_line(bottom_selection.range.start);
    //bottom_selection_line must be zero based, and text.len_lines() one based...   //TODO: verify
    if bottom_selection_line >= buffer.len_lines().saturating_sub(1){return Err(SelectionsError::CannotAddSelectionBelow);}
    // should error if any selection spans multiple lines. //callee can determine appropriate response behavior in this case        //vscode behavior is to extend topmost selection down one line if any selection spans multiple lines
    for selection in &selections.inner{  //self.selections.iter(){   //change suggested by clippy lint
        if selection.spans_multiple_lines(buffer){return Err(SelectionsError::SpansMultipleLines);}
    }

    // using primary selection here, because that is the selection we want our added selection to emulate, if possible with the available text
    let start_offset = buffer.offset_from_line_start(selections.primary().range.start);
    let end_offset = start_offset.saturating_add(selections.primary().range.end.saturating_sub(selections.primary().range.start));  //start_offset + (end char index - start char index)
    let line_below = bottom_selection_line.saturating_add(1);
    let line_start = buffer.line_to_char(line_below);
    let line_text = buffer.inner.line(line_below);
    let line_width = buffer.line_width(line_below, false);
    let line_width_including_newline = buffer.line_width(line_below, true);
    let (start, end) = if line_text.to_string().is_empty() || line_text == "\n"{    //should be impossible for the text in the line above first selection to be empty. is_empty() check is redundant here...
        match semantics{
            CursorSemantics::Bar => (line_start, line_start),
            CursorSemantics::Block => (line_start, buffer.next_grapheme_boundary_index(line_start))
        }
    }
    else if selections.primary().is_extended(){
        if start_offset < line_width{   //should we exclusively handle start_offset < line_width && end_offset < line_width as well?
            (line_start.saturating_add(start_offset), line_start.saturating_add(end_offset.min(line_width_including_newline))) //start offset already verified within line text bounds
        }
        else{
            // currently same as non extended. this might change...
            match semantics{    //ensure adding the offsets doesn't make this go past line width
                CursorSemantics::Bar => (line_start.saturating_add(start_offset.min(line_width)), line_start.saturating_add(start_offset.min(line_width))),
                CursorSemantics::Block => (line_start.saturating_add(start_offset.min(line_width)), buffer.next_grapheme_boundary_index(line_start.saturating_add(start_offset.min(line_width))))
            }
        }
    }
    else{  //not extended
        match semantics{    //ensure adding the offsets doesn't make this go past line width
            CursorSemantics::Bar => (line_start.saturating_add(start_offset.min(line_width)), line_start.saturating_add(start_offset.min(line_width))),
            CursorSemantics::Block => (line_start.saturating_add(start_offset.min(line_width)), buffer.next_grapheme_boundary_index(line_start.saturating_add(start_offset.min(line_width))))
        }
    };

    //match selections.primary().direction{
    //    Direction::Forward => Ok(selections.push(Selection::new(Range::new(start, end), Direction::Forward), false)),
    //    Direction::Backward => Ok(selections.push(Selection::new(Range::new(start, end), Direction::Backward), false))
    //}
    let mut selection = selections.primary().clone();
    selection.range.start = start;
    selection.range.end = end;
    selection.extension_direction = selection.direction(buffer, semantics.clone());
    Ok(selections.push(selection, false))
}

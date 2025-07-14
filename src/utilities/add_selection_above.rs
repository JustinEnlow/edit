use crate::{
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

/// Adds a new [`Selection`] directly above the top-most [`Selection`], with the same start and end offsets from line start, if possible.
pub fn selections_impl(selections: &Selections, buffer: &crate::buffer::Buffer, semantics: CursorSemantics) -> Result<Selections, SelectionsError>{
    assert!(selections.count() > 0);  //ensure at least one selection in selections

    let top_selection = selections.first();
    let top_selection_line = buffer.char_to_line(top_selection.range.start);
    if top_selection_line == 0{return Err(SelectionsError::CannotAddSelectionAbove);}
    // should error if any selection spans multiple lines. //callee can determine appropriate response behavior in this case        //vscode behavior is to extend topmost selection up one line if any selection spans multiple lines
    for selection in &selections.inner{  //self.selections.iter(){   //change suggested by clippy lint
        if selection.spans_multiple_lines(buffer){return Err(SelectionsError::SpansMultipleLines);}
    }

    // using primary selection here, because that is the selection we want our added selection to emulate, if possible with the available text
    let start_offset = buffer.offset_from_line_start(selections.primary().range.start);
    let end_offset = start_offset.saturating_add(selections.primary().range.end.saturating_sub(selections.primary().range.start));  //start_offset + (end char index - start char index)
    let line_above = top_selection_line.saturating_sub(1);
    let line_start = buffer.line_to_char(line_above);
    let line_text = buffer./*inner.*/line(line_above);
    let line_width = buffer.line_width_chars(line_above, false);
    let line_width_including_newline = buffer.line_width_chars(line_above, true);
    let (start, end) = if line_text.to_string().is_empty() || line_text == "\n"{    //should be impossible for the text in the line above first selection to be empty. is_empty() check is redundant here...
        match semantics{
            CursorSemantics::Bar => (line_start, line_start),
            CursorSemantics::Block => (line_start, buffer.next_grapheme_char_index(line_start))
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
                CursorSemantics::Block => (line_start.saturating_add(start_offset.min(line_width)), buffer.next_grapheme_char_index(line_start.saturating_add(start_offset.min(line_width))))
            }
        }
    }
    else{  //not extended
        match semantics{    //ensure adding the offsets doesn't make this go past line width
            CursorSemantics::Bar => (line_start.saturating_add(start_offset.min(line_width)), line_start.saturating_add(start_offset.min(line_width))),
            CursorSemantics::Block => (line_start.saturating_add(start_offset.min(line_width)), buffer.next_grapheme_char_index(line_start.saturating_add(start_offset.min(line_width))))
        }
    };

    let mut selection = selections.primary().clone();
    selection.range.start = start;
    selection.range.end = end;
    Ok(selections.push_front(selection, false))
}

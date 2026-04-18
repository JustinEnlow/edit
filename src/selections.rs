use crate::{
    selection::{Selection, SelectionError, CursorSemantics},
    buffer::Buffer,
    display_area::DisplayArea,
};



#[derive(Debug, PartialEq)] pub enum SelectionsError{
    SingleSelection,
    MultipleSelections,
    SpansMultipleLines,
    CannotAddSelectionAbove,
    CannotAddSelectionBelow,
    NoSearchMatches,
    ResultsInSameState
}

#[derive(Clone, PartialEq, Debug)] pub struct Selections{
    pub leading: Vec<Selection>,    //TODO?: maybe VecDeque for performance     //or consider SmallVec
    pub primary: Selection,
    pub trailing: Vec<Selection>,   //TODO?: maybe VecDeque for performance     //or consider SmallVec
}
impl Selections{
    /// Returns new instance of [`Selections`] from provided input.
    #[must_use] pub fn new(selections: Vec<Selection>, primary_selection_index: usize, buffer: &Buffer, semantics: CursorSemantics) -> Self{
        assert!(!selections.is_empty());
        assert!(primary_selection_index < selections.len());
        Selections::from_flattened(selections, primary_selection_index).normalized(buffer, semantics)
    }

    fn from_flattened(mut selections: Vec<Selection>, primary_selection_index: usize) -> Self{
        assert!(primary_selection_index < selections.len());
        let primary = selections.remove(primary_selection_index);
        let (leading, trailing) = selections.split_at(primary_selection_index);
        Self{leading: leading.to_vec(), primary, trailing: trailing.to_vec()}
    }
    pub fn normalized(&self, buffer: &Buffer, semantics: CursorSemantics) -> Self{
        let mut selections = self.clone();
        //TODO: selections.grapheme_align();
        selections = selections.sort();
        if let Ok(merged_selections) = selections.merge_overlapping(buffer, semantics){
            selections = merged_selections;
        }
        //are these asserts needed anymore? i think the structure guarantees these now...
        //assert!(!selections.flatten().is_empty());
        //assert!(selections.primary_selection_index() < selections.flatten().len());
        selections
    }

    pub fn flatten(&self) -> Vec<Selection>{
        let mut selections = Vec::with_capacity(self.leading.len() + 1 + self.trailing.len());
        selections.extend(self.leading.iter().cloned());
        selections.push(self.primary.clone());
        selections.extend(self.trailing.iter().cloned());
        selections
    }
    //could be used for display of selections, when we want to handle primary separately
    pub fn flatten_non_primary(&self) -> Vec<Selection>{
        let mut selections = Vec::with_capacity(self.leading.len() + self.trailing.len());
        selections.extend(self.leading.iter().cloned());
        selections.extend(self.trailing.iter().cloned());
        selections
    }

    //TODO: should this go in buffer.rs instead? fn to_string_with_debug_selections
    #[cfg(test)] fn debug_over_buffer_content(&self, buffer: &Buffer, semantics: CursorSemantics) -> String{
        use unicode_segmentation::UnicodeSegmentation;

        let mut debug_string = String::new();
        for (i, grapheme) in buffer.to_string().graphemes(true).enumerate(){
            for selection in self.iter(){
                if selection.anchor() == i{
                    debug_string.push('|');
                }
                if semantics == CursorSemantics::Block && (selection.extension_direction == None || selection.extension_direction == Some(crate::selection::Direction::Forward)){
                    if selection.cursor(buffer, semantics.clone()) == i{
                        debug_string.push(':');
                    }
                }
                if selection.head() == i{
                    match selection.extension_direction{
                        None | Some(crate::selection::Direction::Forward) => {
                            debug_string.push('>');
                        }
                        Some(crate::selection::Direction::Backward) => {
                            debug_string.push('<');
                        }
                    }
                }
            }
            debug_string.push_str(grapheme);
        }
        debug_string
    }

    /// Returns the number of [`Selection`]s in [`Selections`].
    // note: not tested in selections_tests module
    #[must_use] pub fn count(&self) -> usize{
        self.leading.len() + 1 + self.trailing.len()
    }
    
    // note: not tested in selections_tests module
    pub fn iter(&self) -> impl Iterator<Item = &Selection>{
        self.leading.iter()
            .chain(std::iter::once(&self.primary))
            .chain(self.trailing.iter())
    }
    
    // note: not tested in selections_tests module
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Selection>{
        self.leading.iter_mut()
            .chain(std::iter::once(&mut self.primary))
            .chain(self.trailing.iter_mut())
    }
    
    /// Returns a new instance of [`Selections`] with the last element removed.
    #[must_use] pub fn pop(&self) -> Self{
        let mut new_selections = self.flatten();
        // Guarantee at least one selection
        if new_selections.len() > 1{new_selections.pop();}
        else{return self.clone();}

        // Is there a better way to determine new primary selection?
        let primary_selection_index = new_selections.len().saturating_sub(1);

        Selections::from_flattened(new_selections, primary_selection_index)
    }

    /// Prepends a [`Selection`] to the front of [Self], updating `primary_selection_index` if desired.
    #[must_use] pub fn push_front(&self, selection: Selection, update_primary: bool) -> Self{
        let mut new_selections = self.clone();
        if update_primary{
            let mut new_trailing = Vec::new();
            for i in self.iter(){
                new_trailing.push(i.clone());
            }
            new_selections.leading = Vec::new();
            new_selections.primary = selection;
            new_selections.trailing = new_trailing;
        }else{
            new_selections.leading.insert(0, selection);
        }
        new_selections
    }
    
    /// Appends a [`Selection`] to the back of [Self], updating `primary_selection_index` if desired.
    #[must_use] pub fn push(&self, selection: Selection, update_primary: bool) -> Self{
        let mut new_selections = self.clone();
        if update_primary{
            let mut new_leading = Vec::new();
            for i in self.iter(){
                new_leading.push(i.clone());
            }
            new_selections.leading = new_leading;
            new_selections.primary = selection;
            new_selections.trailing = Vec::new();
        }else{
            new_selections.trailing.push(selection);
        }
        new_selections
    }
    
    pub fn primary_selection_index(&self) -> usize{
        self.leading.len()
    }
    
    // note: not tested in selections_tests module
    #[must_use] pub fn first(&self) -> &Selection{
        if self.leading.is_empty(){
            &self.primary
        }else{
            // unwrapping because we ensure leading is not empty
            self.leading.first().unwrap()
        }
    }
    
    // note: not tested in selections_tests module
    #[must_use] pub fn last(&self) -> &Selection{
        if self.trailing.is_empty(){
            &self.primary
        }else{
            // unwrapping because we ensure trailing is not empty
            self.trailing.last().unwrap()
        }
    }
    
    //unwrapping instead of returning Option<> because, if the index is invalid,
    //there is obviously some error in logic or the program is in a bad state
    // note: not tested in selections_tests module
    pub fn nth_mut(&mut self, index: usize) -> &mut Selection{
        match index{
            i if i < self.leading.len() => self.leading.get_mut(i).unwrap(),
            i if i == self.leading.len() => &mut self.primary,
            i => self.trailing.get_mut(i - self.leading.len() - 1).unwrap() //index - leading and primary to get correct index for trailing
        }
    }

    /// Sorts each [`Selection`] in [Selections] by position.
    /// #### Invariants:
    /// - preserves primary selection through the sorting process
    #[must_use] pub fn sort(&self) -> Self{ //TODO: return error instead...
        if self.count() < 2{return self.clone();}

        let mut sorted_selections = self.flatten();
        sorted_selections.sort_unstable_by_key(Selection::start);
    
        let primary_selection_index = sorted_selections
            .iter()
            .position(|selection| selection == &self.primary)
            .unwrap_or(0);
    
        Selections::from_flattened(sorted_selections, primary_selection_index)
    }

    /// Merges overlapping [`Selection`]s.
    pub fn merge_overlapping(&self, buffer: &Buffer, semantics: CursorSemantics) -> Result<Self, SelectionsError>{
        if self.count() < 2{return Err(SelectionsError::SingleSelection);}

        let mut primary = self.primary.clone();
        let mut new_selections = self.flatten();
        new_selections.dedup_by(|current_selection, prev_selection|{
            //if prev_selection.overlaps(current_selection){
                //let merged_selection = match current_selection.merge(prev_selection, text, semantics){
                //    Ok(val) => val,
                //    Err(_) => {return false;}
                //};
                //let merged_selection = match current_selection.merge_overlapping(prev_selection, text, semantics){
                //    Ok(val) => val,
                //    Err(_) => {return false;}
                //};
                let Ok(merged_selection) = current_selection.merge_overlapping(prev_selection, buffer, semantics.clone()) //change suggested by clippy lint
                else{return false;};

                // Update primary selection to track index in next code block // Only clone if necessary
                if prev_selection == &primary || current_selection == &primary{
                    primary = merged_selection.clone();
                }

                *prev_selection = merged_selection;
                true
            //}else{false}
        });

        let primary_selection_index = new_selections.iter()
            .position(|selection| selection == &primary)
            .unwrap_or(0);

        assert!(self.count() > 0);

        Ok(Selections::from_flattened(new_selections, primary_selection_index))
    }

    // should these be made purely functional?  //for selection in selections{if selection <= current_selection_index{push selection to vec}}
    pub fn shift_subsequent_selections_forward(&mut self, current_selection_index: usize, amount: usize){
        for subsequent_selection_index in current_selection_index.saturating_add(1)..self.count(){
            let subsequent_selection = self.nth_mut(subsequent_selection_index);
            subsequent_selection.range.start = subsequent_selection.range.start.saturating_add(amount);
            subsequent_selection.range.end = subsequent_selection.range.end.saturating_add(amount);
        }
    }
    pub fn shift_subsequent_selections_backward(&mut self, current_selection_index: usize, amount: usize){
        for subsequent_selection_index in current_selection_index.saturating_add(1)..self.count(){
            let subsequent_selection = self.nth_mut(subsequent_selection_index);
            subsequent_selection.range.start = subsequent_selection.range.start.saturating_sub(amount);
            subsequent_selection.range.end = subsequent_selection.range.end.saturating_sub(amount);
        }
    }

                //TODO: pub fn search_whole_text

    /// Intended to ease the use of Selection functions, when used over multiple selections, where the returned selections could be overlapping.
    pub fn move_cursor_potentially_overlapping<F>(
        &self, 
        buffer: &Buffer, 
        semantics: CursorSemantics, 
        move_fn: F
    ) -> Result<Self, SelectionsError>
        where F: Fn(&Selection, &Buffer, CursorSemantics) -> Result<Selection, SelectionError>
    {
        let mut new_selections = Vec::with_capacity(self.count());  //the maximum size this vec should ever be is num selections in self
        for selection in self.iter(){
            match move_fn(selection, buffer, semantics.clone()){
                Ok(new_selection) => {new_selections.push(new_selection);}
                Err(e) => {
                    match e{
                        SelectionError::ResultsInSameState => {
                            if self.count() == 1{return Err(SelectionsError::ResultsInSameState)}
                            new_selections.push(selection.clone()); //retains selections with no change resulting from move_fn
                        }
                        //TODO: figure out what to do with other errors, if they can even happen...
                        //are we guaranteed by fn impls to never have these errors returned?
                        //what if user passes an unintended move_fn to this one?...
                        SelectionError::SpansMultipleLines => { //changed this when moving selection impls into utilities module
                            if self.count() == 1{return Err(SelectionsError::SpansMultipleLines)}
                            new_selections.push(selection.clone()); //retains selections with no change resulting from move_fn
                        }
                        SelectionError::DirectionMismatch |
                        SelectionError::NoOverlap => {unreachable!()}   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
                    }
                }
            }
        }
        let mut new_selections = Selections::new(new_selections, self.primary_selection_index(), buffer, semantics.clone());
        if let Ok(merged_selections) = new_selections.merge_overlapping(buffer, semantics.clone()){
            new_selections = merged_selections;
        }
        if &new_selections == self{return Err(SelectionsError::ResultsInSameState);}    //this should handle multicursor at doc end and another extend all the way right at text and, and no same state error
        Ok(new_selections)
    }
    
    /// Intended to ease the use of Selection functions, when used over multiple selections, where the returned selections should definitely not be overlapping.
    pub fn move_cursor_non_overlapping<F>(&self, buffer: &Buffer, semantics: CursorSemantics, move_fn: F) -> Result<Self, SelectionsError>
        where F: Fn(&Selection, &Buffer, CursorSemantics) -> Result<Selection, SelectionError>
    {
        let mut new_selections = Vec::with_capacity(self.count());  //the maximum size this vec should ever be is num selections in self
        let mut movement_succeeded = false;
        for selection in self.iter(){
            match move_fn(selection, buffer, semantics.clone()){
                Ok(new_selection) => {
                    new_selections.push(new_selection);
                    movement_succeeded = true;
                }
                Err(e) => {
                    match e{
                        SelectionError::ResultsInSameState => {new_selections.push(selection.clone());} //same state handled later in fn
                        //figure out what to do with other errors, if they can even happen...
                        SelectionError::DirectionMismatch |
                        SelectionError::SpansMultipleLines |//InvalidInput |
                        SelectionError::NoOverlap => {unreachable!()}   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
                    }
                }
            }
        }
        if !movement_succeeded{return Err(SelectionsError::ResultsInSameState)}
        let new_selections = Selections::new(new_selections, self.primary_selection_index(), buffer, semantics.clone());
        Ok(new_selections)
    }
    
    /// Intended to ease the use of Selection functions, when used over multiple selections, where movement should result in a single selection.
    pub fn move_cursor_clearing_non_primary<F>(&self, buffer: &Buffer, semantics: CursorSemantics, move_fn: F) -> Result<Self, SelectionsError>
    where
        F: Fn(&Selection, &Buffer, CursorSemantics) -> Result<Selection, SelectionError>
    {
        let mut new_selections = self.clone();
        //if let Ok(primary_only) = crate::utilities::clear_non_primary_selections::selections_impl(self){new_selections = primary_only;} //intentionally ignoring any errors
        if let Ok(primary_only) = clear_non_primary_selections(self){new_selections = primary_only;}    //intentionally ignoring any errors
        match move_fn(&new_selections.primary.clone(), buffer, semantics.clone()){
            Ok(new_selection) => {
                new_selections = Selections::new(vec![new_selection], 0, buffer, semantics);
            }
            Err(e) => {
                match e{
                    SelectionError::ResultsInSameState => {return Err(SelectionsError::ResultsInSameState);}
                    //figure out what to do with other errors, if they can even happen...
                    SelectionError::DirectionMismatch |
                    SelectionError::SpansMultipleLines |//InvalidInput |
                    SelectionError::NoOverlap => {unreachable!()}   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
                }
            }
        }
        Ok(new_selections)
    }
    
    // Intended to ease the use of Selection functions, when used over multiple selections, where the returned selections are moved by view height and could be overlapping.
    // TODO: take view related data(document_widget area) as input param, instead of storing a separate source of truth that needs synching with frontend
    //pub fn move_cursor_page<F>(
    //    &self, 
    //    buffer: &Buffer, 
    //    view: &crate::view::DisplayArea, 
    //    semantics: CursorSemantics, 
    //    move_fn: F
    //) -> Result<Self, SelectionsError>
    //    where F: Fn(
    //        &Selection, 
    //        &Buffer, 
    //        &crate::view::DisplayArea, 
    //        CursorSemantics
    //    ) -> Result<Selection, SelectionError>
    //{
    //    let mut new_selections = Vec::with_capacity(self.count());  //the maximum size this vec should ever be is num selections in self
    //    for selection in self.iter(){
    //        match move_fn(selection, buffer, view, semantics.clone()){
    //            Ok(new_selection) => {new_selections.push(new_selection);}
    //            Err(e) => {
    //                match e{
    //                    SelectionError::ResultsInSameState => {
    //                        if self.count() == 1{return Err(SelectionsError::ResultsInSameState)}
    //                        new_selections.push(selection.clone()); //retains selections with no change resulting from move_fn
    //                    }
    //                    //TODO: figure out what to do with other errors, if they can even happen...
    //                    //are we guaranteed by fn impls to never have these errors returned?
    //                    //what if user passes an unintended move_fn to this one?...
    //                    SelectionError::DirectionMismatch |
    //                    SelectionError::SpansMultipleLines |//InvalidInput |
    //                    SelectionError::NoOverlap => {
    //                        //unreachable!()
    //                        println!("{e:#?}");
    //                    }   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
    //                }
    //            }
    //        }
    //    }
    //    let mut new_selections = Selections::new(new_selections, self.primary_selection_index, buffer, semantics.clone());
    //    if let Ok(merged_selections) = new_selections.merge_overlapping(buffer, semantics.clone()){
    //        new_selections = merged_selections;
    //    }
    //    Ok(new_selections)
    //}

    //could this handle all movement functions?...
    //maybe take a clear_non_primary bool that, if true, could shortcut iteration
    pub fn move_selection<F>(&self, count: /*Option<*/usize/*>*/, buffer: &Buffer, display_area: Option<&DisplayArea>, semantics: CursorSemantics, move_fn: F) -> Result<Self, SelectionsError>
        where F: Fn(&Selection, /*Option<*/usize/*>*/, &Buffer, Option<&DisplayArea>, CursorSemantics) -> Result<Selection, SelectionError>
    {
        let mut new_selections = Vec::with_capacity(self.count());  //the maximum size this vec should ever be is num selections in self
        /*
        if clear_non_primary{
            if let Ok(primary_only) = crate::utilities::clear_non_primary_selections::selections_impl(self){
                selections = primary_only;
            }
        }
        */
        for selection in self.iter(){
            match move_fn(selection, count, buffer, display_area, semantics.clone()){
                Ok(new_selection) => {new_selections.push(new_selection);}
                Err(e) => {
                    match e{
                        SelectionError::ResultsInSameState => {
                            if self.count() == 1{return Err(SelectionsError::ResultsInSameState)}
                            new_selections.push(selection.clone()); //retains selections with no change resulting from move_fn
                        }
                        //TODO: figure out what to do with other errors, if they can even happen...
                        //are we guaranteed by fn impls to never have these errors returned?
                        //what if user passes an unintended move_fn to this one?...
                        SelectionError::SpansMultipleLines => { //changed this when moving selection impls into utilities module
                            if self.count() == 1{return Err(SelectionsError::SpansMultipleLines)}
                            new_selections.push(selection.clone()); //retains selections with no change resulting from move_fn
                        }
                        SelectionError::DirectionMismatch |
                        SelectionError::NoOverlap => {unreachable!()}   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
                    }
                }
            }
        }
        let mut new_selections = Selections::new(new_selections, self.primary_selection_index(), buffer, semantics.clone());
        if let Ok(merged_selections) = new_selections.merge_overlapping(buffer, semantics.clone()){
            new_selections = merged_selections;
        }
        if &new_selections == self{return Err(SelectionsError::ResultsInSameState);}    //this should handle multicursor at doc end and another extend all the way right at text and, and no same state error
        Ok(new_selections)
    }
}




pub fn surround(
    selections: &Selections, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selections, SelectionsError>{
    let mut new_selections = Vec::with_capacity(2 * selections.count());
    let mut num_pushed: usize = 0;
    let primary_selection = &selections.primary;
    let mut primary_selection_index = selections.primary_selection_index();
    for selection in &selections.flatten(){
        //let surrounds = selection_impl(selection, buffer);
        let surrounds = crate::selection::surround(selection, buffer);
        //if selection == primary_selection{
        //    primary_selection_index = num_pushed;//.saturating_sub(1);
        //}
        //for surround in surrounds{
        //    new_selections.push(surround);
        //    num_pushed = num_pushed + 1;
        //}
        if surrounds.is_empty(){    //needed to handle mixed valid and invalid selections
            if selections.count() == 1{return Err(SelectionsError::ResultsInSameState);}
            if selection == primary_selection{
                primary_selection_index = num_pushed;//.saturating_sub(1);
            }
            new_selections.push(selection.clone());
            num_pushed = num_pushed + 1;
        }
        else{
            if selection == primary_selection{
                primary_selection_index = num_pushed;//.saturating_sub(1);
            }
            for surround in surrounds{
                new_selections.push(surround);
                num_pushed = num_pushed + 1;
            }
        }
    }
    assert!(!new_selections.is_empty());
    //if new_selections.is_empty(){Err(SelectionsError::ResultsInSameState)} //TODO: create better error?...
    //else{
        Ok(Selections::new(new_selections, primary_selection_index, buffer, semantics))
    //}
}

//TODO: for some reason, repeated calls after successfully selecting bracket pair do not return same state error...
pub fn nearest_surrounding_pair(
    selections: &Selections, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selections, SelectionsError>{
    let mut new_selections = Vec::with_capacity(2 * selections.count());
    let mut num_pushed: usize = 0;
    let primary_selection = &selections.primary;
    let mut primary_selection_index = selections.primary_selection_index();
    for selection in &selections.flatten(){
        //let surrounds = selection_impl(selection, buffer);
        let surrounds = crate::selection::nearest_surrounding_pair(selection, buffer);
        if selection == primary_selection{
            primary_selection_index = num_pushed;
        }
        if surrounds.is_empty(){//push selection
            new_selections.push(selection.clone());
            num_pushed = num_pushed + 1;
        }
        else{//push surrounds
            for surround in surrounds{
                new_selections.push(surround);
                num_pushed = num_pushed + 1;
            }
        }
    }
    if new_selections.is_empty() || new_selections == selections.flatten(){Err(SelectionsError::ResultsInSameState)}
    else{
        //Ok(Selections::new(new_selections, primary_selection_index, text))
        Selections::new(new_selections, primary_selection_index, buffer, semantics.clone()).sort().merge_overlapping(buffer, semantics)
    }
}

/// Removes all [`Selection`]s except [`Selection`] at `primary_selection_index`.
/// Errors if [`Selections`] has only 1 [`Selection`].
pub fn clear_non_primary_selections(selections: &Selections) -> Result<Selections, SelectionsError>{ //left this as public, because it is used elsewhere in codebase...
    if selections.count() < 2{return Err(SelectionsError::SingleSelection);}
    
    //let primary_as_vec = vec![selections.primary().clone()];
    //assert!(primary_as_vec.len() == 1);
    let primary = selections.primary.clone();
    
    //Ok(Selections{
    //    inner: primary_as_vec,
    //    primary_selection_index: 0
    //})
    Ok(Selections{
        leading: Vec::new(),
        primary,
        trailing: Vec::new()
    })
}

//TODO: add selection above/below fns don't work as expected when multiple selections on same line. only adds primary selection range above/below
/// Adds a new [`Selection`] directly above the top-most [`Selection`], with the same start and end offsets from line start, if possible.
pub fn add_selection_above(
    selections: &Selections, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selections, SelectionsError>{
    assert!(selections.count() > 0);  //ensure at least one selection in selections

    let top_selection = selections.first();
    let top_selection_line = buffer.char_to_line(top_selection.range.start);
    if top_selection_line == 0{return Err(SelectionsError::CannotAddSelectionAbove);}
    // should error if any selection spans multiple lines. //callee can determine appropriate response behavior in this case        //vscode behavior is to extend topmost selection up one line if any selection spans multiple lines
    for selection in &selections.flatten(){  //self.selections.iter(){   //change suggested by clippy lint
        if selection.spans_multiple_lines(buffer){return Err(SelectionsError::SpansMultipleLines);}
    }

    // using primary selection here, because that is the selection we want our added selection to emulate, if possible with the available text
    let start_offset = buffer.offset_from_line_start(selections.primary.range.start);
    let end_offset = start_offset.saturating_add(selections.primary.range.end.saturating_sub(selections.primary.range.start));  //start_offset + (end char index - start char index)
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
    else if selections.primary.is_extended(){
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

    let mut selection = selections.primary.clone();
    selection.range.start = start;
    selection.range.end = end;
    Ok(selections.push_front(selection, false))
}

//TODO: add selection above/below fns don't work as expected when multiple selections on same line. only adds primary selection range above/below
// TODO: selection added below at text end is not rendering on last line(this is a frontend issue though)
/// Adds a new [`Selection`] directly below bottom-most [`Selection`], with the same start and end offsets from line start, if possible.
pub fn add_selection_below(
    selections: &Selections, 
    buffer: &Buffer, 
    semantics: CursorSemantics
) -> Result<Selections, SelectionsError>{
    assert!(selections.count() > 0);  //ensure at least one selection in selections

    let bottom_selection = selections.last();
    let bottom_selection_line = buffer.char_to_line(bottom_selection.range.start);
    //bottom_selection_line must be zero based, and text.len_lines() one based...   //TODO: verify
    if bottom_selection_line >= buffer.len_lines().saturating_sub(1){return Err(SelectionsError::CannotAddSelectionBelow);}
    // should error if any selection spans multiple lines. //callee can determine appropriate response behavior in this case        //vscode behavior is to extend topmost selection down one line if any selection spans multiple lines
    for selection in &selections.flatten(){  //self.selections.iter(){   //change suggested by clippy lint
        if selection.spans_multiple_lines(buffer){return Err(SelectionsError::SpansMultipleLines);}
    }

    // using primary selection here, because that is the selection we want our added selection to emulate, if possible with the available text
    let start_offset = buffer.offset_from_line_start(selections.primary.range.start);
    let end_offset = start_offset.saturating_add(selections.primary.range.end.saturating_sub(selections.primary.range.start));  //start_offset + (end char index - start char index)
    let line_below = bottom_selection_line.saturating_add(1);
    let line_start = buffer.line_to_char(line_below);
    let line_text = buffer./*inner.*/line(line_below);
    let line_width = buffer.line_width_chars(line_below, false);
    let line_width_including_newline = buffer.line_width_chars(line_below, true);
    let (start, end) = if line_text.to_string().is_empty() || line_text == "\n"{    //should be impossible for the text in the line above first selection to be empty. is_empty() check is redundant here...
        match semantics{
            CursorSemantics::Bar => (line_start, line_start),
            CursorSemantics::Block => (line_start, buffer.next_grapheme_char_index(line_start))
        }
    }
    else if selections.primary.is_extended(){
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

    //match selections.primary().direction{
    //    Direction::Forward => Ok(selections.push(Selection::new(Range::new(start, end), Direction::Forward), false)),
    //    Direction::Backward => Ok(selections.push(Selection::new(Range::new(start, end), Direction::Backward), false))
    //}
    let mut selection = selections.primary.clone();
    selection.range.start = start;
    selection.range.end = end;
    selection.extension_direction = selection.direction(buffer, semantics.clone());
    Ok(selections.push(selection, false))
}

/// Returns a new instance of [`Selections`] with the current primary selection removed, if possible.
/// # Errors
/// errors if `self` containts only a single `Selection`.
pub fn remove_primary_selection(selections: &Selections) -> Result<Selections, SelectionsError>{
    if selections.count() < 2{return Err(SelectionsError::SingleSelection);}
        
    let mut new_selections = Vec::new();
    for selection in &selections.flatten(){
        if selection != &selections.primary{
            new_selections.push(selection.clone());
        }
    }
    //keep the new primary selection relatively close by
    let new_primary_index = if selections.primary_selection_index() > 0{
        selections.primary_selection_index().saturating_sub(1)
    }else{
        selections.primary_selection_index()
    };

    //Ok(Selections{inner: new_selections, primary_selection_index: new_primary_index})
    let mut leading = Vec::new();
    let mut trailing = Vec::new();

    for (i, sel) in new_selections.iter().enumerate(){
        match std::cmp::Ord::cmp(&i, &new_primary_index){
            std::cmp::Ordering::Less => {
                leading.push(sel.clone());
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                trailing.push(sel.clone());
            }
        }
    }

    Ok(Selections{
        leading,
        primary: new_selections[new_primary_index].clone(),
        trailing
    })
}

/// Increments `primary_selection_index`.
pub fn increment_primary_selection(selections: &Selections) -> Result<Selections, SelectionsError>{
    if selections.count() < 2{return Err(SelectionsError::SingleSelection);}
    //if selections.primary_selection_index().saturating_add(1) < selections.count(){
    //    Ok(Selections{inner: selections.inner.clone(), primary_selection_index: selections.primary_selection_index + 1})
    //}else{
    //    Ok(Selections{inner: selections.inner.clone(), primary_selection_index: 0})
    //}
    if let Some(next) = selections.trailing.first().cloned(){
        let mut  leading = selections.leading.clone();  //leading       //primary       //trailing
        let mut trailing = selections.trailing.clone(); //[0] [1]           [2]           [3] [4]
        
        leading.push(selections.primary.clone());                       //[0] [1] [2]       [2]           [3] [4]
        trailing.remove(0);                                       //[0] [1] [2]       [2]           [4]
        Ok(Selections{leading, primary: next, trailing})                //[0] [1] [2]       [3]           [4]
    }else{
                                                                        //leading           //primary       //trailing
        let mut leading = selections.leading.clone();   //[0] [1] [2] [3]       [4]
        leading.push(selections.primary.clone());                       //[0] [1] [2] [3] [4]   [4]
        let trailing = leading.split_off(1);        //[0]                   [4]         [1] [2] [3] [4]
        let primary = leading[0].clone();                    //[0]                   [0]         [1] [2] [3] [4]
        Ok(Selections{leading: Vec::new(), primary, trailing})          //                      [0]         [1] [2] [3] [4]
    }
}

/// Decrements the primary selection index.
pub fn decrement_primary_selection(selections: &Selections) -> Result<Selections, SelectionsError>{
    if selections.count() < 2{return Err(SelectionsError::SingleSelection);}
    //if selections.primary_selection_index() > 0{
    //    Ok(Selections{inner: selections.inner.clone(), primary_selection_index: selections.primary_selection_index - 1})
    //}else{
    //    Ok(Selections{inner: selections.inner.clone(), primary_selection_index: selections.count().saturating_sub(1)})
    //}
    if let Some(previous) = selections.leading.last().cloned(){
        let mut  leading = selections.leading.clone();  //leading       //primary       //trailing
        let mut trailing = selections.trailing.clone(); //[0] [1]           [2]           [3] [4]
        
        trailing.insert(0, selections.primary.clone());  //[0] [1]           [2]           [2] [3] [4]
        leading.pop();                                                  //[0]               [2]           [2] [3] [4]
        Ok(Selections{leading, primary: previous, trailing})            //[0]               [1]           [2] [3] [4]
    }else{
                                                                        //leading               //primary       //trailing
                                                                        //                      [0]             [1] [2] [3] [4]
        let mut leading = selections.trailing.clone();  //[1] [2] [3] [4]       [0]
        leading.insert(0, selections.primary.clone());   //[0] [1] [2] [3] [4]   [0]
        let primary = leading.pop().unwrap();                //[0] [1] [2] [3]       [0]
        Ok(Selections{leading, primary, trailing: Vec::new()})          //[0] [1] [2] [3]       [4]
    }
}





#[cfg(test)]
mod tests{
    use crate::{
        range::Range,
        selection::{Selection, CursorSemantics, Direction},
        selections::Selections,
        buffer::Buffer
    };

    #[test] fn non_extended_bar_semantics(){
        let semantics = CursorSemantics::Bar;
        let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
        let selection = Selection::new_from_range(Range::new(0, 0), None, &buffer, semantics.clone());
        let selections = Selections::new(vec![selection], 0, &buffer, semantics.clone());
        assert_eq!("|>idk\nsome\nshit\n", selections.debug_over_buffer_content(&buffer, semantics));
    }
    #[test] fn forward_extended_bar_semantics(){
        let semantics = CursorSemantics::Bar;
        let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
        let selection = Selection::new_from_range(Range::new(2, 6), Some(Direction::Forward), &buffer, semantics.clone());
        let selections = Selections::new(vec![selection], 0, &buffer, semantics.clone());
        assert_eq!("id|k\nso>me\nshit\n", selections.debug_over_buffer_content(&buffer, semantics));
    }
    #[test] fn backward_extended_bar_semantics(){
        let semantics = CursorSemantics::Bar;
        let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
        let selection = Selection::new_from_range(Range::new(2, 6), Some(Direction::Backward), &buffer, semantics.clone());
        let selections = Selections::new(vec![selection], 0, &buffer, semantics.clone());
        assert_eq!("id<k\nso|me\nshit\n", selections.debug_over_buffer_content(&buffer, semantics));
    }
    #[test] fn selections_with_all_extension_directions_bar_semantics(){
        let semantics = CursorSemantics::Bar;
        let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
        let selection_1 = Selection::new_from_range(Range::new(0, 4), Some(Direction::Forward), &buffer, semantics.clone());
        let selection_2 = Selection::new_from_range(Range::new(6, 6), None, &buffer, semantics.clone());
        let selection_3 = Selection::new_from_range(Range::new(8, 12), Some(Direction::Backward), &buffer, semantics.clone());
        let selections = Selections::new(vec![selection_1, selection_2, selection_3], 0, &buffer, semantics.clone());
        assert_eq!("|idk\n>so|>me<\nshi|t\n", selections.debug_over_buffer_content(&buffer, semantics));
    }
    
    #[test] fn non_extended_block_semantics(){
        let semantics = CursorSemantics::Block;
        let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
        let selection = Selection::new_from_range(Range::new(0, 1), None, &buffer, semantics.clone());
        let selections = Selections::new(vec![selection], 0, &buffer, semantics.clone());
        assert_eq!("|:i>dk\nsome\nshit\n", selections.debug_over_buffer_content(&buffer, semantics));
    }
    #[test] fn forward_extended_block_semantics(){
        let semantics = CursorSemantics::Block;
        let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
        let selection = Selection::new_from_range(Range::new(2, 6), Some(Direction::Forward), &buffer, semantics.clone());
        let selections = Selections::new(vec![selection], 0, &buffer, semantics.clone());
        assert_eq!("id|k\ns:o>me\nshit\n", selections.debug_over_buffer_content(&buffer, semantics));
    }
    #[test] fn backward_extended_block_semantics(){
        let semantics = CursorSemantics::Block;
        let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
        let selection = Selection::new_from_range(Range::new(2, 6), Some(Direction::Backward), &buffer, semantics.clone());
        let selections = Selections::new(vec![selection], 0, &buffer, semantics.clone());
        assert_eq!("id<k\nso|me\nshit\n", selections.debug_over_buffer_content(&buffer, semantics));
    }
    #[test] fn selections_with_all_extension_directions_block_semantics(){
        let semantics = CursorSemantics::Block;
        let buffer = Buffer::new("idk\nsome\nshit\n", None, false);
        let selection_1 = Selection::new_from_range(Range::new(0, 4), Some(Direction::Forward), &buffer, semantics.clone());
        let selection_2 = Selection::new_from_range(Range::new(6, 7), None, &buffer, semantics.clone());
        let selection_3 = Selection::new_from_range(Range::new(8, 12), Some(Direction::Backward), &buffer, semantics.clone());
        let selections = Selections::new(vec![selection_1, selection_2, selection_3], 0, &buffer, semantics.clone());
        assert_eq!("|idk:\n>so|:m>e<\nshi|t\n", selections.debug_over_buffer_content(&buffer, semantics));
    }
}

#[derive(Debug, PartialEq)]
pub enum SelectionsError{
    SingleSelection,
    MultipleSelections,
    SpansMultipleLines,
    CannotAddSelectionAbove,
    CannotAddSelectionBelow,
    NoSearchMatches,
    ResultsInSameState
}

#[derive(Clone, PartialEq, Debug)]
pub struct Selections{
    pub inner: Vec<crate::selection::Selection>,
    pub primary_selection_index: usize,
}
impl Selections{
    /// Returns new instance of [`Selections`] from provided input.
    #[must_use] pub fn new(
        selections: Vec<crate::selection::Selection>, 
        primary_selection_index: usize, 
        buffer: &crate::buffer::Buffer, 
        semantics: crate::selection::CursorSemantics
    ) -> Self{
        assert!(!selections.is_empty());
        assert!(primary_selection_index < selections.len());

        let mut instance = Self{
            inner: selections,
            primary_selection_index,
        };

        //TODO: instance.grapheme_align();
        instance = instance.sort();
        if let Ok(merged_selections) = instance.merge_overlapping(buffer, semantics){
            instance = merged_selections;
        }

        assert!(!instance.inner.is_empty());
        assert!(instance.primary_selection_index < instance.inner.len());

        instance
    }

    /// Returns the number of [`Selection`]s in [`Selections`].
    // note: not tested in selections_tests module
    #[must_use] pub fn count(&self) -> usize{
        self.inner.len()
    }
    
    // note: not tested in selections_tests module
    pub fn iter(&self) -> std::slice::Iter<'_, crate::selection::Selection>{
        self.inner.iter()
    }
    
    // note: not tested in selections_tests module
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, crate::selection::Selection>{
        self.inner.iter_mut()
    }
    
    /// Returns a new instance of [`Selections`] with the last element removed.
    #[must_use] pub fn pop(&self) -> Self{
        let mut new_selections = self.inner.clone();
        // Guarantee at least one selection
        if new_selections.len() > 1{new_selections.pop();}
        else{return self.clone();}

        // Is there a better way to determine new primary selection?
        let primary_selection_index = new_selections.len().saturating_sub(1);

        Self{
            inner: new_selections,
            primary_selection_index
        }
    }

    /// Prepends a [`Selection`] to the front of [Self], updating `primary_selection_index` if desired.
    #[must_use] pub fn push_front(&self, selection: crate::selection::Selection, update_primary: bool) -> Self{
        let mut new_selections = self.inner.clone();
        new_selections.insert(0, selection);
        Self{
            inner: new_selections,
            primary_selection_index: if update_primary{0}else{self.primary_selection_index.saturating_add(1)} //0
        }
    }
    
    /// Appends a [`Selection`] to the back of [Self], updating `primary_selection_index` if desired.
    #[must_use] pub fn push(&self, selection: crate::selection::Selection, update_primary: bool) -> Self{
        let mut new_selections = self.inner.clone();
        new_selections.push(selection);
        let primary_selection_index = new_selections.len().saturating_sub(1);
        Self{
            inner: new_selections,
            primary_selection_index: if update_primary{primary_selection_index}else{self.primary_selection_index}
        }
    }
    
    /// Returns a reference to the [`Selection`] at `primary_selection_index`.
    // note: not tested in selections_tests module
    #[must_use] pub fn primary(&self) -> &crate::selection::Selection{
        &self.inner[self.primary_selection_index]
    }
    /// Returns a mutable reference to the [`Selection`] at `primary_selection_index`.
    // note: not tested in selections_tests module
    pub fn primary_mut(&mut self) -> &mut crate::selection::Selection{
        &mut self.inner[self.primary_selection_index]
    }
    
    // note: not tested in selections_tests module
    #[must_use] pub fn first(&self) -> &crate::selection::Selection{
        // unwrapping because we ensure at least one selection is always present
        self.inner.first().unwrap()
    }
    //pub fn first_mut(&mut self) -> &mut Selection{
    //    self.selections.first_mut().unwrap()
    //}
    
    // note: not tested in selections_tests module
    #[must_use] pub fn last(&self) -> &crate::selection::Selection{
        // unwrapping because we ensure at least one selection is always present
        self.inner.last().unwrap()
    }
    
    // note: not tested in selections_tests module
    pub fn nth_mut(&mut self, index: usize) -> &mut crate::selection::Selection{
        self.inner.get_mut(index).unwrap()
    }

    /// Sorts each [`Selection`] in [Selections] by position.
    /// #### Invariants:
    /// - preserves primary selection through the sorting process
    #[must_use] pub fn sort(&self) -> Self{ //TODO: return error instead...
        if self.count() < 2{return self.clone();}

        let primary = self.primary().clone();
        let mut sorted_selections = self.inner.clone();
        sorted_selections.sort_unstable_by_key(crate::selection::Selection::start);
    
        let primary_selection_index = sorted_selections
            .iter()
            .position(|selection| selection == &primary)
            .unwrap_or(0);
    
        Self{
            inner: sorted_selections,
            primary_selection_index,
        }
    }

    /// Merges overlapping [`Selection`]s.
    pub fn merge_overlapping(&self, buffer: &crate::buffer::Buffer, semantics: crate::selection::CursorSemantics) -> Result<Self, SelectionsError>{
        if self.count() < 2{return Err(SelectionsError::SingleSelection);}

        let mut primary = self.primary().clone();
        let mut new_selections = self.inner.clone();
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

        Ok(Self{
            inner: new_selections,
            primary_selection_index,
        })
    }

    // should these be made purely functional?  //for selection in selections{if selection <= current_selection_index{push selection to vec}}
    pub fn shift_subsequent_selections_forward(&mut self, current_selection_index: usize, amount: usize){
        for subsequent_selection_index in current_selection_index.saturating_add(1)..self.count(){
            let subsequent_selection = self.nth_mut(subsequent_selection_index);
            //*subsequent_selection = crate::selection::Selection::new(
            //    crate::range::Range::new(
            //        subsequent_selection.anchor().saturating_add(amount), 
            //        subsequent_selection.head().saturating_add(amount)
            //    ), 
            //    Direction::Forward
            //);   //TODO: figure out how to actually determine direction
            subsequent_selection.range.start = subsequent_selection.range.start.saturating_add(amount);
            subsequent_selection.range.end = subsequent_selection.range.end.saturating_add(amount);
        }
    }
    pub fn shift_subsequent_selections_backward(&mut self, current_selection_index: usize, amount: usize){
        for subsequent_selection_index in current_selection_index.saturating_add(1)..self.count(){
            let subsequent_selection = self.nth_mut(subsequent_selection_index);
            //*subsequent_selection = crate::selection::Selection::new(
            //    crate::range::Range::new(
            //            subsequent_selection.anchor().saturating_sub(amount), 
            //            subsequent_selection.head().saturating_sub(amount)
            //        ), 
            //    Direction::Forward
            //);   //TODO: figure out how to actually determine direction
            subsequent_selection.range.start = subsequent_selection.range.start.saturating_sub(amount);
            subsequent_selection.range.end = subsequent_selection.range.end.saturating_sub(amount);
        }
    }

                //TODO: pub fn search_whole_text

    /// Intended to ease the use of Selection functions, when used over multiple selections, where the returned selections could be overlapping.
    pub fn move_cursor_potentially_overlapping<F>(
        &self, 
        buffer: &crate::buffer::Buffer, 
        semantics: crate::selection::CursorSemantics, 
        move_fn: F
    ) -> Result<Self, SelectionsError>
        where F: Fn(&crate::selection::Selection, &crate::buffer::Buffer, crate::selection::CursorSemantics) -> Result<crate::selection::Selection, crate::selection::SelectionError>
    {
        let mut new_selections = Vec::with_capacity(self.count());  //the maximum size this vec should ever be is num selections in self
        for selection in self.iter(){
            match move_fn(selection, buffer, semantics.clone()){
                Ok(new_selection) => {new_selections.push(new_selection);}
                Err(e) => {
                    match e{
                        crate::selection::SelectionError::ResultsInSameState => {
                            if self.count() == 1{return Err(SelectionsError::ResultsInSameState)}
                            new_selections.push(selection.clone()); //retains selections with no change resulting from move_fn
                        }
                        //TODO: figure out what to do with other errors, if they can even happen...
                        //are we guaranteed by fn impls to never have these errors returned?
                        //what if user passes an unintended move_fn to this one?...
                        crate::selection::SelectionError::SpansMultipleLines => { //changed this when moving selection impls into utilities module
                            if self.count() == 1{return Err(SelectionsError::SpansMultipleLines)}
                            new_selections.push(selection.clone()); //retains selections with no change resulting from move_fn
                        }
                        crate::selection::SelectionError::DirectionMismatch |
                        crate::selection::SelectionError::NoOverlap => {unreachable!()}   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
                    }
                }
            }
        }
        let mut new_selections = Selections::new(new_selections, self.primary_selection_index, buffer, semantics.clone());
        if let Ok(merged_selections) = new_selections.merge_overlapping(buffer, semantics.clone()){
            new_selections = merged_selections;
        }
        if &new_selections == self{return Err(SelectionsError::ResultsInSameState);}    //this should handle multicursor at doc end and another extend all the way right at text and, and no same state error
        Ok(new_selections)
    }
    
    /// Intended to ease the use of Selection functions, when used over multiple selections, where the returned selections should definitely not be overlapping.
    pub fn move_cursor_non_overlapping<F>(&self, buffer: &crate::buffer::Buffer, semantics: crate::selection::CursorSemantics, move_fn: F) -> Result<Self, SelectionsError>
        where F: Fn(&crate::selection::Selection, &crate::buffer::Buffer, crate::selection::CursorSemantics) -> Result<crate::selection::Selection, crate::selection::SelectionError>
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
                        crate::selection::SelectionError::ResultsInSameState => {new_selections.push(selection.clone());} //same state handled later in fn
                        //figure out what to do with other errors, if they can even happen...
                        crate::selection::SelectionError::DirectionMismatch |
                        crate::selection::SelectionError::SpansMultipleLines |//InvalidInput |
                        crate::selection::SelectionError::NoOverlap => {unreachable!()}   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
                    }
                }
            }
        }
        if !movement_succeeded{return Err(SelectionsError::ResultsInSameState)}
        let new_selections = Selections::new(new_selections, self.primary_selection_index, buffer, semantics.clone());
        Ok(new_selections)
    }
    
    /// Intended to ease the use of Selection functions, when used over multiple selections, where movement should result in a single selection.
    pub fn move_cursor_clearing_non_primary<F>(&self, buffer: &crate::buffer::Buffer, semantics: crate::selection::CursorSemantics, move_fn: F) -> Result<Self, SelectionsError>
    where
        F: Fn(&crate::selection::Selection, &crate::buffer::Buffer, crate::selection::CursorSemantics) -> Result<crate::selection::Selection, crate::selection::SelectionError>
    {
        let mut new_selections = self.clone();
        //if let Ok(primary_only) = self.clear_non_primary_selections(){new_selections = primary_only;}   //intentionally ignoring any errors
        if let Ok(primary_only) = crate::utilities::clear_non_primary_selections::selections_impl(self){new_selections = primary_only;}
        match move_fn(&new_selections.primary().clone(), buffer, semantics.clone()){
            Ok(new_selection) => {
                new_selections = Selections::new(vec![new_selection], 0, buffer, semantics);
            }
            Err(e) => {
                match e{
                    crate::selection::SelectionError::ResultsInSameState => {return Err(SelectionsError::ResultsInSameState);}
                    //figure out what to do with other errors, if they can even happen...
                    crate::selection::SelectionError::DirectionMismatch |
                    crate::selection::SelectionError::SpansMultipleLines |//InvalidInput |
                    crate::selection::SelectionError::NoOverlap => {unreachable!()}   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
                }
            }
        }
        Ok(new_selections)
    }
    
    /// Intended to ease the use of Selection functions, when used over multiple selections, where the returned selections are moved by view height and could be overlapping.
    pub fn move_cursor_page<F>(
        &self, 
        buffer: &crate::buffer::Buffer, 
        view: &crate::view::View, 
        semantics: crate::selection::CursorSemantics, 
        move_fn: F
    ) -> Result<Self, SelectionsError>
        where F: Fn(
            &crate::selection::Selection, 
            &crate::buffer::Buffer, 
            &crate::view::View, 
            crate::selection::CursorSemantics
        ) -> Result<crate::selection::Selection, crate::selection::SelectionError>
    {
        let mut new_selections = Vec::with_capacity(self.count());  //the maximum size this vec should ever be is num selections in self
        for selection in self.iter(){
            match move_fn(selection, buffer, view, semantics.clone()){
                Ok(new_selection) => {new_selections.push(new_selection);}
                Err(e) => {
                    match e{
                        crate::selection::SelectionError::ResultsInSameState => {
                            if self.count() == 1{return Err(SelectionsError::ResultsInSameState)}
                            new_selections.push(selection.clone()); //retains selections with no change resulting from move_fn
                        }
                        //TODO: figure out what to do with other errors, if they can even happen...
                        //are we guaranteed by fn impls to never have these errors returned?
                        //what if user passes an unintended move_fn to this one?...
                        crate::selection::SelectionError::DirectionMismatch |
                        crate::selection::SelectionError::SpansMultipleLines |//InvalidInput |
                        crate::selection::SelectionError::NoOverlap => {
                            //unreachable!()
                            println!("{e:#?}");
                        }   //if this is reached, move_fn called on one of the selections has probably put us in an unintended state. prob best to panic
                    }
                }
            }
        }
        let mut new_selections = Selections::new(new_selections, self.primary_selection_index, buffer, semantics.clone());
        if let Ok(merged_selections) = new_selections.merge_overlapping(buffer, semantics.clone()){
            new_selections = merged_selections;
        }
        Ok(new_selections)
    }
}

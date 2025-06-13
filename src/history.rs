use crate::selection::Selection;
use crate::selections::Selections;



#[derive(Clone, Debug, PartialEq)]
pub enum Operation{
    Insert{inserted_text: String},  //should this be Insert(String), so that when destructuring, the variable name can be assigned to make its intended more clear?
    Delete,
    Replace{replacement_text: String},  //should this be Replace(String), so that when destructuring, the variable name can be assigned to make its intended use more clear?
    ///useful for edits where one selection cannot perform the action, but still need a "change" 
    ///operation associated with it, so that vec of changes and vec of selections are always same len
    NoOp,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Change{
    operation: Operation,
    selection_before_change: Selection, //these are selections with positions offset by any previous change applied
    selection_after_change: Selection,  //these are selections with positions offset by any previous change applied
    inverse_operation: Operation,
}
impl Change{
    #[must_use]
    pub fn new(operation: Operation, old_selection: Selection, new_selection: Selection, inverse_operation: Operation) -> Self{
        Self{
            operation,
            selection_before_change: old_selection,
            selection_after_change: new_selection,
            inverse_operation,
        }
    }
    #[must_use]
    pub fn operation(&self) -> Operation{
        self.operation.clone()
    }
    //pub fn selection_after_change(&self) -> Selection{    //this doesn't appear to be needed just yet...
    //    self.selection_after_change.clone()
    //}
    #[must_use]
    pub fn selection_before_change(&self) -> Selection{
        self.selection_before_change.clone()
    }
    #[must_use]
    pub fn inverse(&self) -> Operation{
        self.inverse_operation.clone()
    }
}

/// `ChangeSet` holds a vec of Changes that should coinside with the vec of Selection in Selections(so the change at changes[0], should be associated with the selection at selections[0])
#[derive(Clone, Debug, PartialEq)]
pub struct ChangeSet{
    changes: Vec<Change>,
    selections_before_changes: Selections, //this could be selections without positions offset by any previous change applied     // could need this for certain things to work. ex. Backspace
    selections_after_changes: Selections,  //this is prob the same as selection_after_change from Change
}
impl ChangeSet{
    #[must_use]
    pub fn new(changes: Vec<Change>, selections_before_changes: Selections, selections_after_changes: Selections) -> Self{
        Self{changes, selections_before_changes, selections_after_changes}
    }
    #[must_use]
    pub fn changes(&self) -> Vec<Change>{
        self.changes.clone()
    }
    #[must_use]
    pub fn selections_before_changes(&self) -> Selections{
        self.selections_before_changes.clone()
    }
    #[must_use]
    pub fn selections_after_changes(self) -> Selections{
        self.selections_after_changes.clone()
    }
}

use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
    history::Operation
};
use std::cmp::Ordering;

/// Reverts the last set of changes made to the document.
pub fn application_impl(app: &mut Application, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    // Check if there is something to undo
    if let Some(change_set) = app.undo_stack.pop(){
        let changes = change_set.changes();
        
        app.selections = change_set.clone().selections_after_changes();    //set selections to selections_after_changes to account for any selection movements that may have occurred since edit
        assert!(app.selections.count() == changes.len());

        for (i, change) in changes.iter().enumerate().take(app.selections.count()){
            let selection = app.selections.nth_mut(i);
            match change.operation(){
                Operation::Insert{inserted_text} => {
                    selection.shift_and_extend(inserted_text.len(), &app.buffer, semantics.clone());
                    let _ = Application::apply_delete(&mut app.buffer, selection, semantics.clone());
                    app.selections.shift_subsequent_selections_backward(i, inserted_text.len());
                }
                Operation::Delete => {
                    if let Operation::Insert{inserted_text} = change.inverse(){
                        let _ = Application::apply_insert(&mut app.buffer, &inserted_text, selection, semantics.clone());   //apply inverse operation
                        app.selections.shift_subsequent_selections_forward(i, inserted_text.len());
                    }
                }
                Operation::Replace{replacement_text} => {
                    let inserted_text = replacement_text;
                    if let Operation::Replace{replacement_text} = change.inverse(){
                        selection.shift_and_extend(inserted_text.len(), &app.buffer, semantics.clone());
                        let _ = Application::apply_replace(&mut app.buffer, &replacement_text, selection, semantics.clone());
                        match inserted_text.len().cmp(&replacement_text.len()){    //old selected text vs new text
                            Ordering::Greater => {app.selections.shift_subsequent_selections_backward(i, inserted_text.len().saturating_sub(replacement_text.len()));}
                            Ordering::Less => {app.selections.shift_subsequent_selections_forward(i, replacement_text.len().saturating_sub(inserted_text.len()));}
                            Ordering::Equal => {}   // no change to subsequent selections
                        }
                    }
                }
                Operation::NoOp => {}
            }
        }
        // selections should be the same as they were before changes were made, because we are restoring that previous state
        app.selections = change_set.selections_before_changes();

        // Push inverted changes onto redo stack
        app.redo_stack.push(change_set);

        Ok(())
    }else{Err(ApplicationError::NoChangesToUndo)}
}

//#[cfg(test)]
//mod tests{
//    use crate::utilities::undo;
//    use crate::{
//        application::Application,
//        selections::Selections,
//        selection::{Selection, CursorSemantics, ExtensionDirection},
//        view::View,
//        history::{Change, ChangeSet, Operation},
//        range::Range,
//        buffer::Buffer
//    };
//
//    fn test(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, undo_stack: Vec<ChangeSet>, expected_text: &str, tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, expected_primary: usize){
//        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
//
//        let expected_buffer = crate::buffer::Buffer::new(expected_text, None, false);
//        let mut vec_expected_selections = Vec::new();
//        for tuple in tuple_expected_selections{
//            vec_expected_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &expected_buffer, semantics.clone()));
//        }
//        let expected_selections = Selections::new(vec_expected_selections, expected_primary, &expected_buffer, semantics.clone());
//        
//        let mut vec_selections = Vec::new();
//        for tuple in tuple_selections{
//            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
//        
//        app.selections = selections;
//        app.undo_stack = undo_stack;
//        
//        let result = undo::application_impl(&mut app, semantics);
//        assert!(!result.is_err());
//        
//        assert_eq!(expected_buffer, app.buffer);
//        assert_eq!(expected_selections, app.selections);
//        //println!("expected: {:?}\ngot: {:?}", expected_buffer, app.buffer);
//        //assert!(app.buffer.is_modified());    //is modified doesn't work with tests, because it now checks against a persistent file, which tests don't have
//    }
//    fn test_error(semantics: CursorSemantics, text: &str, tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, undo_stack: Vec<ChangeSet>){
//        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
//        
//        let mut vec_selections = Vec::new();
//        for tuple in tuple_selections{
//            vec_selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, &app.buffer, semantics.clone()));
//        }
//        let selections = Selections::new(vec_selections, primary, &app.buffer, semantics.clone());
//        
//        app.selections = selections;
//        app.undo_stack = undo_stack;
//        
//        assert!(undo::application_impl(&mut app, semantics).is_err());
//        assert!(!app.buffer.is_modified());
//    }
//
//    #[test] fn with_insert_change_on_stack(){
//        test(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (9, 10, None)
//            ], 0, 
//            vec![
//                //TODO: figure out how to move this changeset setup into the test fn...
//                ChangeSet::new(
//                    vec![
//                        Change::new(
//                            Operation::Insert{inserted_text: "some\n".to_string()}, 
//                            Selection::new(Range::new(4, 5), ExtensionDirection::Forward), 
//                            Selection::new(Range::new(9, 10), ExtensionDirection::Forward), 
//                            Operation::Delete
//                        )
//                    ], 
//                    Selections::new(
//                        vec![
//                            Selection::new(
//                                Range::new(4, 5), 
//                                ExtensionDirection::Forward
//                            )
//                        ], 
//                        0, 
//                        //&Rope::from("idk\nshit\n"), 
//                        &Buffer::new("idk\nshit\n", None, false),
//                        CursorSemantics::Block
//                    ), 
//                    Selections::new(
//                        vec![
//                            Selection::new(
//                                Range::new(9, 10), 
//                                ExtensionDirection::Forward
//                            )
//                        ], 
//                        0, 
//                        //&Rope::from("idk\nsome\nshit\n"), 
//                        &Buffer::new("idk\nsome\nshit\n", None, false),
//                        CursorSemantics::Block
//                    )
//                )
//            ], 
//            "idk\nshit\n", 
//            //"idk\nshit\n", 
//            vec![
//                (4, 5, None)
//            ], 0
//        );
//    }
//
//    //TODO: test with delete_change_on_stack
//    //TODO: test with replace change on stack
//    //TODO: test with no_op change on stack
//
//    //TODO: test with multiple selections/changes
//
//    #[test] fn undo_with_nothing_on_stack_errors(){
//        //test_error(CursorSemantics::Block);
//        test_error(
//            CursorSemantics::Block, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 1, None)
//            ], 
//            0, 
//            Vec::new()
//        );
//        //test_error(CursorSemantics::Bar);
//        test_error(
//            CursorSemantics::Bar, 
//            "idk\nsome\nshit\n", 
//            vec![
//                (0, 0, None)
//            ], 
//            0, 
//            Vec::new()
//        );
//    }
//}

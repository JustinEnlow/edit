use crate::{
    application::{Application, ApplicationError},
    selections::Selections,
    selection::{Selection, CursorSemantics},
    display_area::DisplayArea,
    buffer::Buffer,
};



//TODO?: could call app.handle_whatever_mode_keypress instead of calling f. this would allow us to test our reaction code as well...
//TODO: could take a view as arg, and verify that cursor movement moves the view correctly as well
//pub fn selection_movement_with_count<F>(
//    f: F, 
//    semantics: CursorSemantics, 
//    text: &str, 
//    tuple_selections: Vec<(usize, usize, Option<usize>)>, 
//    primary: usize, 
//    count: usize, 
//    display_area: Option<&DisplayArea>, 
//    tuple_expected_selections: Vec<(usize, usize, Option<usize>)>, 
//    expected_primary: usize
//)
//    where F: Fn(&mut Application, usize, Option<&DisplayArea>, CursorSemantics) -> Result<(), ApplicationError>
//{
//    let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
//    let expected_selections = generate_selections(tuple_expected_selections, expected_primary, &app.buffer, semantics.clone());
//    let selections = generate_selections(tuple_selections, primary, &app.buffer, semantics.clone());
//    app.selections = selections;
//
//    //TODO: this only tests our application_impl, and not any response code to it...
//    let result = f(&mut app, count, display_area, semantics.clone());
//    assert!(!result.is_err());
//    
//    assert_eq!(expected_selections, app.selections);
//    assert!(!app.buffer.is_modified());
//}

//could this be used for tests that should panic as well?...
//pub fn error_selection_movement_with_count<F>(
//    f: F, 
//    semantics: CursorSemantics, 
//    text: &str, 
//    tuple_selections: Vec<(usize, usize, Option<usize>)>, 
//    primary: usize, 
//    count: usize, 
//    display_area: Option<&DisplayArea>
//)
//    where F: Fn(&mut Application, usize, Option<&DisplayArea>, CursorSemantics) -> Result<(), ApplicationError>
//{
//    let mut app = Application::new_test_app(text, None, false, &DisplayArea::new(0, 0, 80, 200));
//    let selections = generate_selections(tuple_selections, primary, &app.buffer, semantics.clone());
//    app.selections = selections;
//    
//    assert!(f(&mut app, count, display_area, semantics).is_err());
//    assert!(!app.buffer.is_modified());
//}

fn generate_selections(tuple_selections: Vec<(usize, usize, Option<usize>)>, primary: usize, buffer: &Buffer, semantics: CursorSemantics) -> Selections{
    let mut selections = Vec::new();
    for tuple in tuple_selections{
        selections.push(Selection::new_from_components(tuple.0, tuple.1, tuple.2, buffer, semantics.clone()));
    }
    Selections::new(selections, primary, buffer, semantics.clone())
}

use crate::{
    application::{Application, ApplicationError},
    view::{View, ViewError},
    selections::SelectionsError
};

pub fn application_impl(app: &mut Application, amount: usize) -> Result<(), ApplicationError>{
    match view_impl(&app.view, amount){
        Ok(view) => {app.view = view}
        Err(e) => {
            match e{
                ViewError::InvalidInput => {return Err(ApplicationError::InvalidInput);}
                ViewError::ResultsInSameState => {return Err(ApplicationError::SelectionsError(SelectionsError::ResultsInSameState));}
            }
        }
    }

    Ok(())
}

/// Returns a new instance of [`View`] with `horizontal_start` decreased by specified amount.
/// # Errors
///     //if `amount` is 0.
///     //if function would return a `View` with the same state.
fn view_impl(view: &View, amount: usize) -> Result<View, ViewError>{
    if amount == 0{return Err(ViewError::InvalidInput);}
    if view.horizontal_start == 0{return Err(ViewError::ResultsInSameState);}
    Ok(View::new(view.horizontal_start.saturating_sub(amount), view.vertical_start, view.width, view.height))
}

#[cfg(test)]
mod tests{
    use crate::utilities::scroll_view_left;
    use crate::{
        application::Application,
        selection::CursorSemantics,
        view::View,
    };

    fn test(_semantics: CursorSemantics, text: &str, view: View, amount: usize, expected_text: &str, expected_view: View){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));

        app.view = view;
        
        let result = scroll_view_left::application_impl(&mut app, amount);
        assert!(!result.is_err());
        
        assert_eq!(expected_text, app.view.text(&app.buffer));
        assert_eq!(expected_view, app.view);
        assert!(!app.buffer.is_modified());
    }
    fn test_error(_semantics: CursorSemantics, text: &str, view: View, amount: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
        
        app.view = view;
        
        assert!(scroll_view_left::application_impl(&mut app, amount).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn scroll_left(){
        test(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            View::new(1, 0, 2, 2), 1, 
            "id\nso\n", 
            View::new(0, 0, 2, 2), 
        );
    }
    //TODO: test when amount > space left to scroll.    //does this saturate at doc bounds currently?

    #[test] fn errors_if_already_scrolled_left_all_the_way(){
        test_error(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            View::new(0, 0, 2, 2), 1, 
        );
    }

    #[test] fn errors_if_amount_is_zero(){
        test_error(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            View::new(1, 0, 2, 2), 0, 
        );
    }
}

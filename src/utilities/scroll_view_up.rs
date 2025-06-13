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

/// Returns a new instance of [`View`] with `vertical_start` decreased by specified amount.
/// # Errors
///     //if `amount` is 0.
///     //if function would return a `View` with the same state.
fn view_impl(view: &View, amount: usize) -> Result<View, ViewError>{
    if amount == 0{return Err(ViewError::InvalidInput);}
    if view.vertical_start == 0{return Err(ViewError::ResultsInSameState);}
    Ok(View::new(view.horizontal_start, view.vertical_start.saturating_sub(amount), view.width, view.height))
}

#[cfg(test)]
mod tests{
    use crate::utilities::scroll_view_up;
    use crate::{
        application::Application,
        selection::CursorSemantics,
        view::View,
    };

    fn test(_semantics: CursorSemantics, text: &str, view: View, amount: usize, expected_text: &str, expected_view: View){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));

        app.view = view;
        
        let result = scroll_view_up::application_impl(&mut app, amount);
        assert!(!result.is_err());
        
        assert_eq!(expected_text, app.view.text(&app.buffer));
        assert_eq!(expected_view, app.view);
        assert!(!app.buffer.is_modified());
    }
    fn test_error(_semantics: CursorSemantics, text: &str, view: View, amount: usize){
        let mut app = Application::new_test_app(text, None, false, &View::new(0, 0, 80, 200));
        
        app.view = view;
        
        assert!(scroll_view_up::application_impl(&mut app, amount).is_err());
        assert!(!app.buffer.is_modified());
    }

    #[test] fn scroll_up(){
        test(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            View::new(0, 2, 2, 2), 1, 
            "so\nsh\n", 
            View::new(0, 1, 2, 2), 
        );

        test(
            CursorSemantics::Bar,
            "idk\nsome\nshit\n", 
            View::new(0, 2, 2, 2), 1, 
            "so\nsh\n", 
            View::new(0, 1, 2, 2), 
        );
    }
    //TODO: test when amount > space left to scroll.    //does this saturate at doc bounds currently?

    #[test] fn errors_if_already_scrolled_up_all_the_way(){
        test_error(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            View::new(0, 0, 2, 2), 1, 
        );
        test_error(
            CursorSemantics::Bar,
            "idk\nsome\nshit\n", 
            View::new(0, 0, 2, 2), 1, 
        );
    }

    #[test] fn errors_if_amount_is_zero(){
        test_error(
            CursorSemantics::Block,
            "idk\nsome\nshit\n", 
            View::new(0, 1, 2, 2), 0, 
        );
        test_error(
            CursorSemantics::Bar,
            "idk\nsome\nshit\n", 
            View::new(0, 1, 2, 2), 0, 
        );
    }
}

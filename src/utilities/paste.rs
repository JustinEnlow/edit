use crate::{
    application::{Application, ApplicationError},
    selection::CursorSemantics,
};

/// Insert clipboard contents at cursor position(s).
pub fn application_impl(app: &mut Application, use_hard_tab: bool, tab_width: usize, semantics: CursorSemantics) -> Result<(), ApplicationError>{
    crate::utilities::insert_string::application_impl(app, &app.clipboard.clone(), use_hard_tab, tab_width, semantics)
}

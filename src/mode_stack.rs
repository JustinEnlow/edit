use crate::mode::Mode;

/// Guarantees at least one element on stack
pub struct ModeStack{
    stack: Vec<Mode>,
    message_stack: Vec<Option<String>>,    //assert message_stack.len() == stack.len()
    top: Mode,
    top_message: Option<String>,
}
impl ModeStack{
    pub fn push(&mut self, new_top: Mode, new_top_message: Option<String>){
        self.stack.push(self.top.clone());
        self.message_stack.push(self.top_message.clone());
        self.top = new_top;
        self.top_message = new_top_message;
    }
    pub fn pop(&mut self) -> Result<(Mode, Option<String>), String>{
        match (self.stack.pop(), self.message_stack.pop()){
            (Some(new_top), Some(new_top_message)) => {
                let old_top = self.top.clone();
                let old_top_message = self.top_message.clone();
                //
                if matches!(old_top, Mode::Error | Mode::Warning | Mode::Notify | Mode::Info){
                    assert!(old_top_message.is_some());
                }else{
                    assert!(old_top_message.is_none());
                }
                //
                self.top = new_top;
                self.top_message = new_top_message;
                Ok((old_top, old_top_message))
            }
            _ => Err("Cannot pop mode stack with single mode".to_string())
        }
    }
    pub fn top(&self) -> Mode{
        self.top.clone()
    }
    pub fn top_message(&self) -> Option<String>{
        self.top_message.clone()
    }
    pub fn len(&self) -> usize{
        self.stack.len().saturating_add(1)
    }
}
impl Default for ModeStack{
    fn default() -> Self{
        ModeStack{stack: Vec::new(), message_stack: Vec::new(), top: Mode::Insert, top_message: None}
    }
}

#[cfg(test)]
mod tests{
    use crate::mode::Mode;
    use crate::mode_stack::ModeStack;

    #[test] fn default_mode_stack_is_insert(){
        let mode_stack = ModeStack::default();
        assert_eq!(Mode::Insert, mode_stack.top);
        assert_eq!(None, mode_stack.top_message);
        assert!(mode_stack.stack.is_empty());
    }

    #[test] fn push_and_pop(){
        let mut mode_stack = ModeStack::default();
        mode_stack.push(Mode::Goto, None);
        assert_eq!(Ok((Mode::Goto, None)), mode_stack.pop());
        assert!(mode_stack.pop().is_err());
    }

    #[test] fn returns_proper_len(){
        let mode_stack = ModeStack::default();
        assert_eq!(1, mode_stack.len());
    }
}

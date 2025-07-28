use crate::mode::Mode;

//TODO
//could we hold a mode related text, that could be pushed/popped with the mode  (intended for error/warning/notify/info mode text)
pub struct StackMember{
    pub mode: Mode,
    //pub text: Option<String>
}

/// Guarantees at least one element on stack
pub struct ModeStack{
    //could this be an array, if our stack will only ever be a certain amount of modes. //current worst case is Insert, SomeUtilMode, Error, Warning
    stack: Vec<Mode>,   //stack: (Vec<Mode>, Option<String>)
    top: Mode   //top: (Mode, Option<String>)
    //could we hold a mode related text, that could be pushed/popped with the mode  (intended for error/warning/notify/info mode text)
}
impl ModeStack{
    pub fn push(&mut self, new_top: Mode){
        self.stack.push(self.top.clone());
        self.top = new_top;
    }
    pub fn pop(&mut self) -> Result<Mode, String>{
        match self.stack.pop(){
            Some(new_top) => {
                let old_top = self.top.clone();
                self.top = new_top;
                Ok(old_top)
            }
            None => {
                Err("Cannot pop mode stack with single mode".to_string())
            }
        }
    }
    pub fn top(&self) -> Mode{
        self.top.clone()
    }
    pub fn len(&self) -> usize{
        self.stack.len().saturating_add(1)
    }
}
impl Default for ModeStack{
    fn default() -> Self{
        ModeStack{stack: Vec::new(), top: Mode::Insert}
    }
}

#[cfg(test)]
mod tests{
    use crate::mode::Mode;
    use crate::mode_stack::ModeStack;

    #[test] fn default_mode_stack_is_insert(){
        let mode_stack = ModeStack::default();
        assert_eq!(Mode::Insert, mode_stack.top);
        assert!(mode_stack.stack.is_empty());
    }

    #[test] fn push_and_pop(){
        let mut mode_stack = ModeStack::default();
        mode_stack.push(Mode::Goto);
        assert_eq!(Ok(Mode::Goto), mode_stack.pop());
        assert!(mode_stack.pop().is_err());
    }

    #[test] fn returns_proper_len(){
        let mode_stack = ModeStack::default();
        assert_eq!(1, mode_stack.len());
    }
}

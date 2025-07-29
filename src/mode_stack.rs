use crate::mode::Mode;

#[derive(Clone, PartialEq, Debug)] pub struct StackMember{
    pub mode: Mode,
    pub text: Option<String>
}

/// Guarantees at least one element on stack
pub struct ModeStack{
    stack: Vec<StackMember>,
    top: StackMember
}
impl ModeStack{
    pub fn push(&mut self, new_top: StackMember){
        self.stack.push(self.top.clone());
        self.top = new_top;
    }
    pub fn pop(&mut self) -> Result<StackMember, String>{
        match self.stack.pop(){
            Some(new_top) => {
                let old_top = self.top.clone();
                //
                if matches!(old_top.mode, Mode::Error | Mode::Warning | Mode::Notify | Mode::Info){
                    assert!(old_top.text.is_some());
                }else{
                    assert!(old_top.text.is_none());
                }
                //
                self.top = new_top;
                Ok(old_top)
            }
            None => {
                Err("Cannot pop mode stack with single mode".to_string())
            }
        }
    }
    pub fn top(&self) -> StackMember{
        self.top.clone()
    }
    pub fn len(&self) -> usize{
        self.stack.len().saturating_add(1)
    }
}
impl Default for ModeStack{
    fn default() -> Self{
        ModeStack{stack: Vec::new(), top: StackMember{mode: Mode::Insert, text: None}}
    }
}

#[cfg(test)]
mod tests{
    use crate::mode::Mode;
    use crate::mode_stack::{ModeStack, StackMember};

    #[test] fn default_mode_stack_is_insert(){
        let mode_stack = ModeStack::default();
        assert_eq!(StackMember{mode: Mode::Insert, text: None}, mode_stack.top);
        assert!(mode_stack.stack.is_empty());
    }

    #[test] fn push_and_pop(){
        let mut mode_stack = ModeStack::default();
        mode_stack.push(StackMember{mode: Mode::Goto, text: None});
        assert_eq!(Ok(StackMember{mode: Mode::Goto, text: None}), mode_stack.pop());
        assert!(mode_stack.pop().is_err());
    }

    #[test] fn returns_proper_len(){
        let mode_stack = ModeStack::default();
        assert_eq!(1, mode_stack.len());
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Range{       //exclusive range
    pub start: usize,
    pub end: usize,
}
impl Range{
    /// Returns a new [`Range`].
    #[must_use] pub fn new(start: usize, end: usize) -> Self{
        assert!(start <= end);

        Self{start, end}
    }

    /// Checks `self` and `other` for overlap.
    #[must_use] pub fn overlaps(&self, other: &Range) -> bool{
        self.start == other.start || 
        self.end == other.end || 
        (self.end > other.start && other.end > self.start)
    }
    
    /// Returns a bool indicating whether the provided index is contained within the [`Range`].
    #[must_use] pub fn contains(&self, idx: usize) -> bool{idx >= self.start && idx <= self.end}
    
    /// Returns a new [`Range`] representing the overlap of `self` and `other`. Returns `Option::None` if `self` and `other` are non-overlapping.
    #[must_use] pub fn intersection(&self, other: &Range) -> Option<Self>{
        if self.overlaps(other){
            Some(Range{start: self.start.max(other.start), end: self.end.min(other.end)})
        }else{None}
    }
    
    /// Create a new [`Range`] by merging self with other. Indiscriminate merge. merges whether overlapping, consecutive, contained, or disconnected entirely.
    #[must_use] pub fn merge(&self, other: &Range) -> Self{
        Range{start: self.start.min(other.start), end: self.end.max(other.end)}
    }
}

//TODO: these should be property based tests
#[cfg(test)] mod tests{
    use crate::range::Range;

    #[test] #[should_panic] fn with_start_greater_than_end(){
        let _ = Range::new(1, 0);
    }

    #[test] fn overlaps(){
        assert_eq!(true, Range::new(0, 5).overlaps(&Range::new(2, 7)));
    }
    #[test] fn doesnt_overlap(){
        assert_eq!(false, Range::new(0, 1).overlaps(&Range::new(5, 6)));
    }

    #[test] fn contains(){
        assert_eq!(true, Range::new(0, 5).contains(3));
    }
    #[test] fn doesnt_contain(){
        assert_eq!(false, Range::new(0, 5).contains(7));
    }

    #[test] fn intersection(){
        assert_eq!(Some(Range::new(2, 4)), Range::new(0, 4).intersection(&Range::new(2, 6)));
    }
    #[test] fn no_intersection(){
        assert_eq!(None, Range::new(0, 1).intersection(&Range::new(5, 6)));
    }

    #[test] fn merge(){
        assert_eq!(Range::new(0, 4), Range::new(0, 1).merge(&Range::new(3, 4)));
    }
}

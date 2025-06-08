pub struct Range{       //exclusive range
    pub start: usize,
    pub end: usize,
}
impl Range{
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

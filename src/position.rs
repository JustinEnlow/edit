/// ... is this a screen position?
#[derive(Debug, Default, Clone)]
pub struct Position{
    pub x: usize,
    pub y: usize,
}
impl Position{
    #[must_use] pub fn new(x: usize, y: usize) -> Self{
        Self{x, y}
    }
}
impl PartialEq for Position{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for Position{}

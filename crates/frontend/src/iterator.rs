#[derive(Clone, Debug)]
pub struct Iter<T: Clone> {
    pub vec: Vec<T>,
    pub pos: usize,
}

impl<T: Clone + std::cmp::PartialEq> Iter<T> {
    pub fn from<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            vec: iter.into_iter().collect(),
            pos: 0,
        }
    }

    /// Get next iterator element with moving of position
    pub fn next(&mut self) -> Option<T> {
        if self.pos >= self.vec.len() {
            None
        } else {
            {
                self.pos += 1;
                Some(self.vec[self.pos - 1].clone())
            }
        }
    }

    /// Get pending iterator element with offset
    pub fn peek(&self, indent: Option<usize>) -> Option<T> {
        if self.pos + indent.unwrap_or(0) >= self.vec.len() {
            None
        } else {
            Some(self.vec[self.pos + indent.unwrap_or(0)].clone())
        }
    }

    /// Move iterator back
    pub fn step_back(&mut self) -> Option<T> {
        if self.pos == 0 {
            return None;
        }
        self.pos -= 1;
        Some(self.vec[self.pos].clone())
    }

    /// If current iter position starts with pattern
    pub fn starts_with(&self, pattern: &[T]) -> bool {
        self.vec[self.pos..].starts_with(pattern)
    }
}

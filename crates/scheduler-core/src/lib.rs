#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Priority(pub i16);

impl Priority {
    pub fn higher_than(self, other: Self) -> bool {
        self.0 > other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_priorities() {
        assert!(Priority(10).higher_than(Priority(1)));
        assert!(!Priority(1).higher_than(Priority(10)));
    }
}

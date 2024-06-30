use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NonEmptyCircularList<T: Clone> {
    next: usize,
    list: Vec<T>,
}

impl<T: Clone> NonEmptyCircularList<T> {
    #[must_use]
    pub fn new(first: T) -> Self {
        Self {
            next: 0,
            list: vec![first],
        }
    }

    pub fn push(&mut self, item: T) {
        self.list.push(item);
    }

    #[must_use]
    pub fn next(&self) -> T {
        self.list[self.next].clone()
    }

    pub fn advance(&mut self) {
        self.next = (self.next + 1) % self.list.len();
    }
}

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct NonEmptyCircularList<T: Clone> {
    next: usize,
    list: Vec<T>,
}

impl<T: Clone + Debug> Debug for NonEmptyCircularList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut results = vec![];
        for i in 0 .. self.list.len() {
            results.push(format!(
                "{:?}",
                self.list[(self.next + i) % self.list.len()]
            ));
        }
        write!(f, "{}", results.join(", "))
    }
}

impl<T: Clone> NonEmptyCircularList<T> {
    #[must_use]
    pub fn one(first: T) -> Self {
        Self {
            next: 0,
            list: vec![first],
        }
    }

    #[must_use]
    pub fn from_vec(data: Vec<T>) -> Option<Self> {
        if data.is_empty() {
            None
        } else {
            Some(Self {
                next: 0,
                list: data,
            })
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

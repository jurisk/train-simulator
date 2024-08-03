#![allow(clippy::module_name_repetitions)]

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct NonEmptyCircularList<T> {
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

    #[must_use]
    pub fn next_index(&self) -> usize {
        self.next
    }

    pub fn remove_by_index(&mut self, index: usize) {
        if self.list.len() > 1 {
            self.list.remove(index);
            if self.next >= index {
                self.next = 0;
            }
        }
    }

    #[must_use]
    #[allow(dead_code)]
    fn iter(&self) -> NonEmptyCircularListIterator<'_, T> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

pub struct NonEmptyCircularListIterator<'a, T> {
    circular_list: &'a NonEmptyCircularList<T>,
    index:         usize,
}

impl<'a, T> Iterator for NonEmptyCircularListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.circular_list.list.get(self.index) {
            None => None,
            Some(found) => {
                self.index += 1;
                Some(found)
            },
        }
    }
}

impl<'a, T: Clone> IntoIterator for &'a NonEmptyCircularList<T> {
    type IntoIter = NonEmptyCircularListIterator<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        NonEmptyCircularListIterator {
            circular_list: self,
            index:         0,
        }
    }
}

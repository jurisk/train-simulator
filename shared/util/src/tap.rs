#![allow(clippy::module_name_repetitions)]

pub trait TapNone {
    #[must_use]
    fn tap_none<F>(self, func: F) -> Self
    where
        F: FnOnce();
}

impl<T> TapNone for Option<T> {
    fn tap_none<F>(self, func: F) -> Self
    where
        F: FnOnce(),
    {
        if self.is_none() {
            func();
        }
        self
    }
}

pub trait TapErr<T> {
    #[must_use]
    fn tap_err<F>(self, func: F) -> Self
    where
        F: FnOnce(&T);
}

pub trait Tap<T> {
    #[must_use]
    fn tap<F>(self, func: F) -> Self
    where
        F: FnOnce(&T);
}

impl<T, E> TapErr<E> for Result<T, E> {
    fn tap_err<F>(self, func: F) -> Self
    where
        F: FnOnce(&E),
    {
        if let Err(ref e) = self {
            func(e);
        }
        self
    }
}

impl<T, E> Tap<T> for Result<T, E> {
    fn tap<F>(self, func: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Ok(ref value) = self {
            func(value);
        }
        self
    }
}

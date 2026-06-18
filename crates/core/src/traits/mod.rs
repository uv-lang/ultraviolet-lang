use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub mod backend;
pub mod ffi;
pub mod frontend;

pub trait UnwrapWeakRefCell<T> {
    /// DANGEROUSLY unwraps `Weak<RefCell<T>>`
    fn unwrap_weak(&self) -> Rc<RefCell<T>>;
}

impl<T> UnwrapWeakRefCell<T> for Weak<RefCell<T>> {
    fn unwrap_weak(&self) -> Rc<RefCell<T>> {
        self.upgrade().unwrap()
    }
}

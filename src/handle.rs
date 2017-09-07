use std::{fmt, ops};
use std::marker::PhantomData;


#[repr(C)]
pub struct Handle<T> {
    pointer: *mut u8,
    marker: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(value: T) -> Self {
        Handle {
            pointer: Box::into_raw(Box::new(value)) as _,
            marker: PhantomData,
        }
    }

    pub fn unwrap(self) -> Box<T> {
        unsafe { Box::from_raw(self.pointer as _) }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            pointer: self.pointer,
            marker: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> ops::Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*(self.pointer as *mut _) }
    }
}

impl<T> ops::DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut*(self.pointer as *mut _) }
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
        //TODO
        Ok(())
    }
}

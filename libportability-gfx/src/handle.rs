use VK_NULL_HANDLE;
use std::{borrow, fmt, ops};

#[repr(C)]
pub struct Handle<T>(*mut T);

impl<T> Handle<T> {
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Handle(ptr)
    }

    pub fn unwrap(self) -> Box<T> {
        unsafe { Box::from_raw(self.0) }
    }

    pub fn is_null(&self) -> bool {
        self.0 == VK_NULL_HANDLE as *mut T
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle(self.0)
    }
}

impl<T> Copy for Handle<T> {}

impl<T> ops::Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.0 }
    }
}

impl<T> ops::DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0 }
    }
}

impl<T> borrow::Borrow<T> for Handle<T> {
    fn borrow(&self) -> &T {
        unsafe { &*self.0 }
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Handle({:p})", self.0)
    }
}

#[cfg(feature = "dispatch")]
pub use self::dispatch::DispatchHandle;
#[cfg(not(feature = "dispatch"))]
pub type DispatchHandle<T> = Handle<T>;

#[cfg(feature = "dispatch")]
mod dispatch {
    const ICD_LOADER_MAGIC: u32 = 0x01CDC0DE;

    #[repr(C)]
    pub struct DispatchHandle<T>(u32, super::Handle<T>);

    impl<T> DispatchHandle<T> {
        pub fn new(value: T) -> Self {
            DispatchHandle(ICD_LOADER_MAGIC, super::Handle::new(value))
        }

        pub fn unwrap(self) -> Box<T> {
            self.1.unwrap()
        }

        pub fn is_null(&self) -> bool {
            self.1.is_null()
        }
    }

    impl<T> Clone for DispatchHandle<T> {
        fn clone(&self) -> Self {
            DispatchHandle(self.0, self.1)
        }
    }

    impl<T> Copy for DispatchHandle<T> {}

    impl<T> ops::Deref for DispatchHandle<T> {
        type Target = T;
        fn deref(&self) -> &T {
            self.1.deref()
        }
    }

    impl<T> ops::DerefMut for DispatchHandle<T> {
        fn deref_mut(&mut self) -> &mut T {
            self.1.deref_mut()
        }
    }

    impl<T> borrow::Borrow<T> for DispatchHandle<T> {
        fn borrow(&self) -> &T {
            self.1.borrow()
        }
    }

    impl<T> fmt::Debug for DispatchHandle<T> {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "DispatchHandle({:p})", (self.1).0)
        }
    }
}

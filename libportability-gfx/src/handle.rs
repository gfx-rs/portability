use VK_NULL_HANDLE;
use std::{borrow, fmt, ops};


#[repr(C)]
pub struct Handle<T>(*mut T);

impl<T> Handle<T> {
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Handle(ptr)
    }

    pub fn null() -> Self {
        Handle(VK_NULL_HANDLE as *mut _)
    }

    pub fn unbox(self) -> T {
        *unsafe { Box::from_raw(self.0) }
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
    use VK_NULL_HANDLE;
    use std::{borrow, fmt, ops};

    const ICD_LOADER_MAGIC: u64 = 0x01CDC0DE;

    #[repr(C)]
    pub struct DispatchHandle<T>(*mut (u64, T));

    impl<T> DispatchHandle<T> {
        pub fn new(value: T) -> Self {
            let ptr = Box::into_raw(Box::new((ICD_LOADER_MAGIC, value)));
            DispatchHandle(ptr)
        }

        pub fn null() -> Self {
            DispatchHandle(VK_NULL_HANDLE as *mut _)
        }

        pub fn unbox(self) -> T {
            unsafe { Box::from_raw(self.0) }.1
        }

        pub fn is_null(&self) -> bool {
            self.0 == VK_NULL_HANDLE as *mut _
        }
    }

    impl<T> Clone for DispatchHandle<T> {
        fn clone(&self) -> Self {
            DispatchHandle(self.0)
        }
    }

    impl<T> Copy for DispatchHandle<T> {}

    impl<T> ops::Deref for DispatchHandle<T> {
        type Target = T;
        fn deref(&self) -> &T {
            unsafe { &(*self.0).1 }
        }
    }

    impl<T> ops::DerefMut for DispatchHandle<T> {
        fn deref_mut(&mut self) -> &mut T {
            unsafe { &mut (*self.0).1 }
        }
    }

    impl<T> borrow::Borrow<T> for DispatchHandle<T> {
        fn borrow(&self) -> &T {
            unsafe { &(*self.0).1 }
        }
    }

    impl<T> fmt::Debug for DispatchHandle<T> {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "DispatchHandle({:p})", self.0)
        }
    }
}

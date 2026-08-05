use std::ptr::NonNull;

pub struct CachedPropertyView<T: Clone> {
    p_value: NonNull<T>,
}

impl<T: Clone> CachedPropertyView<T> {
    pub(crate) fn new(p_value: NonNull<T>) -> Self {
        Self { p_value }
    }

    pub fn read(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }
}

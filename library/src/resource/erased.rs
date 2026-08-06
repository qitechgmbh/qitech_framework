use std::any::TypeId;
use std::ptr::NonNull;

#[derive(Clone, Copy)]
pub struct Erased {
    type_id: TypeId,
    ptr: NonNull<()>,
}

impl Erased {
    pub fn new<T: 'static>(ptr: NonNull<T>) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            ptr: ptr.cast(),
        }
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }

    pub fn downcast<T: 'static>(&self) -> Option<NonNull<T>> {
        if self.is::<T>() {
            Some(self.ptr.cast())
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The caller must guarantee that the stored type is `T`.
    pub unsafe fn downcast_unchecked<T>(&self) -> NonNull<T> {
        self.ptr.cast()
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

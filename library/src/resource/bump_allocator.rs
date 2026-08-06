use std::ptr::{self, NonNull};

/// bump allocator with rollback feature
pub struct BumpAllocator {
    buffer: Box<[u8]>,
    pos: usize,
}

impl BumpAllocator {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0u8; size].into_boxed_slice(),
            pos: 0,
        }
    }

    pub fn sync(&mut self, other: &BumpAllocator) {
        assert_eq!(
            self.buffer.len(),
            other.buffer.len(),
            "allocator size mismatch during sync"
        );

        unsafe {
            ptr::copy_nonoverlapping(
                other.buffer.as_ptr(),
                self.buffer.as_mut_ptr(),
                other.pos,
            );
        }

        self.pos = other.pos;
    }

    pub fn allocate<T>(&mut self) -> NonNull<T> {
        let align = align_of::<T>();
        let size = size_of::<T>();

        let offset = align_up(self.pos, align);

        assert!(
            offset + size <= self.buffer.len(),
            "bump allocator exhausted"
        );

        self.pos = offset + size;

        unsafe { NonNull::new_unchecked(self.buffer.as_mut_ptr().add(offset).cast::<T>()) }
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.pos
    }

    pub fn used(&self) -> usize {
        self.pos
    }

    // --- rollback ---
    pub fn mark(&self) -> BumpAllocatorMark {
        BumpAllocatorMark { pos: self.pos }
    }

    pub fn rollback(&mut self, mark: BumpAllocatorMark) {
        assert!(
            mark.pos <= self.pos,
            "cannot rollback bump allocator forward"
        );

        self.pos = mark.pos;
    }
}

// --- mark ---
#[derive(Debug, Clone, Copy)]
pub struct BumpAllocatorMark {
    pos: usize,
}

// --- utils ---
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

//! Arena Memory Allocator - Zero-Cost Memory Allocation Optimization
//!
//! Industrial-grade Arena (Bump) allocator implementation
//! For high-speed memory allocation within request lifecycle

use std::alloc::{alloc, dealloc, Layout};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;
use std::slice;

/// Arena allocator - for fast sequential allocation
///
/// # Features
/// - O(1) allocation time complexity
/// - Zero fragmentation
/// - Batch deallocation
/// - Cache-friendly contiguous memory layout
pub struct Arena {
    chunks: RefCell<Vec<ArenaChunk>>,
    current: Cell<usize>,
    current_pos: Cell<usize>,
    default_chunk_size: usize,
}

struct ArenaChunk {
    data: NonNull<u8>,
    capacity: usize,
    layout: Layout,
}

impl Arena {
    /// Create new Arena with default chunk size of 64KB
    pub fn new() -> Self {
        Self::with_capacity(64 * 1024)
    }

    /// Create Arena with specified initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let mut chunks = Vec::new();
        chunks.push(ArenaChunk::new(capacity));

        Arena {
            chunks: RefCell::new(chunks),
            current: Cell::new(0),
            current_pos: Cell::new(0),
            default_chunk_size: capacity,
        }
    }

    /// Allocate memory with specified size and alignment
    ///
    /// # Safety
    /// Returned pointer is valid until Arena is dropped
    pub fn alloc_raw(&self, layout: Layout) -> NonNull<u8> {
        let size = layout.size();
        let align = layout.align();

        // Align current position
        let current_pos = self.current_pos.get();
        let aligned_pos = (current_pos + align - 1) & !(align - 1);

        let chunks = self.chunks.borrow();
        let current_chunk = &chunks[self.current.get()];

        // Check if current chunk has enough space
        if aligned_pos + size > current_chunk.capacity {
            drop(chunks);
            self.grow(size.max(self.default_chunk_size));
            return self.alloc_raw(layout);
        }

        self.current_pos.set(aligned_pos + size);

        unsafe { NonNull::new_unchecked(current_chunk.data.as_ptr().add(aligned_pos)) }
    }

    /// Allocate single object
    pub fn alloc<T>(&self, value: T) -> &mut T {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_raw(layout);

        unsafe {
            let ptr = ptr.as_ptr() as *mut T;
            ptr.write(value);
            &mut *ptr
        }
    }

    /// Allocate array
    pub fn alloc_slice<T: Copy>(&self, len: usize) -> &mut [T] {
        let layout = Layout::array::<T>(len).unwrap();
        let ptr = self.alloc_raw(layout);

        unsafe { slice::from_raw_parts_mut(ptr.as_ptr() as *mut T, len) }
    }

    /// Allocate and initialize array
    pub fn alloc_slice_copy<T: Copy>(&self, data: &[T]) -> &mut [T] {
        let slice = self.alloc_slice(data.len());
        slice.copy_from_slice(data);
        slice
    }

    /// Allocate string
    pub fn alloc_str(&self, s: &str) -> &str {
        let bytes = self.alloc_slice_copy(s.as_bytes());
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    /// Grow Arena by adding new memory chunk
    fn grow(&self, min_size: usize) {
        let new_size = min_size.max(self.default_chunk_size);
        let new_chunk = ArenaChunk::new(new_size);

        let mut chunks = self.chunks.borrow_mut();
        chunks.push(new_chunk);

        self.current.set(chunks.len() - 1);
        self.current_pos.set(0);
    }

    /// Get current memory usage in bytes
    pub fn used_bytes(&self) -> usize {
        let chunks = self.chunks.borrow();
        let mut total = 0;

        for (i, chunk) in chunks.iter().enumerate() {
            if i < self.current.get() {
                total += chunk.capacity;
            } else if i == self.current.get() {
                total += self.current_pos.get();
            }
        }

        total
    }

    /// Get total allocation capacity
    pub fn capacity(&self) -> usize {
        self.chunks.borrow().iter().map(|c| c.capacity).sum()
    }

    /// Reset Arena, keeping memory for reuse
    pub fn reset(&mut self) {
        self.current.set(0);
        self.current_pos.set(0);
    }
}

impl ArenaChunk {
    fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 16).unwrap();
        let data = unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr).expect("allocation failed")
        };

        ArenaChunk {
            data,
            capacity,
            layout,
        }
    }
}

impl Drop for ArenaChunk {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data.as_ptr(), self.layout);
        }
    }
}

unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

/// Scoped Arena allocator
/// Automatically manages lifecycle, deallocates when leaving scope
pub struct ScopedArena<'a> {
    arena: Arena,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ScopedArena<'a> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            arena: Arena::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T {
        // Use unsafe to extend lifetime to 'a
        unsafe { mem::transmute(self.arena.alloc(value)) }
    }

    pub fn alloc_str(&'a self, s: &str) -> &'a str {
        unsafe { mem::transmute(self.arena.alloc_str(s)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let arena = Arena::new();

        let x = arena.alloc(42);
        assert_eq!(*x, 42);

        let y = arena.alloc("hello");
        assert_eq!(*y, "hello");
    }

    #[test]
    fn test_slice_allocation() {
        let arena = Arena::new();

        let slice = arena.alloc_slice_copy(&[1, 2, 3, 4, 5]);
        assert_eq!(slice, &[1, 2, 3, 4, 5]);

        slice[0] = 10;
        assert_eq!(slice[0], 10);
    }

    #[test]
    fn test_string_allocation() {
        let arena = Arena::new();

        let s1 = arena.alloc_str("Hello, ");
        let s2 = arena.alloc_str("World!");

        assert_eq!(s1, "Hello, ");
        assert_eq!(s2, "World!");
    }

    #[test]
    fn test_growth() {
        let arena = Arena::with_capacity(16);

        // Allocate data exceeding initial capacity
        for i in 0..100 {
            arena.alloc(i);
        }

        assert!(arena.capacity() > 16);
    }

    #[test]
    fn test_scoped_arena() {
        let scoped = ScopedArena::new();

        let x = scoped.alloc(42);
        let s = scoped.alloc_str("test");

        assert_eq!(*x, 42);
        assert_eq!(s, "test");
    }
}

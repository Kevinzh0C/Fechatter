//! Object Pool - Zero-Cost Object Reuse
//!
//! Production-grade object pool implementation for avoiding frequent memory allocation and deallocation

use crossbeam::queue::ArrayQueue;
use std::cell::RefCell;
use std::fmt::Debug;
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::sync::{Arc, Mutex};

/// Object pool trait - defines behavior for poolable objects
pub trait Poolable: Sized {
    /// Reset object to initial state
    fn reset(&mut self);

    /// Create new object (called when pool is empty)
    fn new() -> Self;
}

/// Single-threaded object pool - zero overhead, maximum performance
pub struct ObjectPool<T: Poolable> {
    objects: RefCell<Vec<T>>,
    capacity: usize,
}

impl<T: Poolable> ObjectPool<T> {
    /// Create object pool with specified capacity
    pub fn new(capacity: usize) -> Self {
        let mut objects = Vec::with_capacity(capacity);

        // Pre-allocate objects
        for _ in 0..capacity {
            objects.push(T::new());
        }

        ObjectPool {
            objects: RefCell::new(objects),
            capacity,
        }
    }

    /// Get object from pool
    pub fn get(&self) -> PooledObject<T> {
        let obj = self.objects.borrow_mut().pop().unwrap_or_else(T::new);

        PooledObject {
            object: Some(obj),
            pool: self,
        }
    }

    /// Return object to pool
    fn put_back(&self, mut object: T) {
        object.reset();

        let mut objects = self.objects.borrow_mut();
        if objects.len() < self.capacity {
            objects.push(object);
        }
        // If pool is full, object will be dropped
    }

    /// Get current number of objects in pool
    pub fn available(&self) -> usize {
        self.objects.borrow().len()
    }
}

/// Pooled object smart pointer - automatic return
pub struct PooledObject<'a, T: Poolable> {
    object: Option<T>,
    pool: &'a ObjectPool<T>,
}

impl<'a, T: Poolable> Drop for PooledObject<'a, T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            self.pool.put_back(object);
        }
    }
}

impl<'a, T: Poolable> Deref for PooledObject<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<'a, T: Poolable> DerefMut for PooledObject<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

/// Thread-safe object pool - using lock-free queue
pub struct ConcurrentObjectPool<T: Poolable + Send> {
    objects: Arc<ArrayQueue<T>>,
    capacity: usize,
}

impl<T: Poolable + Send> ConcurrentObjectPool<T> {
    pub fn new(capacity: usize) -> Self {
        let queue = ArrayQueue::new(capacity);

        // Pre-fill
        for _ in 0..capacity {
            let _ = queue.push(T::new());
        }

        ConcurrentObjectPool {
            objects: Arc::new(queue),
            capacity,
        }
    }

    /// Get object, create new one if pool is empty
    pub fn get(&self) -> ConcurrentPooledObject<T> {
        let object = self.objects.pop().unwrap_or_else(T::new);

        ConcurrentPooledObject {
            object: Some(object),
            pool: Arc::clone(&self.objects),
        }
    }

    /// Get current number of available objects
    pub fn available(&self) -> usize {
        self.objects.len()
    }
}

/// Thread-safe pooled object
pub struct ConcurrentPooledObject<T: Poolable + Send> {
    object: Option<T>,
    pool: Arc<ArrayQueue<T>>,
}

impl<T: Poolable + Send> Drop for ConcurrentPooledObject<T> {
    fn drop(&mut self) {
        if let Some(mut object) = self.object.take() {
            object.reset();
            // Try to return, drop if pool is full
            let _ = self.pool.push(object);
        }
    }
}

impl<T: Poolable + Send> Deref for ConcurrentPooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T: Poolable + Send> DerefMut for ConcurrentPooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

/// Fixed-size stack-based object pool - zero heap allocation
pub struct StackObjectPool<T: Poolable, const N: usize> {
    objects: RefCell<StackVec<T, N>>,
}

/// Fixed-size vector on stack
struct StackVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    fn new() -> Self {
        unsafe {
            StackVec {
                data: MaybeUninit::uninit().assume_init(),
                len: 0,
            }
        }
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }

        unsafe {
            self.data[self.len].as_mut_ptr().write(value);
        }
        self.len += 1;
        Ok(())
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        unsafe { Some(self.data[self.len].as_ptr().read()) }
    }
}

impl<T, const N: usize> Drop for StackVec<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                self.data[i].as_mut_ptr().drop_in_place();
            }
        }
    }
}

impl<T: Poolable, const N: usize> StackObjectPool<T, N> {
    pub fn new() -> Self {
        let mut vec = StackVec::new();

        // Pre-fill
        for _ in 0..N {
            let _ = vec.push(T::new());
        }

        StackObjectPool {
            objects: RefCell::new(vec),
        }
    }

    pub fn get(&self) -> StackPooledObject<T, N> {
        let object = self.objects.borrow_mut().pop().unwrap_or_else(T::new);

        StackPooledObject {
            object: Some(object),
            pool: self,
        }
    }

    fn put_back(&self, mut object: T) {
        object.reset();
        let _ = self.objects.borrow_mut().push(object);
    }
}

pub struct StackPooledObject<'a, T: Poolable, const N: usize> {
    object: Option<T>,
    pool: &'a StackObjectPool<T, N>,
}

impl<'a, T: Poolable, const N: usize> Drop for StackPooledObject<'a, T, N> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            self.pool.put_back(object);
        }
    }
}

impl<'a, T: Poolable, const N: usize> Deref for StackPooledObject<'a, T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<'a, T: Poolable, const N: usize> DerefMut for StackPooledObject<'a, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

// Examples: implement Poolable for common types
impl Poolable for Vec<u8> {
    fn reset(&mut self) {
        self.clear();
    }

    fn new() -> Self {
        Vec::with_capacity(1024)
    }
}

impl Poolable for String {
    fn reset(&mut self) {
        self.clear();
    }

    fn new() -> Self {
        String::with_capacity(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct TestObject {
        value: i32,
        data: Vec<u8>,
    }

    impl Poolable for TestObject {
        fn reset(&mut self) {
            self.value = 0;
            self.data.clear();
        }

        fn new() -> Self {
            Self::default()
        }
    }

    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::<TestObject>::new(2);

        {
            let mut obj1 = pool.get();
            obj1.value = 42;
            obj1.data.push(1);

            let mut obj2 = pool.get();
            obj2.value = 100;

            assert_eq!(pool.available(), 0);
        }

        // Objects have been returned
        // 对象已归还
        assert_eq!(pool.available(), 2);

        // 重用对象应该被重置
        let obj = pool.get();
        assert_eq!(obj.value, 0);
        assert!(obj.data.is_empty());
    }

    #[test]
    fn test_concurrent_pool() {
        use std::thread;

        let pool = Arc::new(ConcurrentObjectPool::<Vec<u8>>::new(10));
        let mut handles = vec![];

        for i in 0..20 {
            let pool = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let mut obj = pool.get();
                obj.extend_from_slice(&[i as u8; 100]);
                assert_eq!(obj.len(), 100);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_stack_pool() {
        let pool = StackObjectPool::<TestObject, 4>::new();

        let mut obj = pool.get();
        obj.value = 123;
        drop(obj);

        let obj2 = pool.get();
        assert_eq!(obj2.value, 0); // 已重置
    }
}

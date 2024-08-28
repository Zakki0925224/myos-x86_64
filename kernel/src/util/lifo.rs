use crate::error::Result;
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LifoError {
    BufferIsLocked,
    BufferIsFull,
    BufferIsEmpty,
}

#[derive(Debug)]
pub struct Lifo<T: Sized + Copy, const SIZE: usize> {
    buf: [T; SIZE],
    top: AtomicUsize,
}

impl<T: Sized + Copy, const SIZE: usize> Lifo<T, SIZE> {
    pub const fn new(default: T) -> Self {
        Self {
            buf: [default; SIZE],
            top: AtomicUsize::new(0),
        }
    }

    pub fn len(&self) -> usize {
        self.top.load(Ordering::Relaxed)
    }

    pub fn is_full(&self) -> bool {
        self.len() == SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn reset(&self) {
        self.top.store(0, Ordering::Relaxed);
    }

    pub fn push(&mut self, value: T) -> Result<()> {
        let current_top = self.top.load(Ordering::Relaxed);
        if current_top == SIZE {
            return Err(LifoError::BufferIsFull.into());
        }

        if self.top.compare_exchange(current_top, current_top + 1, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(LifoError::BufferIsLocked.into());
        }

        self.buf[current_top] = value;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T> {
        let current_top = self.top.load(Ordering::Relaxed);
        if current_top == 0 {
            return Err(LifoError::BufferIsEmpty.into());
        }

        if self.top.compare_exchange(current_top, current_top - 1, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(LifoError::BufferIsLocked.into());
        }

        Ok(self.buf[current_top - 1])
    }

    pub fn get_buf_ref(&self) -> &[T; SIZE] {
        &self.buf
    }
}

#[test_case]
fn test_new() {
    let lifo: Lifo<u8, 4> = Lifo::new(0);
    assert_eq!(lifo.len(), 0);
    assert!(lifo.is_empty());
}

#[test_case]
fn test_push_pop() {
    let mut lifo: Lifo<u8, 4> = Lifo::new(0);
    assert!(lifo.push(1).is_ok());
    assert!(lifo.push(2).is_ok());
    assert!(lifo.push(3).is_ok());
    assert!(lifo.push(4).is_ok());
    assert!(lifo.push(5).is_err());

    assert_eq!(lifo.pop(), Ok(4));
    assert_eq!(lifo.pop(), Ok(3));
    assert_eq!(lifo.pop(), Ok(2));
    assert_eq!(lifo.pop(), Ok(1));
    assert!(lifo.pop().is_err());
}

#[test_case]
fn test_reset() {
    let mut lifo: Lifo<u8, 4> = Lifo::new(0);
    lifo.push(1).unwrap();
    lifo.push(2).unwrap();
    lifo.push(3).unwrap();
    lifo.reset();

    assert_eq!(lifo.len(), 0);
    assert!(lifo.is_empty());
    assert!(lifo.pop().is_err());
}
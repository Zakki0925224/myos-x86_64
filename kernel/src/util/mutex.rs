use crate::error::Result;
use core::{
    cell::SyncUnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutexError {
    Locked,
}

// reference https://github.com/hikalium/wasabi/blob/main/os/src/mutex.rs
pub struct Mutex<T> {
    value: SyncUnsafeCell<T>,
    locked: AtomicBool,
}

impl<T: Sized> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: SyncUnsafeCell::new(value),
            locked: AtomicBool::new(false),
        }
    }

    pub fn try_lock(&self) -> Result<MutexGuard<T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            Ok(unsafe { MutexGuard::new(self, &self.value) })
        } else {
            Err(MutexError::Locked.into())
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.try_lock().unwrap_or_else(|e| panic!("{:?}", e))
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::SeqCst)
    }
}

unsafe impl<T> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    value: &'a mut T,
}

impl<'a, T> MutexGuard<'a, T> {
    unsafe fn new(mutex: &'a Mutex<T>, value: &SyncUnsafeCell<T>) -> Self {
        Self {
            mutex,
            value: &mut *value.get(),
        }
    }
}

unsafe impl<'a, T> Sync for MutexGuard<'a, T> {}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Relaxed);
    }
}

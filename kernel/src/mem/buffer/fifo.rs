use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy)]
pub enum FifoError {
    BufferIsLocked,
    BufferIsFull,
    BufferIsEmpty,
}

#[derive(Debug)]
pub struct Fifo<T: Sized + Copy, const SIZE: usize> {
    buf: [T; SIZE],
    size: usize,
    read_ptr: AtomicUsize,
    write_ptr: AtomicUsize,
}

impl<T: Sized + Copy, const SIZE: usize> Fifo<T, SIZE> {
    pub const fn new(default: T) -> Self {
        return Self {
            buf: [default; SIZE],
            size: SIZE,
            read_ptr: AtomicUsize::new(0),
            write_ptr: AtomicUsize::new(0),
        };
    }

    pub fn size(&self) -> usize {
        return self.size;
    }

    pub fn len(&self) -> usize {
        return self.write_ptr.load(Ordering::Relaxed);
    }

    pub fn reset_ptr(&self) {
        self.read_ptr.store(0, Ordering::Relaxed);
        self.write_ptr.store(0, Ordering::Relaxed);
    }

    pub fn enqueue(&mut self, value: T) -> Result<(), FifoError> {
        let read_ptr = self.read_ptr.load(Ordering::Relaxed);
        let write_ptr = self.write_ptr.load(Ordering::Relaxed);
        let next_write_ptr = (write_ptr + 1) % self.size;

        if next_write_ptr == read_ptr {
            return Err(FifoError::BufferIsFull);
        }

        if self
            .write_ptr
            .compare_exchange(
                write_ptr,
                next_write_ptr,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_err()
        {
            return Err(FifoError::BufferIsLocked);
        }

        self.buf[write_ptr] = value;

        return Ok(());
    }

    pub fn dequeue(&mut self) -> Result<T, FifoError> {
        let read_ptr = self.read_ptr.load(Ordering::Relaxed);
        let write_ptr = self.write_ptr.load(Ordering::Relaxed);
        let next_read_ptr = (read_ptr + 1) % self.size;

        if read_ptr == write_ptr {
            return Err(FifoError::BufferIsEmpty);
        }

        if self
            .read_ptr
            .compare_exchange(
                read_ptr,
                next_read_ptr,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            return Err(FifoError::BufferIsLocked);
        }

        return Ok(self.buf[read_ptr]);
    }
}

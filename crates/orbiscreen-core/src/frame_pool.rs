// Orbiscreen - orbiscreen-core - frame pool module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

const MAX_POOLED_BUFFERS_PER_SIZE: usize = 4;

#[derive(Debug, Default)]
pub struct FramePool {
    buffers: Mutex<HashMap<usize, Vec<Vec<u8>>>>,
}

impl FramePool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn acquire(self: &Arc<Self>, len: usize) -> PooledFrameBuffer {
        let data = self
            .buffers
            .lock()
            .ok()
            .and_then(|mut buckets| buckets.get_mut(&len).and_then(|v| v.pop()))
            .unwrap_or_else(|| vec![0u8; len]);
        PooledFrameBuffer {
            data,
            pool: Arc::clone(self),
        }
    }

    pub fn wrap(self: &Arc<Self>, data: Vec<u8>) -> PooledFrameBuffer {
        PooledFrameBuffer {
            data,
            pool: Arc::clone(self),
        }
    }

    fn release(&self, data: Vec<u8>) {
        let len = data.len();
        let Ok(mut buckets) = self.buffers.lock() else {
            return;
        };
        let bucket = buckets.entry(len).or_default();
        if bucket.len() < MAX_POOLED_BUFFERS_PER_SIZE {
            bucket.push(data);
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct PooledFrameBuffer {
    data: Vec<u8>,
    pool: Arc<FramePool>,
}

impl Drop for PooledFrameBuffer {
    fn drop(&mut self) {
        let data = std::mem::take(&mut self.data);
        if !data.is_empty() {
            self.pool.release(data);
        }
    }
}

impl Deref for PooledFrameBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.data
    }
}

impl DerefMut for PooledFrameBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl AsMut<[u8]> for PooledFrameBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_returns_buffer_with_requested_len() {
        let pool = FramePool::new();
        let buffer = pool.acquire(4096);
        assert_eq!(buffer.len(), 4096);
        assert!(buffer.iter().all(|b| *b == 0));
    }

    #[test]
    fn released_buffer_is_reused() {
        let pool = FramePool::new();
        let buffer = pool.acquire(1024);
        let ptr = buffer.as_ptr();
        drop(buffer);
        let recycled = pool.acquire(1024);
        assert_eq!(recycled.as_ptr(), ptr);
    }

    #[test]
    fn different_sizes_do_not_share_buckets() {
        let pool = FramePool::new();
        let small = pool.acquire(512);
        let small_ptr = small.as_ptr();
        drop(small);
        let large = pool.acquire(1024);
        assert_ne!(large.as_ptr(), small_ptr);
        assert_eq!(large.len(), 1024);
    }

    #[test]
    fn wrap_adopts_external_vec_and_returns_it_to_pool() {
        let pool = FramePool::new();
        let mut owned = vec![9u8; 128];
        let ptr = owned.as_ptr();
        owned[0] = 42;
        let buffer = pool.wrap(owned);
        assert_eq!(buffer.as_ptr(), ptr);
        assert_eq!(buffer[0], 42);
        drop(buffer);
        let recycled = pool.acquire(128);
        assert_eq!(recycled.as_ptr(), ptr);
        assert_eq!(recycled.len(), 128);
    }

    #[test]
    fn buffer_supports_mutable_access() {
        let pool = FramePool::new();
        let mut buffer = pool.acquire(8);
        buffer.copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(&buffer[..], &[1, 2, 3, 4, 5, 6, 7, 8][..]);
        let slice: &mut [u8] = buffer.as_mut();
        slice[0] = 9;
        assert_eq!(buffer[0], 9);
    }

    #[test]
    fn empty_buffers_are_not_pooled() {
        let pool = FramePool::new();
        let buffer = pool.wrap(Vec::new());
        drop(buffer);
        let buckets = pool.buffers.lock().expect("lock");
        assert!(buckets.is_empty());
    }
}

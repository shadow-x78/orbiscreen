use std::fs::File;
use std::io;
use std::os::fd::{AsRawFd as _, FromRawFd as _, RawFd};

pub fn anonymous_file(label: &str, len: usize) -> io::Result<File> {
    let name = std::ffi::CString::new(label)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("shm name: {e}")))?;
    #[allow(unsafe_code)]
    let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    #[allow(unsafe_code)]
    let file = unsafe { File::from_raw_fd(fd) };
    if len > 0 {
        truncate(&file, len)?;
    }
    Ok(file)
}

pub fn truncate(file: &File, len: usize) -> io::Result<()> {
    #[allow(unsafe_code)]
    let rc = unsafe { libc::ftruncate(file.as_raw_fd(), len as libc::off_t) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub struct MmapRegion {
    ptr: *mut u8,
    len: usize,
}

impl std::fmt::Debug for MmapRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MmapRegion")
            .field("len", &self.len)
            .finish()
    }
}

impl MmapRegion {
    pub fn empty() -> Self {
        Self {
            ptr: std::ptr::NonNull::<u8>::dangling().as_ptr(),
            len: 0,
        }
    }

    pub fn new(fd: RawFd, len: usize) -> io::Result<Self> {
        if len == 0 {
            return Ok(Self::empty());
        }
        #[allow(unsafe_code)]
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            ptr: ptr.cast(),
            len,
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[allow(unsafe_code)]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    #[allow(unsafe_code)]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for MmapRegion {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if self.len > 0 {
            unsafe { libc::munmap(self.ptr.cast(), self.len) };
        }
    }
}

#[allow(unsafe_code)]
unsafe impl Send for MmapRegion {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anonymous_file_is_mapped_and_readable() {
        let file = anonymous_file("orbiscreen-shm-test", 8).expect("memfd");
        {
            use std::io::Write as _;
            let mut writer = file.try_clone().unwrap();
            writer.write_all(b"12345678").unwrap();
        }
        let mapping = MmapRegion::new(file.as_raw_fd(), 8).expect("mmap");
        assert_eq!(mapping.as_slice(), b"12345678");
        assert_eq!(mapping.len(), 8);
    }

    #[test]
    fn zero_length_file_is_allowed() {
        let file = anonymous_file("orbiscreen-shm-empty", 0).expect("memfd");
        let mapping = MmapRegion::new(file.as_raw_fd(), 0).expect("mmap");
        assert!(mapping.is_empty());
    }
}

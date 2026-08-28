// Orbiscreen - orbiscreen-capture - x11 module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::os::fd::{AsRawFd, FromRawFd as _, OwnedFd, RawFd};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use orbiscreen_core::frame_pool::{FramePool, PooledFrameBuffer};
use tokio::task;
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::shm;
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat, Screen};
use x11rb::xcb_ffi::XCBConnection;

use super::{CaptureError, CapturedFrame};

const SKIP_POLL_INTERVAL: Duration = Duration::from_millis(8);
const SHM_LABEL: &str = "orbiscreen-xshm";

#[allow(missing_debug_implementations)]
pub struct X11Capture {
    conn: Arc<XCBConnection>,
    screen: Screen,
    width: u32,
    height: u32,
    pool: Arc<FramePool>,
    shm: Arc<Mutex<Option<ShmSegment>>>,
    last_hash: Arc<Mutex<Option<[u64; 2]>>>,
}

impl X11Capture {
    pub fn open(width: u32, height: u32) -> Result<Self, CaptureError> {
        let (conn, screen_num) = XCBConnection::connect(None)?;
        let screen = conn.setup().roots[screen_num].clone();
        let cap_w = width.min(screen.width_in_pixels as u32);
        let cap_h = height.min(screen.height_in_pixels as u32);
        if cap_w != width || cap_h != height {
            tracing::warn!(
                requested = format!("{width}x{height}"),
                actual = format!("{cap_w}x{cap_h}"),
                "capture region clamped to root window size"
            );
        }
        let conn = Arc::new(conn);
        let shm = open_shm_segment(&conn, cap_w, cap_h);
        if shm.is_some() {
            tracing::info!(
                "X11 capture uses MIT-SHM (pooled shared image, zero per-frame replies)"
            );
        } else {
            tracing::info!("X11 capture uses plain GetImage (MIT-SHM unavailable)");
        }
        Ok(Self {
            conn,
            screen,
            width: cap_w,
            height: cap_h,
            pool: FramePool::new(),
            shm: Arc::new(Mutex::new(shm)),
            last_hash: Arc::new(Mutex::new(None)),
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn uses_shm(&self) -> bool {
        self.shm
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        loop {
            let conn = Arc::clone(&self.conn);
            let screen = self.screen.clone();
            let (width, height) = (self.width, self.height);
            let pool = Arc::clone(&self.pool);
            let shm = Arc::clone(&self.shm);
            let last_hash = Arc::clone(&self.last_hash);
            let outcome = task::spawn_blocking(move || {
                capture_changed_frame(&conn, &screen, width, height, &pool, &shm, &last_hash)
            })
            .await
            .map_err(|e| CaptureError::Io(format!("capture task join error: {e}")))?;
            match outcome? {
                Some(frame) => return Ok(frame),
                None => tokio::time::sleep(SKIP_POLL_INTERVAL).await,
            }
        }
    }
}

struct ShmSegment {
    seg: shm::Seg,
    conn: Arc<XCBConnection>,
    _file: std::fs::File,
    mapping: Mapping,
}

impl Drop for ShmSegment {
    fn drop(&mut self) {
        let _ = shm::detach(&self.conn, self.seg);
    }
}

struct Mapping {
    ptr: *mut u8,
    len: usize,
}

impl Mapping {
    #[allow(unsafe_code)]
    fn new(fd: RawFd, len: usize) -> Result<Self, CaptureError> {
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
            return Err(CaptureError::Io(format!(
                "shm mmap: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(Self {
            ptr: ptr.cast(),
            len,
        })
    }

    #[allow(unsafe_code)]
    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for Mapping {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { libc::munmap(self.ptr.cast(), self.len) };
        }
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Mapping {}
#[allow(unsafe_code)]
unsafe impl Send for ShmSegment {}

#[allow(unsafe_code)]
fn anonymous_memfd(label: &str) -> Result<OwnedFd, CaptureError> {
    let name =
        std::ffi::CString::new(label).map_err(|e| CaptureError::Io(format!("shm name: {e}")))?;
    let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(CaptureError::Io(format!(
            "memfd_create: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn open_shm_segment(conn: &Arc<XCBConnection>, width: u32, height: u32) -> Option<ShmSegment> {
    let len = (width as usize) * (height as usize) * 4;
    conn.extension_information(shm::X11_EXTENSION_NAME)
        .ok()
        .flatten()?;
    let version = shm::query_version(conn).ok()?.reply().ok()?;
    if (version.major_version, version.minor_version) < (1, 2) {
        tracing::info!(
            "MIT-SHM {}.{} is older than 1.2: no fd passing; using plain GetImage",
            version.major_version,
            version.minor_version
        );
        return None;
    }
    let memfd = anonymous_memfd(SHM_LABEL).ok()?;
    let file = memfd_to_file(memfd);
    if ftruncate(file.as_raw_fd(), len as i64) != 0 {
        tracing::warn!("shm ftruncate failed: {}", std::io::Error::last_os_error());
        return None;
    }
    let seg = conn.generate_id().ok()?;
    let fd_for_server = file.try_clone().ok()?;
    if shm::attach_fd(conn, seg, fd_for_server, false)
        .ok()?
        .check()
        .is_err()
    {
        tracing::warn!("MIT-SHM attach_fd was rejected; using plain GetImage");
        return None;
    }
    let mapping = match Mapping::new(file.as_raw_fd(), len) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("shm mmap failed: {e}; using plain GetImage");
            let _ = shm::detach(conn, seg);
            return None;
        }
    };
    Some(ShmSegment {
        seg,
        conn: Arc::clone(conn),
        _file: file,
        mapping,
    })
}

fn memfd_to_file(fd: OwnedFd) -> std::fs::File {
    std::fs::File::from(fd)
}

#[allow(unsafe_code)]
fn ftruncate(fd: RawFd, len: i64) -> i32 {
    unsafe { libc::ftruncate(fd, len as libc::off_t) }
}

fn capture_changed_frame(
    conn: &XCBConnection,
    screen: &Screen,
    width: u32,
    height: u32,
    pool: &Arc<FramePool>,
    shm: &Mutex<Option<ShmSegment>>,
    last_hash: &Mutex<Option<[u64; 2]>>,
) -> Result<Option<CapturedFrame>, CaptureError> {
    let expected = CapturedFrame::size_in_bytes(width, height);
    let mut shm_guard = shm
        .lock()
        .map_err(|_| CaptureError::Io("shm state lock poisoned".into()))?;
    let source = match shm_guard.as_mut() {
        Some(segment) => match shm_frame_source(conn, screen, segment, width, height, expected) {
            Ok(()) => FrameSource::Shm(segment.mapping.as_slice()),
            Err(e) => {
                tracing::warn!("MIT-SHM capture failed ({e}); falling back to plain GetImage");
                let _seg = shm_guard.take();
                FrameSource::Fallback
            }
        },
        None => FrameSource::Fallback,
    };
    let data = match source {
        FrameSource::Shm(slice) => {
            let hash = fast_frame_hash(&slice[..expected]);
            if frame_is_unchanged(last_hash, hash)? {
                return Ok(None);
            }
            let mut data = pool.acquire(expected);
            data.copy_from_slice(&slice[..expected]);
            data
        }
        FrameSource::Fallback => {
            let data = get_image_frame(conn, screen, width, height, expected, pool)?;
            let hash = fast_frame_hash(&data);
            if frame_is_unchanged(last_hash, hash)? {
                return Ok(None);
            }
            data
        }
    };
    Ok(Some(CapturedFrame {
        width,
        height,
        data,
    }))
}

enum FrameSource<'a> {
    Shm(&'a [u8]),
    Fallback,
}

fn shm_frame_source(
    conn: &XCBConnection,
    screen: &Screen,
    segment: &ShmSegment,
    width: u32,
    height: u32,
    expected: usize,
) -> Result<(), CaptureError> {
    let reply = shm::get_image(
        conn,
        screen.root,
        0,
        0,
        width as u16,
        height as u16,
        u32::MAX,
        ImageFormat::Z_PIXMAP.into(),
        segment.seg,
        0,
    )
    .map_err(|e| CaptureError::X11Connect(e.to_string()))?
    .reply()
    .map_err(|e| match e {
        x11rb::errors::ReplyError::X11Error(err) => CaptureError::X11Protocol(err.error_code),
        x11rb::errors::ReplyError::ConnectionError(err) => {
            CaptureError::X11Connect(err.to_string())
        }
    })?;
    if (reply.size as usize) < expected {
        return Err(CaptureError::Io(format!(
            "shm GetImage wrote {} B, expected {expected} B",
            reply.size
        )));
    }
    Ok(())
}

fn get_image_frame(
    conn: &XCBConnection,
    screen: &Screen,
    width: u32,
    height: u32,
    expected: usize,
    pool: &Arc<FramePool>,
) -> Result<PooledFrameBuffer, CaptureError> {
    let cookie = conn.get_image(
        ImageFormat::Z_PIXMAP,
        screen.root,
        0,
        0,
        width as u16,
        height as u16,
        u32::MAX,
    );
    let reply = cookie?.reply().map_err(|e| match e {
        x11rb::errors::ReplyError::X11Error(err) => CaptureError::X11Protocol(err.error_code),
        x11rb::errors::ReplyError::ConnectionError(err) => {
            CaptureError::X11Connect(err.to_string())
        }
    })?;
    if reply.data.len() < expected {
        return Err(CaptureError::Io(format!(
            "GetImage reply truncated: need {expected} B, have {}",
            reply.data.len()
        )));
    }
    let mut data = pool.acquire(expected);
    data.copy_from_slice(&reply.data[..expected]);
    Ok(data)
}

fn frame_is_unchanged(
    last_hash: &Mutex<Option<[u64; 2]>>,
    hash: [u64; 2],
) -> Result<bool, CaptureError> {
    let mut guard = last_hash
        .lock()
        .map_err(|_| CaptureError::Io("frame hash lock poisoned".into()))?;
    let unchanged = guard.as_ref() == Some(&hash);
    *guard = Some(hash);
    Ok(unchanged)
}

pub(crate) fn fast_frame_hash(data: &[u8]) -> [u64; 2] {
    let mut a: u64 = 0xcbf2_9ce4_8422_2325;
    let mut b: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut rest = data;
    while rest.len() >= 16 {
        let mut lo = [0u8; 8];
        let mut hi = [0u8; 8];
        lo.copy_from_slice(&rest[..8]);
        hi.copy_from_slice(&rest[8..16]);
        a = (a ^ u64::from_le_bytes(lo)).wrapping_mul(0x0000_0100_0000_01b3);
        b = (b ^ u64::from_le_bytes(hi)).wrapping_mul(0x8da0_b8e1_8098_b5d5);
        rest = &rest[16..];
    }
    for &byte in rest {
        a = (a ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
        b = (b ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    a ^= data.len() as u64;
    b ^= data.len() as u64;
    [a, b]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let data = vec![0xAB; 64];
        assert_eq!(fast_frame_hash(&data), fast_frame_hash(&data));
    }

    #[test]
    fn hash_detects_single_bit_change() {
        let mut a = vec![0u8; 256];
        let mut b = a.clone();
        b[127] ^= 1;
        assert_ne!(fast_frame_hash(&a), fast_frame_hash(&b));
        a[0] ^= 0x80;
        assert_ne!(fast_frame_hash(&a), fast_frame_hash(&b));
    }

    #[test]
    fn hash_depends_on_length() {
        let short = vec![0u8; 15];
        let long = vec![0u8; 16];
        assert_ne!(fast_frame_hash(&short), fast_frame_hash(&long));
    }

    #[test]
    fn hash_handles_unaligned_tail() {
        let data: Vec<u8> = (0..37u8).collect();
        assert_eq!(fast_frame_hash(&data), fast_frame_hash(&data));
    }

    #[test]
    fn hash_handles_empty_input() {
        assert_eq!(fast_frame_hash(&[]), fast_frame_hash(&[]));
    }
}

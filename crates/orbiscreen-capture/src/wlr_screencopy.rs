use std::os::fd::{AsFd, AsRawFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::Notify;
use wayland_client::backend::WaylandError;
use wayland_client::protocol::wl_buffer::{self, WlBuffer};
use wayland_client::protocol::wl_output::{self, WlOutput};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_shm::{self, WlShm};
use wayland_client::protocol::wl_shm_pool::{self, WlShmPool};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::{
    self as frame_proto, ZwlrScreencopyFrameV1,
};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::{
    self as manager_proto, ZwlrScreencopyManagerV1,
};

use orbiscreen_core::frame_pool::FramePool;

use crate::{CaptureError, CapturedFrame};

const FRAME_CHANNEL_CAPACITY: usize = 2;
const HANDSHAKE_DEADLINE: Duration = Duration::from_secs(3);
const EVENT_POLL_TIMEOUT_MS: i32 = 100;

#[derive(Debug, Clone, Default)]
pub struct WlrScreencopySpec {
    pub output_name: Option<String>,
}

#[derive(Debug, Error)]
pub enum WlrScreencopyError {
    #[error("zwlr_screencopy_manager_v1 is not available: this compositor is not wlroots-based (or the protocol is disabled)")]
    ProtocolUnavailable,
    #[error("no wl_shm global: cannot allocate shared buffers")]
    ShmUnavailable,
    #[error("no wl_output global is available to capture")]
    NoOutputs,
    #[error("no output named {0:?} was advertised by the compositor")]
    OutputNotFound(String),
    #[error("the compositor does not offer an XRGB8888/ARGB8888 shm format")]
    UnsupportedFormat,
    #[error("timed out waiting for output metadata")]
    Timeout,
    #[error("compositor aborted the screencopy frame")]
    FrameFailed,
    #[error("wayland error: {0}")]
    Wayland(String),
}

impl From<WlrScreencopyError> for CaptureError {
    fn from(error: WlrScreencopyError) -> Self {
        CaptureError::Io(error.to_string())
    }
}

#[derive(Debug, Clone)]
struct OutputInfo {
    proxy: WlOutput,
    name: Option<String>,
    width: i32,
    height: i32,
    got_done: bool,
}

struct PendingFrame {
    buffer: Option<WlBuffer>,
    pool: Option<WlShmPool>,
    mapping: Mapping,
    width: u32,
    height: u32,
    stride: u32,
    premultiplied: bool,
    ready: bool,
    failed: bool,
}

struct Mapping {
    ptr: *mut u8,
    len: usize,
}

impl Mapping {
    #[allow(unsafe_code)]
    fn new(fd: std::os::fd::RawFd, len: usize) -> Result<Self, String> {
        // SAFETY: `fd` is the raw descriptor of a live `std::fs::File` (`file` in the
        // `Dispatch<ZwlrScreencopyFrameV1, ()>` `Buffer` arm), which stays alive across this
        // call, so the `MAP_SHARED` mapping cannot be installed over a descriptor that is
        // closed underneath it. `len` was validated above: `checked_mul(stride, height)`
        // filtered to `<= i32::MAX` and then `usize::try_from`, so the requested length is
        // exactly the buffer the compositor is about to fill and cannot overflow the length
        // argument. A null address hint lets the kernel pick the address, so the mapping
        // cannot alias the Rust frame it is created in, and `PROT_READ | PROT_WRITE` together
        // with `MAP_SHARED` is what `zwlr_screencopy_frame_v1.buffer` requires of the shm
        // pool. `MAP_FAILED` is rejected on the next line, so an invalid address is never
        // stored.
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
            return Err(format!("mmap: {}", std::io::Error::last_os_error()));
        }
        Ok(Self {
            ptr: ptr.cast(),
            len,
        })
    }

    #[allow(unsafe_code)]
    fn as_slice(&self) -> &[u8] {
        // SAFETY: `ptr` and `len` are exactly the pair captured from the successful `mmap` in
        // `Mapping::new` (a `MAP_FAILED` result is rejected there and the fields are never
        // mutated afterwards), so `[ptr, ptr + len)` is mapped memory of at least `len` bytes;
        // `mmap` rounds the length up to whole pages and returns zero-filled pages for a
        // freshly truncated memfd, so every one of those `len` bytes is initialized, which is
        // what the `u8` element type of the slice requires. The null sentinel built by
        // `PendingFrame::failed` (`ptr: null_mut`, `len: 0`) can never reach this call:
        // `run_capture_loop` returns early on `pending.mapping.len == 0` and `copy_frame` is
        // the only caller of `as_slice`. The returned slice borrows `&self`, so it cannot
        // outlive the `Mapping` and `Drop::drop` cannot `munmap` underneath it.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for Mapping {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        // SAFETY: the null check above skips the `PendingFrame::failed` sentinel, so `ptr` and
        // `len` are the unchanged address/length pair that the successful `mmap` in
        // `Mapping::new` was given, which is precisely the pair `munmap` must be given for the
        // same mapping. `&mut self` proves exclusive access, so no `&[u8]` handed out by
        // `as_slice` is still live while the pages are torn down. The return value is ignored
        // because unmapping a mapping that `mmap` just created cannot fail.
        unsafe { libc::munmap(self.ptr.cast(), self.len) };
    }
}

#[allow(unsafe_code)]
// SAFETY: `Mapping` owns its mapping outright: the `ptr`/`len` fields are private, the type
// is neither `Copy` nor `Clone`, and no API ever hands out a `&mut` view of the mapped bytes,
// so moving the value to another thread cannot produce two views of the same pages. The value
// is only ever moved into `state.pending` and consumed on the single `orbiscreen-wlr-copy`
// event thread spawned in `WlrScreencopyCapture::open`, where `copy_frame` borrows it and the
// frame is released before the loop advances. `Mapping` deliberately stays `!Sync`, so no
// shared reference to it crosses a thread boundary either.
unsafe impl Send for Mapping {}

#[derive(Default)]
struct CaptureState {
    shm: Option<WlShm>,
    manager: Option<(ZwlrScreencopyManagerV1, u32)>,
    damage_copy: bool,
    outputs: Vec<OutputInfo>,
    pending: Option<PendingFrame>,
}

impl Dispatch<WlRegistry, ()> for CaptureState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == <WlShm as Proxy>::interface().name {
                state.shm = Some(registry.bind(name, version.min(1), qh, ()));
            } else if interface == <ZwlrScreencopyManagerV1 as Proxy>::interface().name {
                let bound = version.min(<ZwlrScreencopyManagerV1 as Proxy>::interface().version);
                state.damage_copy = bound >= 2;
                state.manager = Some((registry.bind(name, bound, qh, ()), bound));
            } else if interface == <WlOutput as Proxy>::interface().name {
                let proxy: WlOutput = registry.bind(name, version.min(4), qh, ());
                state.outputs.push(OutputInfo {
                    proxy,
                    name: None,
                    width: 0,
                    height: 0,
                    got_done: false,
                });
            }
        }
    }
}

impl Dispatch<WlOutput, ()> for CaptureState {
    fn event(
        state: &mut Self,
        output: &WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(info) = state.outputs.iter_mut().find(|o| o.proxy == *output) else {
            return;
        };
        match event {
            wl_output::Event::Mode { width, height, .. } => {
                info.width = width;
                info.height = height;
            }
            wl_output::Event::Name { name } => {
                info.name = Some(name);
            }
            wl_output::Event::Done => {
                info.got_done = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<WlShm, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlShmPool, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &WlShmPool,
        _: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlBuffer, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &WlBuffer,
        _: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrScreencopyManagerV1, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &ZwlrScreencopyManagerV1,
        _: manager_proto::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for CaptureState {
    fn event(
        state: &mut Self,
        frame: &ZwlrScreencopyFrameV1,
        event: frame_proto::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            frame_proto::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                let Some(shm) = state.shm.as_ref() else {
                    tracing::warn!("screencopy buffer event but no wl_shm global");
                    return;
                };
                let (premultiplied, fmt) = match format {
                    WEnum::Value(f @ wl_shm::Format::Xrgb8888) => (true, f),
                    WEnum::Value(f @ wl_shm::Format::Argb8888) => (false, f),
                    other => {
                        tracing::warn!("screencopy offered unsupported shm format {other:?}");
                        state.pending = Some(PendingFrame::failed(width, height, stride));
                        return;
                    }
                };
                let dims_fit = width <= i32::MAX as u32
                    && height <= i32::MAX as u32
                    && stride <= i32::MAX as u32;
                let len = u64::from(stride)
                    .checked_mul(u64::from(height))
                    .filter(|l| *l <= i32::MAX as u64)
                    .and_then(|l| usize::try_from(l).ok());
                let Some(len) = len.filter(|_| dims_fit) else {
                    tracing::warn!("screencopy offered frame with out-of-range dimensions");
                    state.pending = Some(PendingFrame::failed(width, height, stride));
                    return;
                };
                let Ok(file) = anonymous_shm_file("orbiscreen-screencopy") else {
                    state.pending = Some(PendingFrame::failed(width, height, stride));
                    return;
                };
                if unsafe_ftruncate(file.as_raw_fd(), len) != 0 {
                    tracing::warn!("ftruncate failed: {}", std::io::Error::last_os_error());
                    state.pending = Some(PendingFrame::failed(width, height, stride));
                    return;
                }
                let mapping = match Mapping::new(file.as_raw_fd(), len) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!("screencopy mmap failed: {e}");
                        state.pending = Some(PendingFrame::failed(width, height, stride));
                        return;
                    }
                };
                let fd = borrowed_fd(file.as_raw_fd());
                let pool = shm.create_pool(fd.as_fd(), len as i32, qh, ());
                let buffer =
                    pool.create_buffer(0, width as i32, height as i32, stride as i32, fmt, qh, ());
                if state.damage_copy {
                    frame.copy_with_damage(&buffer);
                } else {
                    frame.copy(&buffer);
                }
                state.pending = Some(PendingFrame {
                    buffer: Some(buffer),
                    pool: Some(pool),
                    mapping,
                    width,
                    height,
                    stride,
                    premultiplied,
                    ready: false,
                    failed: false,
                });
            }
            frame_proto::Event::Flags { .. } => {}
            frame_proto::Event::Ready { .. } => {
                if let Some(pending) = state.pending.as_mut() {
                    pending.ready = true;
                }
            }
            frame_proto::Event::Failed => {
                if let Some(pending) = state.pending.as_mut() {
                    pending.failed = true;
                }
            }
            frame_proto::Event::Damage { .. } => {}
            _ => {}
        }
    }
}

impl PendingFrame {
    fn failed(width: u32, height: u32, stride: u32) -> Self {
        Self {
            buffer: None,
            pool: None,
            mapping: Mapping {
                ptr: std::ptr::null_mut(),
                len: 0,
            },
            width,
            height,
            stride,
            premultiplied: false,
            ready: false,
            failed: true,
        }
    }

    fn release(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            buffer.destroy();
        }
        if let Some(pool) = self.pool.take() {
            pool.destroy();
        }
    }
}

#[allow(unsafe_code)]
fn anonymous_shm_file(label: &str) -> Result<std::fs::File, String> {
    use std::os::fd::FromRawFd as _;
    let name = std::ffi::CString::new(label).map_err(|e| format!("shm name: {e}"))?;
    // SAFETY: `name` is a `CString` that outlives this call and contains no interior NUL, so
    // `name.as_ptr()` is a valid NUL-terminated string; `memfd_create` only reads it (the
    // kernel copies it into the `/proc/self/fd` link) and derives no length from Rust, so
    // there is no uninitialized or out-of-bounds read. `MFD_CLOEXEC` keeps the shm descriptor
    // from leaking across `exec`.
    let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(format!("memfd_create: {}", std::io::Error::last_os_error()));
    }
    // SAFETY: `fd` is the value `memfd_create` returned on the previous line, the `fd < 0`
    // guard above already turned the error case into `Err`, and no other binding owns or
    // closes it yet, so it is a fresh, valid, open descriptor. Ownership is transferred to
    // the returned `File` exactly once, and `File` closes it on drop.
    Ok(unsafe { std::fs::File::from_raw_fd(fd) })
}

#[allow(unsafe_code)]
fn unsafe_ftruncate(fd: std::os::fd::RawFd, len: usize) -> i32 {
    // SAFETY: the only caller passes `file.as_raw_fd()` of a `std::fs::File` that is still
    // alive for the duration of the call, so the descriptor is valid and open. `len` was
    // computed as `checked_mul(stride, height)`, filtered to `<= i32::MAX`, and converted with
    // `usize::try_from`, so `len as libc::off_t` is lossless and non-negative rather than a
    // truncated or wrapped value. The caller checks the non-zero return and turns it into a
    // failed frame.
    unsafe { libc::ftruncate(fd, len as libc::off_t) }
}

#[allow(unsafe_code)]
fn borrowed_fd(fd: std::os::fd::RawFd) -> std::os::fd::BorrowedFd<'static> {
    // SAFETY: the fabricated `'static` lifetime can never be observed, because the returned
    // `BorrowedFd` is a private local of the caller's `Buffer` arm and is destroyed at the end
    // of that scope; it is declared after `file`, so it drops first, i.e. strictly before the
    // owning `std::fs::File` closes the descriptor. While it exists it is only used through
    // `as_fd()` on the next line to pass the descriptor to `wl_shm.create_pool`, which reads
    // it and closes nothing, and `BorrowedFd` itself has no `Drop` that closes the fd.
    unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) }
}

#[allow(missing_debug_implementations)]
pub struct WlrScreencopyCapture {
    rx: tokio::sync::Mutex<mpsc::Receiver<CapturedFrame>>,
    width: u32,
    height: u32,
    stop: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
    ended_notify: Arc<Notify>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl WlrScreencopyCapture {
    pub fn open(spec: WlrScreencopySpec) -> Result<Self, WlrScreencopyError> {
        let conn =
            Connection::connect_to_env().map_err(|e| WlrScreencopyError::Wayland(e.to_string()))?;
        let mut queue: EventQueue<CaptureState> = conn.new_event_queue();
        let qh = queue.handle();
        let mut state = CaptureState::default();
        conn.display().get_registry(&qh, ());

        let deadline = Instant::now() + HANDSHAKE_DEADLINE;
        loop {
            queue
                .roundtrip(&mut state)
                .map_err(|e| WlrScreencopyError::Wayland(e.to_string()))?;
            let globals_ready = state.manager.is_some() && state.shm.is_some();
            let output_ready = match &spec.output_name {
                Some(name) => state
                    .outputs
                    .iter()
                    .any(|o| o.name.as_deref() == Some(name.as_str()) && o.width > 0),
                None => state.outputs.iter().any(|o| o.got_done || o.width > 0),
            };
            if globals_ready && output_ready {
                break;
            }
            if Instant::now() >= deadline {
                break;
            }
        }
        if state.manager.is_none() {
            return Err(WlrScreencopyError::ProtocolUnavailable);
        }
        if state.shm.is_none() {
            return Err(WlrScreencopyError::ShmUnavailable);
        }
        if state.outputs.is_empty() {
            return Err(WlrScreencopyError::NoOutputs);
        }

        let output_index = match &spec.output_name {
            Some(name) => state
                .outputs
                .iter()
                .position(|o| o.name.as_deref() == Some(name))
                .ok_or_else(|| WlrScreencopyError::OutputNotFound(name.clone()))?,
            None => state
                .outputs
                .iter()
                .position(|o| o.got_done || o.width > 0)
                .unwrap_or(0),
        };
        let output = state.outputs[output_index].clone();
        if output.width <= 0 || output.height <= 0 {
            return Err(WlrScreencopyError::Timeout);
        }
        let (width, height) = (output.width as u32, output.height as u32);

        let (tx, rx) = mpsc::channel::<CapturedFrame>(FRAME_CHANNEL_CAPACITY);
        let stop = Arc::new(AtomicBool::new(false));
        let ended = Arc::new(AtomicBool::new(false));
        let ended_notify = Arc::new(Notify::new());
        let pool = FramePool::new();

        let thread = std::thread::Builder::new()
            .name("orbiscreen-wlr-copy".into())
            .spawn({
                let stop = Arc::clone(&stop);
                let ended = Arc::clone(&ended);
                let ended_notify = Arc::clone(&ended_notify);
                let pool = Arc::clone(&pool);
                move || {
                    run_capture_loop(
                        conn,
                        queue,
                        state,
                        output,
                        tx,
                        stop,
                        ended,
                        ended_notify,
                        pool,
                    )
                }
            })
            .map_err(|e| WlrScreencopyError::Wayland(format!("spawn capture thread: {e}")))?;

        Ok(Self {
            rx: tokio::sync::Mutex::new(rx),
            width,
            height,
            stop,
            ended,
            ended_notify,
            thread: Some(thread),
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn is_ended(&self) -> bool {
        self.ended.load(Ordering::Relaxed)
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        let mut rx = self.rx.lock().await;
        loop {
            if self.ended.load(Ordering::Relaxed) {
                return Err(CaptureError::Io("wlroots screencopy capture ended".into()));
            }
            tokio::select! {
                frame = rx.recv() => {
                    return match frame {
                        Some(frame) => Ok(frame),
                        None => Err(CaptureError::Io(
                            "wlroots screencopy capture thread closed".into(),
                        )),
                    };
                }
                _ = self.ended_notify.notified() => continue,
            }
        }
    }
}

impl Drop for WlrScreencopyCapture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_capture_loop(
    conn: Connection,
    mut queue: EventQueue<CaptureState>,
    mut state: CaptureState,
    output: OutputInfo,
    tx: mpsc::Sender<CapturedFrame>,
    stop: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
    ended_notify: Arc<Notify>,
    pool: Arc<FramePool>,
) {
    let qh = queue.handle();
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let Some((manager, _version)) = state.manager.as_ref() else {
            break;
        };
        let frame = manager.capture_output(0, &output.proxy, &qh, ());

        let mut stopped = false;
        loop {
            if stop.load(Ordering::Relaxed) {
                stopped = true;
                break;
            }
            if let Err(e) = queue.dispatch_pending(&mut state) {
                tracing::warn!("screencopy dispatch failed: {e}");
                ended.store(true, Ordering::Relaxed);
                ended_notify.notify_waiters();
                return;
            }
            if let Some(pending) = state.pending.as_mut() {
                if pending.ready || pending.failed {
                    break;
                }
            }
            let Some(guard) = conn.prepare_read() else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };
            let fd = guard.connection_fd().as_raw_fd();
            let mut fds = [libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            }];
            let ready = poll_fd_readable(&mut fds);
            if ready > 0 && fds[0].revents & libc::POLLIN != 0 {
                if let Err(e) = guard.read() {
                    let would_block = matches!(&e, WaylandError::Io(io) if io.kind() == std::io::ErrorKind::WouldBlock);
                    if !would_block {
                        tracing::warn!("screencopy read failed: {e}");
                        ended.store(true, Ordering::Relaxed);
                        ended_notify.notify_waiters();
                        return;
                    }
                }
            }
            let _ = conn.flush();
        }
        frame.destroy();

        let mut pending = match state.pending.take() {
            Some(p) => p,
            None => {
                if stopped {
                    break;
                }
                continue;
            }
        };
        if stopped {
            pending.release();
            break;
        }
        if pending.failed || pending.mapping.len == 0 {
            pending.release();
            ended.store(true, Ordering::Relaxed);
            ended_notify.notify_waiters();
            return;
        }

        let copied = copy_frame(&pending, &pool);
        pending.release();

        let Some(frame_data) = copied else {
            continue;
        };
        match tx.try_send(frame_data) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                tracing::debug!("screencopy frame dropped: consumer channel full");
            }
            Err(mpsc::error::TrySendError::Closed(_)) => break,
        }
    }
}

#[allow(unsafe_code)]
fn poll_fd_readable(fds: &mut [libc::pollfd; 1]) -> libc::c_int {
    // SAFETY: `fds` is `&mut [libc::pollfd; 1]`, so `as_mut_ptr()` is a writable, correctly
    // aligned array of exactly one element that is uniquely borrowed for the whole call, and
    // the `nfds` argument is the matching literal `1`, so `poll` can neither read nor write
    // past that element; the only field it writes is `revents`, which the caller inspects via
    // `fds[0].revents` after the call returns. The `fd` in element 0 comes from
    // `guard.connection_fd()` in the caller and that read guard is still alive here, so the
    // Wayland socket stays open across the call. `EVENT_POLL_TIMEOUT_MS` is a bounded, valid
    // timeout and `poll` allocates nothing, so no aliasing can be introduced meanwhile.
    unsafe { libc::poll(fds.as_mut_ptr(), 1, EVENT_POLL_TIMEOUT_MS) }
}

fn copy_frame(pending: &PendingFrame, pool: &Arc<FramePool>) -> Option<CapturedFrame> {
    let row_bytes = u64::from(pending.width).checked_mul(4)?;
    let len = row_bytes.checked_mul(u64::from(pending.height))?;
    let len = usize::try_from(len).ok()?;
    if row_bytes > u64::from(pending.stride) || len > pending.mapping.len {
        tracing::warn!("screencopy frame dimensions inconsistent with buffer; dropping");
        return None;
    }
    let mut data = pool.acquire(len);
    if !assemble_frame_rows(
        &mut data,
        pending.mapping.as_slice(),
        pending.width,
        pending.height,
        pending.stride,
        pending.premultiplied,
    ) {
        return None;
    }
    Some(CapturedFrame {
        width: pending.width,
        height: pending.height,
        data,
    })
}

fn assemble_frame_rows(
    dst: &mut [u8],
    src: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    premultiplied: bool,
) -> bool {
    let row_bytes = (width as usize) * 4;
    let expected = row_bytes * height as usize;
    if dst.len() != expected {
        tracing::warn!("screencopy destination size mismatch; dropping frame");
        return false;
    }
    let stride = stride as usize;
    for row in 0..height as usize {
        let start = row * stride;
        let end = start + row_bytes;
        if end > src.len() {
            tracing::warn!("screencopy buffer shorter than expected; dropping frame");
            return false;
        }
        let dst = &mut dst[row * row_bytes..row * row_bytes + row_bytes];
        dst.copy_from_slice(&src[start..end]);
        if premultiplied {
            for px in dst.as_chunks_mut::<4>().0 {
                px[3] = 0xFF;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assemble(
        src: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        premultiplied: bool,
    ) -> Option<Vec<u8>> {
        let mut dst = vec![0u8; (width as usize) * 4 * height as usize];
        if assemble_frame_rows(&mut dst, src, width, height, stride, premultiplied) {
            Some(dst)
        } else {
            None
        }
    }

    #[test]
    fn assembly_copies_contiguous_rows() {
        let src = vec![7u8; 4 * 3];
        let data = assemble(&src, 3, 1, 12, false).expect("assembly");
        assert_eq!(data, src);
    }

    #[test]
    fn assembly_strips_stride_padding() {
        let src = [
            0xAA, 0xBB, 0xCC, 0xDD, 0x01, 0x02, 0x03, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x05, 0x06, 0x07, 0x08,
        ];
        let data = assemble(&src, 2, 2, 16, false).expect("assembly");
        assert_eq!(data.len(), 16);
        assert_eq!(&data[..8], &src[..8]);
        assert_eq!(data[8..], src[16..24]);
    }

    #[test]
    fn assembly_forces_opaque_alpha_for_premultiplied_xrgb() {
        let src = [0x10, 0x20, 0x30, 0x00, 0x40, 0x50, 0x60, 0x7F];
        let data = assemble(&src, 2, 1, 8, true).expect("assembly");
        assert_eq!(data[3], 0xFF);
        assert_eq!(data[7], 0xFF);
        assert_eq!(&data[..3], &src[..3]);
        assert_eq!(&data[4..7], &src[4..7]);
    }

    #[test]
    fn assembly_keeps_alpha_for_argb_source() {
        let src = [0x10, 0x20, 0x30, 0x7F];
        let data = assemble(&src, 1, 1, 4, false).expect("assembly");
        assert_eq!(data[3], 0x7F);
    }

    #[test]
    fn assembly_rejects_short_buffers() {
        let src = vec![0u8; 7];
        assert!(assemble(&src, 2, 1, 8, false).is_none());
        assert!(assemble(&src, 1, 2, 4, false).is_none());
    }

    #[test]
    fn assembly_handles_zero_sized_frames() {
        let data = assemble(&[], 0, 0, 0, true).expect("assembly");
        assert!(data.is_empty());
    }
}

// Orbiscreen - aoa.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

#![allow(unsafe_code)]

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info, warn};

#[repr(C)]
struct UsbDevFsCtrlTransfer {
    request_type: u8,
    request: u8,
    value: u16,
    index: u16,
    length: u16,
    timeout: u32,
    data: *mut u8,
}

#[repr(C)]
struct UsbDevFsBulkTransfer {
    ep: u32,
    len: u32,
    timeout: u32,
    _pad: u32,
    data: *mut u8,
}

#[repr(C)]
struct UsbDevFsDisconnectClaim {
    interface: u32,
    flags: u32,
    driver: [u8; 256],
}

extern "C" {
    fn ioctl(fd: i32, request: u64, ...) -> i32;
}

const USBDEVFS_CONTROL: u64 = 0xc0185500;
const USBDEVFS_BULK: u64 = 0xc0185502;
const USBDEVFS_CLAIMINTERFACE: u64 = 0x8004550f;
const USBDEVFS_RELEASEINTERFACE: u64 = 0x80045510;
const USBDEVFS_DISCONNECT_CLAIM: u64 = 0x8108551b;

const AOA_GET_PROTOCOL: u8 = 51;
const AOA_SEND_STRING: u8 = 52;
const AOA_START_ACCESSORY: u8 = 53;

const FRAME_FLAG_DATA: u8 = 0x01;
const FRAME_FLAG_OPEN: u8 = 0x02;
const FRAME_FLAG_CLOSE: u8 = 0x04;
const FRAME_HEADER_LEN: usize = 5;
const MAX_PAYLOAD_LEN: usize = 16384;

#[derive(Clone, Debug)]
pub struct UsbDeviceInfo {
    pub bus_num: u16,
    pub dev_num: u16,
    pub vendor_id: u16,
    pub product_id: u16,
    pub sysfs_path: PathBuf,
    pub dev_node: PathBuf,
}

pub fn scan_usb_devices() -> Vec<UsbDeviceInfo> {
    let mut devices = Vec::new();
    let sys_usb = Path::new("/sys/bus/usb/devices");
    let entries = match std::fs::read_dir(sys_usb) {
        Ok(e) => e,
        Err(_) => return devices,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.contains(':') {
            continue;
        }

        let vid_str = match std::fs::read_to_string(path.join("idVendor")) {
            Ok(s) => s.trim().to_owned(),
            Err(_) => continue,
        };
        let pid_str = match std::fs::read_to_string(path.join("idProduct")) {
            Ok(s) => s.trim().to_owned(),
            Err(_) => continue,
        };
        let bus_str = match std::fs::read_to_string(path.join("busnum")) {
            Ok(s) => s.trim().to_owned(),
            Err(_) => continue,
        };
        let dev_str = match std::fs::read_to_string(path.join("devnum")) {
            Ok(s) => s.trim().to_owned(),
            Err(_) => continue,
        };

        let vid = match u16::from_str_radix(&vid_str, 16) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let pid = match u16::from_str_radix(&pid_str, 16) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let bus = match bus_str.parse::<u16>() {
            Ok(b) => b,
            Err(_) => continue,
        };
        let dev = match dev_str.parse::<u16>() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let dev_node = PathBuf::from(format!("/dev/bus/usb/{bus:03}/{dev:03}"));
        devices.push(UsbDeviceInfo {
            bus_num: bus,
            dev_num: dev,
            vendor_id: vid,
            product_id: pid,
            sysfs_path: path,
            dev_node,
        });
    }

    devices
}

pub fn is_google_accessory(vid: u16, pid: u16) -> bool {
    vid == 0x18d1 && matches!(pid, 0x2d00 | 0x2d01 | 0x2d04 | 0x2d05)
}

pub fn is_android_candidate(dev: &UsbDeviceInfo) -> bool {
    if is_google_accessory(dev.vendor_id, dev.product_id) {
        return false;
    }
    const ANDROID_VENDORS: &[u16] = &[
        0x18d1, 0x17ef, 0x2717, 0x04e8, 0x12d1, 0x22b8, 0x22d9, 0x2d95, 0x1004, 0x0fce, 0x0b05,
        0x0bb4, 0x19d2, 0x2a45, 0x2a70, 0x2833, 0x1949,
    ];
    if ANDROID_VENDORS.contains(&dev.vendor_id) {
        return true;
    }
    let prod = std::fs::read_to_string(dev.sysfs_path.join("product"))
        .unwrap_or_default()
        .to_lowercase();
    let mfg = std::fs::read_to_string(dev.sysfs_path.join("manufacturer"))
        .unwrap_or_default()
        .to_lowercase();
    prod.contains("android")
        || prod.contains("phone")
        || prod.contains("tablet")
        || prod.contains("pad")
        || mfg.contains("android")
}

fn ctrl_transfer(
    fd: i32,
    req_type: u8,
    req: u8,
    value: u16,
    index: u16,
    buf: &mut [u8],
    timeout_ms: u32,
) -> i32 {
    let mut ctrl = UsbDevFsCtrlTransfer {
        request_type: req_type,
        request: req,
        value,
        index,
        length: buf.len() as u16,
        timeout: timeout_ms,
        data: buf.as_mut_ptr(),
    };
    unsafe { ioctl(fd, USBDEVFS_CONTROL, &mut ctrl) }
}

pub fn probe_aoa_protocol(fd: i32) -> Option<u16> {
    let mut buf = [0u8; 2];
    let res = ctrl_transfer(fd, 0xC0, AOA_GET_PROTOCOL, 0, 0, &mut buf, 500);
    if res == 2 {
        let version = (buf[0] as u16) | ((buf[1] as u16) << 8);
        if version >= 1 {
            return Some(version);
        }
    }
    None
}

pub fn send_aoa_string(fd: i32, index: u16, s: &str) -> bool {
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(0);
    let res = ctrl_transfer(fd, 0x40, AOA_SEND_STRING, 0, index, &mut bytes, 500);
    res >= 0
}

pub fn start_aoa(fd: i32) -> bool {
    let mut dummy = [0u8; 0];
    let res = ctrl_transfer(fd, 0x40, AOA_START_ACCESSORY, 0, 0, &mut dummy, 500);
    res >= 0
}

pub fn initiate_aoa_handshake(device: &UsbDeviceInfo) -> bool {
    let f = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(&device.dev_node)
    {
        Ok(f) => f,
        Err(_) => return false,
    };
    let fd = f.as_raw_fd();

    let version = match probe_aoa_protocol(fd) {
        Some(v) => v,
        None => return false,
    };
    info!(
        "Detected AOA capable Android device {:04x}:{:04x} (AOA v{}) at {:?}",
        device.vendor_id, device.product_id, version, device.dev_node
    );

    let serial = std::fs::read_to_string(device.sysfs_path.join("serial"))
        .unwrap_or_else(|_| "orbiscreen".into())
        .trim()
        .to_owned();

    send_aoa_string(fd, 0, "shadow-x78");
    send_aoa_string(fd, 1, "Orbiscreen");
    send_aoa_string(fd, 2, "Orbiscreen Display Server");
    send_aoa_string(fd, 3, env!("CARGO_PKG_VERSION"));
    send_aoa_string(fd, 4, "https://github.com/shadow-x78/orbiscreen");
    send_aoa_string(fd, 5, &serial);

    info!(
        "Triggering AOA switch to accessory mode on {:?}",
        device.dev_node
    );
    start_aoa(fd)
}

fn detect_endpoints(sysfs_path: &Path) -> (u32, u32) {
    let mut in_ep = 0x81;
    let mut out_ep = 0x02;

    if let Ok(entries) = std::fs::read_dir(sysfs_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && entry.file_name().to_string_lossy().contains(':') {
                if let Ok(ep_entries) = std::fs::read_dir(&path) {
                    for ep_entry in ep_entries.flatten() {
                        let name = ep_entry.file_name();
                        let name_str = name.to_string_lossy();
                        if let Some(hex_str) = name_str.strip_prefix("ep_") {
                            if let Ok(ep_val) = u32::from_str_radix(hex_str, 16) {
                                if ep_val >= 0x80 {
                                    in_ep = ep_val;
                                } else if ep_val > 0 {
                                    out_ep = ep_val;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (in_ep, out_ep)
}

pub fn run_accessory_bridge(
    device: &UsbDeviceInfo,
    daemon_port: u16,
    running: Arc<AtomicBool>,
) -> Result<(), String> {
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&device.dev_node)
        .map_err(|e| {
            format!(
                "Failed to open accessory dev node {:?}: {e}",
                device.dev_node
            )
        })?;
    let fd = f.as_raw_fd();

    let iface = 0u32;
    let mut claimed = false;
    let mut last_err = std::io::Error::last_os_error();

    for attempt in 0..3 {
        let mut dc = UsbDevFsDisconnectClaim {
            interface: iface,
            flags: 0,
            driver: [0u8; 256],
        };
        let mut res = unsafe { ioctl(fd, USBDEVFS_DISCONNECT_CLAIM, &mut dc) };
        if res < 0 {
            res = unsafe { ioctl(fd, USBDEVFS_CLAIMINTERFACE, &iface) };
        }
        if res >= 0 {
            claimed = true;
            break;
        }
        last_err = std::io::Error::last_os_error();
        if attempt < 2 {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    if !claimed {
        return Err(format!("Failed to claim interface 0: {}", last_err));
    }

    let (in_ep, out_ep) = detect_endpoints(&device.sysfs_path);
    info!(
        "AOA accessory claimed on {:?}, in_ep=0x{:02x}, out_ep=0x{:02x}",
        device.dev_node, in_ep, out_ep
    );

    let (prio_tx, prio_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let (video_tx, video_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let running_writer = running.clone();
    let fd_writer = fd;
    let writer_handle = std::thread::spawn(move || {
        while running_writer.load(Ordering::Relaxed) {
            let chunk = match prio_rx.try_recv() {
                Ok(c) => c,
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    match video_rx.recv_timeout(Duration::from_millis(50)) {
                        Ok(c) => c,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            };

            let mut offset = 0;
            while offset < chunk.len() && running_writer.load(Ordering::Relaxed) {
                let to_write = std::cmp::min(chunk.len() - offset, MAX_PAYLOAD_LEN);
                let mut bulk = UsbDevFsBulkTransfer {
                    ep: out_ep,
                    len: to_write as u32,
                    timeout: 100,
                    _pad: 0,
                    data: chunk[offset..].as_ptr() as *mut u8,
                };
                let written = unsafe { ioctl(fd_writer, USBDEVFS_BULK, &mut bulk) };
                if written < 0 {
                    let err = std::io::Error::last_os_error();
                    debug!("USB bulk write error: {err}");
                    break;
                }
                offset += written as usize;
            }
        }
    });

    type StreamEntry = (std::sync::mpsc::Sender<Vec<u8>>, TcpStream, Arc<AtomicBool>);
    type StreamMap = Arc<Mutex<HashMap<u16, StreamEntry>>>;

    let tcp_streams: StreamMap = Arc::new(Mutex::new(HashMap::new()));
    let mut rx_buf = vec![0u8; MAX_PAYLOAD_LEN];
    let mut acc_buf = Vec::new();

    while running.load(Ordering::Relaxed) {
        let mut bulk = UsbDevFsBulkTransfer {
            ep: in_ep,
            len: rx_buf.len() as u32,
            timeout: 500,
            _pad: 0,
            data: rx_buf.as_mut_ptr(),
        };
        let read_bytes = unsafe { ioctl(fd, USBDEVFS_BULK, &mut bulk) };
        if read_bytes < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(110) {
                continue;
            }
            warn!("USB bulk read error on accessory: {err}");
            break;
        }

        if read_bytes > 0 {
            acc_buf.extend_from_slice(&rx_buf[..read_bytes as usize]);

            while acc_buf.len() >= FRAME_HEADER_LEN {
                let stream_id = u16::from_be_bytes([acc_buf[0], acc_buf[1]]);
                let flags = acc_buf[2];
                let payload_len = u16::from_be_bytes([acc_buf[3], acc_buf[4]]) as usize;
                let total_frame_len = FRAME_HEADER_LEN + payload_len;

                if acc_buf.len() < total_frame_len {
                    break;
                }

                let payload = acc_buf[FRAME_HEADER_LEN..total_frame_len].to_vec();
                acc_buf.drain(..total_frame_len);

                if (flags & FRAME_FLAG_OPEN) != 0 {
                    let addr = format!("127.0.0.1:{daemon_port}");
                    match TcpStream::connect(&addr) {
                        Ok(mut tcp_stream) => {
                            let _ = tcp_stream.set_nodelay(true);
                            let _ = tcp_stream.set_read_timeout(Some(Duration::from_millis(1500)));
                            let (tcp_tx, tcp_rx) = std::sync::mpsc::channel::<Vec<u8>>();
                            let is_video = Arc::new(AtomicBool::new(false));
                            if let Ok(stream_for_map) = tcp_stream.try_clone() {
                                let mut map = tcp_streams.lock().unwrap();
                                map.insert(stream_id, (tcp_tx, stream_for_map, is_video.clone()));
                            }

                            let prio_tx_clone = prio_tx.clone();
                            let video_tx_clone = video_tx.clone();
                            let running_tcp = running.clone();
                            let tcp_streams_reader = tcp_streams.clone();
                            let mut tcp_read_stream = match tcp_stream.try_clone() {
                                Ok(s) => s,
                                Err(_) => continue,
                            };

                            std::thread::spawn(move || {
                                let mut buf = vec![0u8; 8192];
                                while running_tcp.load(Ordering::Relaxed) {
                                    match tcp_read_stream.read(&mut buf) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            let mut frame =
                                                Vec::with_capacity(FRAME_HEADER_LEN + n);
                                            frame.extend_from_slice(&stream_id.to_be_bytes());
                                            frame.push(FRAME_FLAG_DATA);
                                            frame.extend_from_slice(&(n as u16).to_be_bytes());
                                            frame.extend_from_slice(&buf[..n]);
                                            if is_video.load(Ordering::Relaxed) {
                                                if video_tx_clone.send(frame).is_err() {
                                                    break;
                                                }
                                            } else if prio_tx_clone.send(frame).is_err() {
                                                break;
                                            }
                                        }
                                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                                            continue
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock =>
                                        {
                                            continue
                                        }
                                        Err(_) => break,
                                    }
                                }
                                let mut close_frame = Vec::with_capacity(FRAME_HEADER_LEN);
                                close_frame.extend_from_slice(&stream_id.to_be_bytes());
                                close_frame.push(FRAME_FLAG_CLOSE);
                                close_frame.extend_from_slice(&0u16.to_be_bytes());
                                let _ = prio_tx_clone.send(close_frame);
                                let mut map = tcp_streams_reader.lock().unwrap();
                                map.remove(&stream_id);
                            });

                            let running_writer = running.clone();
                            std::thread::spawn(move || {
                                while running_writer.load(Ordering::Relaxed) {
                                    match tcp_rx.recv_timeout(Duration::from_millis(200)) {
                                        Ok(bytes) => {
                                            if tcp_stream.write_all(&bytes).is_err() {
                                                break;
                                            }
                                        }
                                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                                            break
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            debug!("Failed to connect to local daemon at {addr}: {e}");
                            let mut close_frame = Vec::with_capacity(FRAME_HEADER_LEN);
                            close_frame.extend_from_slice(&stream_id.to_be_bytes());
                            close_frame.push(FRAME_FLAG_CLOSE);
                            close_frame.extend_from_slice(&0u16.to_be_bytes());
                            let _ = prio_tx.send(close_frame);
                        }
                    }
                } else if (flags & FRAME_FLAG_DATA) != 0 {
                    let map = tcp_streams.lock().unwrap();
                    if let Some((tx, _, is_video)) = map.get(&stream_id) {
                        if payload.windows(7).any(|w| w == b"/stream") {
                            is_video.store(true, Ordering::Relaxed);
                        }
                        let _ = tx.send(payload);
                    }
                } else if (flags & FRAME_FLAG_CLOSE) != 0 {
                    let mut map = tcp_streams.lock().unwrap();
                    if let Some((_, stream, _)) = map.remove(&stream_id) {
                        let _ = stream.shutdown(std::net::Shutdown::Both);
                    }
                }
            }
        }
    }

    running.store(false, Ordering::Relaxed);
    {
        let mut map = tcp_streams.lock().unwrap();
        for (_, (_, stream, _)) in map.drain() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }
    let _ = writer_handle.join();
    unsafe { ioctl(fd, USBDEVFS_RELEASEINTERFACE, &iface) };
    info!("AOA accessory released on {:?}", device.dev_node);
    Ok(())
}

pub async fn supervisor(
    daemon_port: u16,
    active_count: Arc<AtomicUsize>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let mut tried_devices: Vec<(u16, u16)> = Vec::new();

    loop {
        let devices = scan_usb_devices();
        let mut accessory_device = None;

        for dev in &devices {
            if is_google_accessory(dev.vendor_id, dev.product_id) {
                accessory_device = Some(dev.clone());
                break;
            }
        }

        if let Some(acc) = accessory_device {
            info!("AOA accessory device detected: {:?}", acc.dev_node);
            active_count.store(1, Ordering::Relaxed);
            let running = Arc::new(AtomicBool::new(true));
            let running_inner = running.clone();
            let mut shutdown_inner = shutdown.clone();

            let bridge_task = tokio::task::spawn_blocking(move || {
                run_accessory_bridge(&acc, daemon_port, running_inner)
            });

            let mut permission_error = false;
            tokio::select! {
                _ = shutdown_inner.changed() => {
                    running.store(false, Ordering::Relaxed);
                }
                res = bridge_task => {
                    if let Ok(Err(e)) = res {
                        if e.contains("Permission denied") {
                            permission_error = true;
                            warn!("AOA USB node requires non-root permissions: run 'orbiscreen doctor --fix' once, or use USB Tethering on Android for 100% root-free streaming");
                        } else {
                            warn!("AOA bridge exited: {e}");
                        }
                    }
                }
            }

            active_count.store(0, Ordering::Relaxed);
            tried_devices.clear();
            let sleep_duration = if permission_error {
                Duration::from_secs(8)
            } else {
                Duration::from_secs(2)
            };
            tokio::time::sleep(sleep_duration).await;
        } else {
            for dev in &devices {
                if dev.vendor_id == 0x1d6b || !is_android_candidate(dev) {
                    continue;
                }
                let pair = (dev.vendor_id, dev.product_id);
                if tried_devices.contains(&pair) {
                    continue;
                }

                let dev_clone = dev.clone();
                let switched =
                    tokio::task::spawn_blocking(move || initiate_aoa_handshake(&dev_clone))
                        .await
                        .unwrap_or(false);

                tried_devices.push(pair);
                if switched {
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    break;
                }
            }
        }

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
            _ = shutdown.changed() => break,
        }
    }
}

// Orbiscreen - orbiscreen-capture - wlroots virtual output module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::collections::HashSet;
use std::io::{Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use thiserror::Error;

const SWAY_IPC_MAGIC: &[u8; 6] = b"i3-ipc";
const SWAY_MSG_RUN_COMMAND: u32 = 0;
const SWAY_MSG_GET_OUTPUTS: u32 = 3;
const IPC_TIMEOUT: Duration = Duration::from_secs(3);
const APPEAR_DEADLINE: Duration = Duration::from_secs(5);
const APPEAR_POLL: Duration = Duration::from_millis(50);
const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WlrootsIpcKind {
    Sway,
    Hyprland,
}

impl std::fmt::Display for WlrootsIpcKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sway => write!(f, "sway"),
            Self::Hyprland => write!(f, "hyprland"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VirtualOutputSpec {
    pub width: u32,
    pub height: u32,
    pub refresh_rate_hz: u32,
}

#[derive(Debug, Clone)]
pub struct OutputSnapshot {
    pub name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Error)]
pub enum WlrootsVirtualOutputError {
    #[error("no wlroots compositor IPC found — neither a sway socket ($SWAYSOCK) nor a Hyprland socket is available")]
    IpcUnavailable,
    #[error("sway IPC error: {0}")]
    Sway(String),
    #[error("hyprland IPC error: {0}")]
    Hyprland(String),
    #[error("the compositor rejected the command: {0}")]
    Rejected(String),
    #[error("the virtual output did not appear within {0:.1}s")]
    Timeout(f64),
}

#[derive(Debug, Clone)]
enum Ipc {
    Sway(PathBuf),
    Hyprland(PathBuf),
}

impl Ipc {
    fn detect() -> Option<Self> {
        if let Some(sock) = std::env::var("SWAYSOCK")
            .ok()
            .filter(|v| !v.trim().is_empty())
        {
            let path = PathBuf::from(sock);
            if path.exists() {
                return Some(Self::Sway(path));
            }
        }
        if let Some(sig) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .ok()
            .filter(|v| !v.trim().is_empty())
        {
            if let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR") {
                let path = PathBuf::from(runtime)
                    .join("hypr")
                    .join(sig)
                    .join(".socket.sock");
                if path.exists() {
                    return Some(Self::Hyprland(path));
                }
            }
        }
        None
    }

    fn kind(&self) -> WlrootsIpcKind {
        match self {
            Self::Sway(_) => WlrootsIpcKind::Sway,
            Self::Hyprland(_) => WlrootsIpcKind::Hyprland,
        }
    }

    fn list_outputs(&self) -> Result<Vec<OutputSnapshot>, WlrootsVirtualOutputError> {
        match self {
            Self::Sway(sock) => {
                let response = sway_roundtrip(sock, SWAY_MSG_GET_OUTPUTS, "")?;
                parse_sway_outputs(&response)
            }
            Self::Hyprland(sock) => {
                let response = hyprland_roundtrip(sock, "j/monitors")?;
                parse_hyprland_monitors(&response)
            }
        }
    }

    fn create(&self, spec: VirtualOutputSpec) -> Result<(), WlrootsVirtualOutputError> {
        match self {
            Self::Sway(sock) => {
                let response = sway_roundtrip(sock, SWAY_MSG_RUN_COMMAND, "create output")?;
                parse_sway_command_success(&response)
            }
            Self::Hyprland(sock) => {
                let w = spec.width;
                let h = spec.height;
                let hz = spec.refresh_rate_hz;
                let attempts = [
                    format!("output create headless {w}x{h}@{hz}"),
                    format!("output create headless {w}x{h}"),
                    "output create headless".to_string(),
                ];
                for attempt in attempts {
                    let response = hyprland_roundtrip(sock, &attempt)?;
                    if response.trim() == "ok" {
                        return Ok(());
                    }
                    tracing::debug!("hyprland rejected {attempt:?}: {}", response.trim());
                }
                Err(WlrootsVirtualOutputError::Rejected(
                    "output create headless was rejected by hyprland (hyprland >= 0.44 required)"
                        .to_string(),
                ))
            }
        }
    }

    fn set_mode_best_effort(&self, name: &str, spec: VirtualOutputSpec) {
        if let Self::Sway(sock) = self {
            if !output_name_is_safe(name) {
                tracing::warn!(output = name, "refusing unsafe output name in mode command");
                return;
            }
            let cmd = format!(
                "output {name} mode {}x{}@{}",
                spec.width, spec.height, spec.refresh_rate_hz
            );
            match sway_roundtrip(sock, SWAY_MSG_RUN_COMMAND, &cmd)
                .and_then(|response| parse_sway_command_success(&response))
            {
                Ok(()) => tracing::info!(output = name, "virtual output mode set"),
                Err(e) => tracing::debug!(
                    output = name,
                    "mode not changed ({e}); keeping the compositor default"
                ),
            }
        }
    }

    fn remove(&self, name: &str) -> Result<(), WlrootsVirtualOutputError> {
        if !output_name_is_safe(name) {
            return Err(WlrootsVirtualOutputError::Rejected(format!(
                "refusing unsafe output name {name:?}"
            )));
        }
        match self {
            Self::Sway(sock) => {
                let cmd = format!("output {name} remove");
                let response = sway_roundtrip(sock, SWAY_MSG_RUN_COMMAND, &cmd)?;
                parse_sway_command_success(&response)
            }
            Self::Hyprland(sock) => {
                let response = hyprland_roundtrip(sock, &format!("output destroy {name}"))?;
                if response.trim() == "ok" {
                    Ok(())
                } else {
                    Err(WlrootsVirtualOutputError::Hyprland(
                        response.trim().to_string(),
                    ))
                }
            }
        }
    }
}

fn output_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn encode_sway_message(msg_type: u32, payload: &str) -> Vec<u8> {
    let payload = payload.as_bytes();
    let mut buf = Vec::with_capacity(SWAY_IPC_MAGIC.len() + 8 + payload.len());
    buf.extend_from_slice(SWAY_IPC_MAGIC);
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(&msg_type.to_le_bytes());
    buf.extend_from_slice(payload);
    buf
}

fn sway_roundtrip(
    socket: &Path,
    msg_type: u32,
    payload: &str,
) -> Result<String, WlrootsVirtualOutputError> {
    let to_sway = |e: std::io::Error| WlrootsVirtualOutputError::Sway(e.to_string());
    let mut stream = UnixStream::connect(socket).map_err(to_sway)?;
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .map_err(to_sway)?;
    stream
        .set_write_timeout(Some(IPC_TIMEOUT))
        .map_err(to_sway)?;
    stream
        .write_all(&encode_sway_message(msg_type, payload))
        .map_err(to_sway)?;

    let mut header = [0u8; 14];
    read_exact(&mut stream, &mut header).map_err(to_sway)?;
    if &header[..6] != SWAY_IPC_MAGIC {
        return Err(WlrootsVirtualOutputError::Sway(
            "response is missing the i3-ipc magic".to_string(),
        ));
    }
    let len = u32::from_le_bytes(header[6..10].try_into().expect("4-byte length")) as usize;
    if len > MAX_RESPONSE_BYTES {
        return Err(WlrootsVirtualOutputError::Sway(format!(
            "oversized IPC response ({len} bytes)"
        )));
    }
    let mut body = vec![0u8; len];
    read_exact(&mut stream, &mut body).map_err(to_sway)?;
    String::from_utf8(body).map_err(|e| WlrootsVirtualOutputError::Sway(e.to_string()))
}

fn read_exact(stream: &mut UnixStream, buf: &mut [u8]) -> std::io::Result<()> {
    stream.read_exact(buf)
}

fn parse_sway_command_success(json: &str) -> Result<(), WlrootsVirtualOutputError> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| WlrootsVirtualOutputError::Sway(format!("bad command reply: {e}")))?;
    let results = value
        .as_array()
        .ok_or_else(|| WlrootsVirtualOutputError::Sway("command reply is not a list".into()))?;
    for result in results {
        let success = result
            .get("success")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);
        if !success {
            let error = result
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown error");
            return Err(WlrootsVirtualOutputError::Rejected(error.to_string()));
        }
    }
    if results.is_empty() {
        return Err(WlrootsVirtualOutputError::Sway(
            "empty command reply".into(),
        ));
    }
    Ok(())
}

fn parse_sway_outputs(json: &str) -> Result<Vec<OutputSnapshot>, WlrootsVirtualOutputError> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| WlrootsVirtualOutputError::Sway(format!("bad outputs reply: {e}")))?;
    let list = value
        .as_array()
        .ok_or_else(|| WlrootsVirtualOutputError::Sway("outputs reply is not a list".into()))?;
    let mut outputs = Vec::with_capacity(list.len());
    for item in list {
        let Some(name) = item.get("name").and_then(|n| n.as_str()) else {
            continue;
        };
        let rect = item.get("rect");
        let width = rect
            .and_then(|r| r.get("width"))
            .and_then(|w| w.as_u64())
            .unwrap_or(0) as u32;
        let height = rect
            .and_then(|r| r.get("height"))
            .and_then(|h| h.as_u64())
            .unwrap_or(0) as u32;
        outputs.push(OutputSnapshot {
            name: name.to_string(),
            width,
            height,
        });
    }
    Ok(outputs)
}

fn hyprland_roundtrip(socket: &Path, request: &str) -> Result<String, WlrootsVirtualOutputError> {
    let to_hypr = |e: std::io::Error| WlrootsVirtualOutputError::Hyprland(e.to_string());
    let mut stream = UnixStream::connect(socket).map_err(to_hypr)?;
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .map_err(to_hypr)?;
    stream.write_all(request.as_bytes()).map_err(to_hypr)?;
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if buf.len() > MAX_RESPONSE_BYTES {
                    return Err(WlrootsVirtualOutputError::Hyprland(
                        "oversized IPC response".to_string(),
                    ));
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::TimedOut
                    || e.kind() == std::io::ErrorKind::WouldBlock =>
            {
                break
            }
            Err(e) => return Err(WlrootsVirtualOutputError::Hyprland(e.to_string())),
        }
    }
    String::from_utf8(buf).map_err(|e| WlrootsVirtualOutputError::Hyprland(e.to_string()))
}

fn parse_hyprland_monitors(json: &str) -> Result<Vec<OutputSnapshot>, WlrootsVirtualOutputError> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| WlrootsVirtualOutputError::Hyprland(format!("bad monitors reply: {e}")))?;
    let list = value.as_array().ok_or_else(|| {
        WlrootsVirtualOutputError::Hyprland("monitors reply is not a list".into())
    })?;
    let mut outputs = Vec::with_capacity(list.len());
    for item in list {
        let Some(name) = item.get("name").and_then(|n| n.as_str()) else {
            continue;
        };
        let width = item.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
        let height = item.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
        outputs.push(OutputSnapshot {
            name: name.to_string(),
            width,
            height,
        });
    }
    Ok(outputs)
}

pub fn detect_ipc_kind() -> Option<WlrootsIpcKind> {
    Ipc::detect().map(|ipc| ipc.kind())
}

#[derive(Debug)]
pub struct WlrootsVirtualOutput {
    ipc: Ipc,
    name: String,
    width: u32,
    height: u32,
}

impl WlrootsVirtualOutput {
    pub fn create(spec: VirtualOutputSpec) -> Result<Self, WlrootsVirtualOutputError> {
        let ipc = Ipc::detect().ok_or(WlrootsVirtualOutputError::IpcUnavailable)?;
        let before: HashSet<String> = ipc.list_outputs()?.into_iter().map(|o| o.name).collect();
        ipc.create(spec)?;

        let deadline = Instant::now() + APPEAR_DEADLINE;
        let mut mode_attempted = false;
        loop {
            for output in ipc.list_outputs()? {
                if before.contains(&output.name) {
                    continue;
                }
                if !mode_attempted {
                    ipc.set_mode_best_effort(&output.name, spec);
                    mode_attempted = true;
                } else if output.width > 0 && output.height > 0 {
                    tracing::info!(
                        output = output.name,
                        width = output.width,
                        height = output.height,
                        "wlroots virtual output created via compositor IPC — no root, no dialog"
                    );
                    return Ok(Self {
                        ipc,
                        name: output.name,
                        width: output.width,
                        height: output.height,
                    });
                }
            }
            if Instant::now() >= deadline {
                return Err(WlrootsVirtualOutputError::Timeout(
                    APPEAR_DEADLINE.as_secs_f64(),
                ));
            }
            std::thread::sleep(APPEAR_POLL);
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for WlrootsVirtualOutput {
    fn drop(&mut self) {
        match self.ipc.remove(&self.name) {
            Ok(()) => tracing::info!(output = self.name, "wlroots virtual output removed"),
            Err(e) => tracing::warn!(
                output = self.name,
                "failed to remove wlroots virtual output: {e}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sway_message_is_i3_ipc_framed() {
        let msg = encode_sway_message(SWAY_MSG_RUN_COMMAND, "create output");
        assert_eq!(&msg[..6], b"i3-ipc");
        assert_eq!(u32::from_le_bytes(msg[6..10].try_into().unwrap()), 13);
        assert_eq!(u32::from_le_bytes(msg[10..14].try_into().unwrap()), 0);
        assert_eq!(&msg[14..], b"create output");
    }

    #[test]
    fn sway_success_reply_is_accepted() {
        parse_sway_command_success(r#"[{"success": true}]"#).expect("accepted");
        parse_sway_command_success(r#"[{"success": true}, {"success": true}]"#)
            .expect("accepted multi");
    }

    #[test]
    fn sway_failure_reply_carries_error() {
        let err = parse_sway_command_success(
            r#"[{"success": false, "error": "Cannot create any more outputs"}]"#,
        )
        .expect_err("rejected");
        assert!(err.to_string().contains("Cannot create any more outputs"));
    }

    #[test]
    fn sway_empty_reply_is_an_error() {
        assert!(parse_sway_command_success("[]").is_err());
        assert!(parse_sway_command_success("not json").is_err());
    }

    #[test]
    fn sway_outputs_reply_is_parsed() {
        let json = r#"[
            {"name": "eDP-1", "active": true, "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080}},
            {"name": "HEADLESS-1", "active": true, "rect": {"x": 1920, "y": 0, "width": 1280, "height": 720}},
            {"no_name": true}
        ]"#;
        let outputs = parse_sway_outputs(json).expect("parse");
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].name, "eDP-1");
        assert_eq!((outputs[0].width, outputs[0].height), (1920, 1080));
        assert_eq!(outputs[1].name, "HEADLESS-1");
        assert_eq!((outputs[1].width, outputs[1].height), (1280, 720));
    }

    #[test]
    fn hyprland_monitors_reply_is_parsed() {
        let json = r#"[
            {"id": 0, "name": "eDP-1", "width": 2560, "height": 1440, "refreshRate": 144.0},
            {"id": 1, "name": "HEADLESS-A-1", "width": 1920, "height": 1080, "refreshRate": 60.0}
        ]"#;
        let outputs = parse_hyprland_monitors(json).expect("parse");
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[1].name, "HEADLESS-A-1");
        assert_eq!((outputs[1].width, outputs[1].height), (1920, 1080));
    }

    #[test]
    fn hyprland_bad_reply_is_reported() {
        assert!(parse_hyprland_monitors("unknown request").is_err());
        assert!(parse_hyprland_monitors(r#"{"oops": 1}"#).is_err());
    }

    #[test]
    fn output_names_with_only_safe_chars_pass() {
        assert!(output_name_is_safe("HEADLESS-1"));
        assert!(output_name_is_safe("eDP-1"));
        assert!(output_name_is_safe("DP-2.1_x"));
    }

    #[test]
    fn output_names_with_command_chars_are_rejected() {
        assert!(!output_name_is_safe(""));
        assert!(!output_name_is_safe("x remove"));
        assert!(!output_name_is_safe("a;exec foo"));
        assert!(!output_name_is_safe("name;exec"));
        assert!(!output_name_is_safe("a,b"));
        assert!(!output_name_is_safe(&"n".repeat(65)));
    }
}

// Orbiscreen - adb.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::path::Path;
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdbError {
    #[error("`adb` binary not found in PATH")]
    NotInstalled,
    #[error("adb command failed: {0}")]
    Failed(String),
    #[error("no adb device is currently connected over USB")]
    NoDevice,
}

fn device_serials(out: &str) -> Vec<&str> {
    out.lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let serial = parts.next()?;
            let state = parts.next()?;
            (state == "device" && serial_is_safe(serial)).then_some(serial)
        })
        .collect()
}

fn serial_is_safe(serial: &str) -> bool {
    !serial.is_empty()
        && serial.len() <= 128
        && serial
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | ':' | '_' | '-'))
}

pub fn reverse_port(adb_path: &Path, device: &str, host_port: u16) -> Result<(), AdbError> {
    let port = format!("tcp:{host_port}");
    let out = Command::new(adb_path)
        .args(["-s", device, "reverse", &port, &port])
        .output()
        .map_err(spawn_error)?;
    if !out.status.success() {
        return Err(AdbError::Failed(format!(
            "adb reverse exited with {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

pub fn remove_reverse(adb_path: &Path, device: &str, host_port: u16) -> Result<(), AdbError> {
    let out = Command::new(adb_path)
        .args([
            "-s",
            device,
            "reverse",
            "--remove",
            &format!("tcp:{host_port}"),
        ])
        .output()
        .map_err(spawn_error)?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_owned();
        if stderr.contains("listener") && stderr.contains("not found") {
            return Ok(());
        }
        return Err(AdbError::Failed(format!(
            "adb reverse --remove exited with {}: {stderr}",
            out.status
        )));
    }
    Ok(())
}

fn spawn_error(e: std::io::Error) -> AdbError {
    if e.kind() == std::io::ErrorKind::NotFound {
        AdbError::NotInstalled
    } else {
        AdbError::Failed(e.to_string())
    }
}

pub fn connect_arc_device(adb_path: &Path) -> Result<String, AdbError> {
    let candidates = [
        "100.115.92.2:5555",
        "192.168.233.2:5555",
        "localhost:5555",
        "127.0.0.1:5555",
    ];
    for addr in candidates {
        if let Ok(out) = Command::new(adb_path).args(["connect", addr]).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("connected to") || stdout.contains("already connected") {
                return Ok(addr.to_string());
            }
        }
    }
    Err(AdbError::NoDevice)
}

pub fn setup_reverse_for_all(adb_path: &Path, host_port: u16) -> Result<Vec<String>, AdbError> {
    let out = Command::new(adb_path)
        .arg("devices")
        .output()
        .map_err(spawn_error)?;
    if !out.status.success() {
        return Err(AdbError::Failed(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut serials: Vec<String> = device_serials(&stdout)
        .into_iter()
        .map(str::to_owned)
        .collect();
    if serials.is_empty() {
        if let Ok(arc_serial) = connect_arc_device(adb_path) {
            serials.push(arc_serial);
        } else {
            return Err(AdbError::NoDevice);
        }
    }
    for serial in &serials {
        reverse_port(adb_path, serial, host_port)?;
    }
    Ok(serials)
}

pub fn teardown_reverse_for_all(adb_path: &Path, host_port: u16) -> Result<Vec<String>, AdbError> {
    let out = Command::new(adb_path)
        .arg("devices")
        .output()
        .map_err(spawn_error)?;
    if !out.status.success() {
        return Err(AdbError::Failed(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let serials: Vec<String> = device_serials(&stdout)
        .into_iter()
        .map(str::to_owned)
        .collect();
    if serials.is_empty() {
        return Err(AdbError::NoDevice);
    }
    for serial in &serials {
        remove_reverse(adb_path, serial, host_port)?;
    }
    Ok(serials)
}

fn parse_reverse_list(out: &str, host_port: u16) -> usize {
    let needle = format!("tcp:{host_port}");
    out.lines()
        .filter(|line| line.split_whitespace().next() == Some(needle.as_str()))
        .count()
}

pub fn reverse_tunnel_count(
    adb_path: &Path,
    device: &str,
    host_port: u16,
) -> Result<usize, AdbError> {
    let out = Command::new(adb_path)
        .args(["-s", device, "reverse", "--list"])
        .output()
        .map_err(spawn_error)?;
    if !out.status.success() {
        return Err(AdbError::Failed(format!(
            "adb reverse --list exited with {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(parse_reverse_list(
        &String::from_utf8_lossy(&out.stdout),
        host_port,
    ))
}

pub fn default_adb_path() -> &'static Path {
    Path::new("adb")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_authorized_devices() {
        let output = "List of devices attached\nAAAA\tdevice\nBBBB\tdevice\nCCCC\tunauthorized\n";
        assert_eq!(device_serials(output), vec!["AAAA", "BBBB"]);
    }

    #[test]
    fn default_adb_path_is_relative() {
        assert_eq!(default_adb_path(), Path::new("adb"));
    }

    #[test]
    fn counts_matching_reverse_tunnels() {
        let output = "tcp:8788 tcp:8788\ntcp:8788 tcp:8788\ntcp:9999 tcp:9999\n";
        assert_eq!(parse_reverse_list(output, 8788), 2);
        assert_eq!(parse_reverse_list(output, 9999), 1);
        assert_eq!(parse_reverse_list(output, 1234), 0);
    }

    #[test]
    fn reverse_list_ignores_malformed_lines() {
        let output = "tcp:8788 tcp:8788\n\nnot-a-tunnel\n";
        assert_eq!(parse_reverse_list(output, 8788), 1);
    }
}

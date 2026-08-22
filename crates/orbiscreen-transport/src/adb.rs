// Orbiscreen - orbiscreen-transport - adb module (GPL-3.0-or-later)
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
            (state == "device").then_some(serial)
        })
        .collect()
}

pub fn reverse_port(adb_path: &Path, device: &str, host_port: u16) -> Result<(), AdbError> {
    let port = format!("tcp:{host_port}");
    let status = Command::new(adb_path)
        .args(["-s", device, "reverse", &port, &port])
        .status()
        .map_err(|_| AdbError::NotInstalled)?;
    if !status.success() {
        return Err(AdbError::Failed(format!(
            "adb reverse exited with {status}"
        )));
    }
    Ok(())
}

pub fn setup_reverse_for_all(adb_path: &Path, host_port: u16) -> Result<Vec<String>, AdbError> {
    let out = Command::new(adb_path)
        .arg("devices")
        .output()
        .map_err(|_| AdbError::NotInstalled)?;
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
        reverse_port(adb_path, serial, host_port)?;
    }
    Ok(serials)
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
}

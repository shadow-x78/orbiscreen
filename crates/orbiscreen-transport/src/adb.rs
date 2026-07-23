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

fn first_device_serial(out: &str) -> Option<&str> {
    for line in out.lines().skip(1) {
        let mut parts = line.split_whitespace();
        let serial = parts.next()?;
        let state = parts.next()?;
        if state == "device" {
            return Some(serial);
        }
    }
    None
}

pub fn find_usb_device(adb_path: &Path) -> Result<String, AdbError> {
    let out = Command::new(adb_path)
        .arg("devices")
        .output()
        .map_err(|_| AdbError::NotInstalled)?;
    if !out.status.success() {
        return Err(AdbError::Failed(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    first_device_serial(&String::from_utf8_lossy(&out.stdout))
        .map(str::to_owned)
        .ok_or(AdbError::NoDevice)
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

pub fn remove_reverse(adb_path: &Path, device: &str, host_port: u16) -> Result<(), AdbError> {
    Command::new(adb_path)
        .args([
            "-s",
            device,
            "reverse",
            "--remove",
            &format!("tcp:{host_port}"),
        ])
        .status()
        .map_err(|_| AdbError::NotInstalled)?;
    Ok(())
}

pub fn setup_reverse_for_all(adb_path: &Path, host_port: u16) -> Result<Vec<String>, AdbError> {
    let out = Command::new(adb_path)
        .arg("devices")
        .output()
        .map_err(|_| AdbError::NotInstalled)?;
    let serials: Vec<String> = first_device_serial(&String::from_utf8_lossy(&out.stdout))
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
    fn parses_devices_output() {
        let output = "List of devices attached\nABCDEFGH\tdevice\nXYZ12345\tunauthorized\n";
        assert_eq!(first_device_serial(output), Some("ABCDEFGH"));
    }

    #[test]
    fn returns_none_when_no_authorised_device() {
        let output = "List of devices attached\nXYZ12345\tunauthorized\n";
        assert_eq!(first_device_serial(output), None);
    }

    #[test]
    fn default_adb_path_is_relative() {
        assert_eq!(default_adb_path(), Path::new("adb"));
    }
}

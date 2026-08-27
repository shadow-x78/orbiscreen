// Orbiscreen - orbiscreen-core library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PortalState {
    pub screencast_restore_token: Option<String>,
    pub remote_desktop_restore_token: Option<String>,
}

pub fn portal_state_path() -> PathBuf {
    portal_state_path_from(|key| std::env::var_os(key))
}

fn portal_state_path_from(mut lookup: impl FnMut(&str) -> Option<std::ffi::OsString>) -> PathBuf {
    if let Some(xdg) =
        lookup("XDG_STATE_HOME").filter(|v| !v.is_empty() && Path::new(&v).is_absolute())
    {
        return PathBuf::from(xdg).join("orbiscreen/portal.json");
    }
    if let Some(home) = lookup("HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(home).join(".local/state/orbiscreen/portal.json");
    }
    PathBuf::from("orbiscreen-portal.json")
}

pub fn load_portal_state() -> PortalState {
    load_portal_state_from(&portal_state_path())
}

pub fn save_portal_state(state: &PortalState) -> std::io::Result<()> {
    save_portal_state_to(state, &portal_state_path())
}

fn load_portal_state_from(path: &Path) -> PortalState {
    let Ok(content) = std::fs::read_to_string(path) else {
        return PortalState::default();
    };
    match serde_json::from_str::<PortalState>(&content) {
        Ok(state) => state,
        Err(e) => {
            tracing::warn!(path = %path.display(), "ignoring unreadable portal state: {e}");
            PortalState::default()
        }
    }
}

fn save_portal_state_to(state: &PortalState, path: &Path) -> std::io::Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    file.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::os::unix::fs::PermissionsExt as _;

    fn unique_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("orbiscreen-portal-{}-{tag}", std::process::id()))
    }

    #[test]
    fn path_prefers_xdg_state_home() {
        let mut vars: HashMap<&str, std::ffi::OsString> = HashMap::new();
        vars.insert("XDG_STATE_HOME", "/tmp/orbiscreen-state".into());
        vars.insert("HOME", "/home/u".into());
        assert_eq!(
            portal_state_path_from(|k| vars.get(k).cloned()),
            PathBuf::from("/tmp/orbiscreen-state/orbiscreen/portal.json")
        );
    }

    #[test]
    fn path_falls_back_to_home_local_state() {
        let mut vars: HashMap<&str, std::ffi::OsString> = HashMap::new();
        vars.insert("HOME", "/home/u".into());
        assert_eq!(
            portal_state_path_from(|k| vars.get(k).cloned()),
            PathBuf::from("/home/u/.local/state/orbiscreen/portal.json")
        );
    }

    #[test]
    fn path_ignores_relative_xdg_state_home() {
        let mut vars: HashMap<&str, std::ffi::OsString> = HashMap::new();
        vars.insert("XDG_STATE_HOME", "relative/path".into());
        vars.insert("HOME", "/home/u".into());
        assert_eq!(
            portal_state_path_from(|k| vars.get(k).cloned()),
            PathBuf::from("/home/u/.local/state/orbiscreen/portal.json")
        );
    }

    #[test]
    fn state_roundtrips_with_private_permissions() {
        let dir = unique_dir("roundtrip");
        let path = dir.join("orbiscreen/portal.json");
        let state = PortalState {
            screencast_restore_token: Some("cast-token".into()),
            remote_desktop_restore_token: Some("rd-token".into()),
        };
        save_portal_state_to(&state, &path).expect("save");
        assert_eq!(load_portal_state_from(&path), state);
        let perm = std::fs::metadata(&path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(
            perm & 0o077,
            0,
            "portal state must not be group/world readable"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_state_falls_back_to_default() {
        let dir = unique_dir("corrupt");
        let path = dir.join("portal.json");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(&path, "not json").expect("write");
        assert_eq!(load_portal_state_from(&path), PortalState::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_state_is_default() {
        assert_eq!(
            load_portal_state_from(&unique_dir("missing").join("nope.json")),
            PortalState::default()
        );
    }
}

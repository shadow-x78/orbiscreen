use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PortalState {
    pub screencast_restore_token: Option<String>,
    pub remote_desktop_restore_token: Option<String>,
}

pub fn portal_state_path() -> Option<PathBuf> {
    crate::paths::portal_state_path()
}

pub fn load_portal_state() -> PortalState {
    match portal_state_path() {
        Some(path) => load_portal_state_from(&path),
        None => PortalState::default(),
    }
}

pub fn save_portal_state(state: &PortalState) -> std::io::Result<()> {
    match portal_state_path() {
        Some(path) => save_portal_state_to(state, &path),
        None => {
            tracing::warn!("portal state not persisted: neither XDG_STATE_HOME nor HOME is set");
            Ok(())
        }
    }
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
    let content = serde_json::to_vec_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    crate::paths::write_secret(path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    fn unique_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("orbiscreen-portal-{}-{tag}", std::process::id()))
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

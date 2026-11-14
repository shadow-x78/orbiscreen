use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static WRITE_NONCE: AtomicU64 = AtomicU64::new(0);

pub fn config_home() -> Option<PathBuf> {
    config_home_from(|key| std::env::var_os(key))
}

pub fn config_home_from(mut lookup: impl FnMut(&str) -> Option<OsString>) -> Option<PathBuf> {
    if let Some(xdg) =
        lookup("XDG_CONFIG_HOME").filter(|v| !v.is_empty() && Path::new(v).is_absolute())
    {
        return Some(PathBuf::from(xdg));
    }
    lookup("HOME")
        .filter(|v| !v.is_empty() && Path::new(v).is_absolute())
        .map(|home| PathBuf::from(home).join(".config"))
}

pub fn state_home() -> Option<PathBuf> {
    state_home_from(|key| std::env::var_os(key))
}

pub fn state_home_from(mut lookup: impl FnMut(&str) -> Option<OsString>) -> Option<PathBuf> {
    if let Some(xdg) =
        lookup("XDG_STATE_HOME").filter(|v| !v.is_empty() && Path::new(v).is_absolute())
    {
        return Some(PathBuf::from(xdg));
    }
    lookup("HOME")
        .filter(|v| !v.is_empty() && Path::new(v).is_absolute())
        .map(|home| PathBuf::from(home).join(".local/state"))
}

pub fn data_home() -> Option<PathBuf> {
    data_home_from(|key| std::env::var_os(key))
}

pub fn data_home_from(mut lookup: impl FnMut(&str) -> Option<OsString>) -> Option<PathBuf> {
    if let Some(xdg) =
        lookup("XDG_DATA_HOME").filter(|v| !v.is_empty() && Path::new(v).is_absolute())
    {
        return Some(PathBuf::from(xdg));
    }
    lookup("HOME")
        .filter(|v| !v.is_empty() && Path::new(v).is_absolute())
        .map(|home| PathBuf::from(home).join(".local/share"))
}

pub fn config_path_from(mut lookup: impl FnMut(&str) -> Option<OsString>) -> PathBuf {
    config_home_from(&mut lookup)
        .map(|dir| dir.join("orbiscreen/orbiscreen.toml"))
        .unwrap_or_else(|| PathBuf::from("orbiscreen.toml"))
}

pub fn default_config_path() -> PathBuf {
    config_path_from(|key| std::env::var_os(key))
}

pub fn default_token_path() -> PathBuf {
    default_config_path().with_file_name("token")
}

pub fn default_wt_cert_path() -> PathBuf {
    default_config_path().with_file_name("wt-cert.pem")
}

pub fn default_wt_key_path() -> PathBuf {
    default_config_path().with_file_name("wt-key.pem")
}

pub fn default_https_cert_path() -> PathBuf {
    default_config_path().with_file_name("https-cert.pem")
}

pub fn default_https_key_path() -> PathBuf {
    default_config_path().with_file_name("https-key.pem")
}

pub fn portal_state_path() -> Option<PathBuf> {
    portal_state_path_from(|key| std::env::var_os(key))
}

pub fn portal_state_path_from(lookup: impl FnMut(&str) -> Option<OsString>) -> Option<PathBuf> {
    state_home_from(lookup).map(|dir| dir.join("orbiscreen/portal.json"))
}

pub fn pairing_state_path() -> Option<PathBuf> {
    pairing_state_path_from(|key| std::env::var_os(key))
}

pub fn pairing_state_path_from(lookup: impl FnMut(&str) -> Option<OsString>) -> Option<PathBuf> {
    state_home_from(lookup).map(|dir| dir.join("orbiscreen/clients.json"))
}

pub fn atomic_write(path: &Path, contents: &[u8], mode: u32) -> io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::other("file has no parent directory"))?;
    std::fs::create_dir_all(dir)?;
    let nonce = WRITE_NONCE.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("orbiscreen");
    let tmp = dir.join(format!(".{file_name}.tmp{}-{nonce}", std::process::id()));
    let result = (|| {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(mode).custom_flags(libc::O_NOFOLLOW);
        }
        let mut file = options.open(&tmp)?;
        use std::io::Write as _;
        file.write_all(contents)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            file.set_permissions(std::fs::Permissions::from_mode(mode))?;
        }
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

pub fn write_secret(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    atomic_write(path, contents, 0o600)
}

pub fn read_trimmed(path: &Path) -> io::Result<String> {
    Ok(std::fs::read_to_string(path)?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::os::unix::fs::PermissionsExt as _;

    fn unique_dir(tag: &str) -> PathBuf {
        let nonce = WRITE_NONCE.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "orbiscreen-paths-{tag}-{}-{nonce}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn lookup(vars: &[(&str, &str)]) -> impl FnMut(&str) -> Option<OsString> {
        let map: HashMap<String, OsString> = vars
            .iter()
            .map(|(k, v)| ((*k).to_string(), OsString::from(v)))
            .collect();
        move |key| map.get(key).cloned()
    }

    #[test]
    fn state_paths_prefer_xdg_then_home() {
        assert_eq!(
            portal_state_path_from(lookup(&[("XDG_STATE_HOME", "/state")])),
            Some(PathBuf::from("/state/orbiscreen/portal.json"))
        );
        assert_eq!(
            pairing_state_path_from(lookup(&[("HOME", "/home/user")])),
            Some(PathBuf::from(
                "/home/user/.local/state/orbiscreen/clients.json"
            ))
        );
    }

    #[test]
    fn relative_xdg_state_home_is_ignored() {
        assert_eq!(
            portal_state_path_from(lookup(&[
                ("XDG_STATE_HOME", "relative/state"),
                ("HOME", "/home/user")
            ])),
            Some(PathBuf::from(
                "/home/user/.local/state/orbiscreen/portal.json"
            ))
        );
    }

    #[test]
    fn state_path_is_none_without_xdg_or_home() {
        assert_eq!(portal_state_path_from(lookup(&[])), None);
    }

    #[test]
    fn config_path_prefers_xdg_then_home() {
        assert_eq!(
            config_path_from(lookup(&[("XDG_CONFIG_HOME", "/cfg")])),
            PathBuf::from("/cfg/orbiscreen/orbiscreen.toml")
        );
        assert_eq!(
            config_path_from(lookup(&[("HOME", "/home/user")])),
            PathBuf::from("/home/user/.config/orbiscreen/orbiscreen.toml")
        );
    }

    #[test]
    fn atomic_write_replaces_contents_and_sets_mode() {
        let dir = unique_dir("atomic");
        let path = dir.join("nested/secret");
        atomic_write(&path, b"first", 0o600).expect("write");
        atomic_write(&path, b"second", 0o600).expect("rewrite");
        assert_eq!(std::fs::read(&path).unwrap(), b"second");
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_secret_restricts_parent_and_file() {
        let dir = unique_dir("secret");
        let path = dir.join("state/clients.json");
        write_secret(&path, b"{}").expect("write");
        assert_eq!(
            std::fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_trimmed_strips_newlines() {
        let dir = unique_dir("trim");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("token");
        std::fs::write(&path, " abc \n").unwrap();
        assert_eq!(read_trimmed(&path).unwrap(), "abc");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

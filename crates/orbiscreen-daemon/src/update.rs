use std::process::Command;

const LATEST_URL: &str = "https://api.github.com/repos/shadow-x78/orbiscreen/releases/latest";

#[derive(Debug, serde::Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

pub fn disabled() -> bool {
    std::env::var_os("ORBISCREEN_NO_UPDATE_CHECK").is_some()
}

pub fn latest() -> Option<(String, String)> {
    if disabled() {
        return None;
    }
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "--max-time",
            "3",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: orbiscreen-update-check",
            LATEST_URL,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let release: Release = serde_json::from_slice(&output.stdout).ok()?;
    Some((release.tag_name, release.html_url))
}

pub fn is_newer(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(current), Some(latest)) => latest > current,
        _ => false,
    }
}

fn parse_version(value: &str) -> Option<(u64, u64, u64)> {
    let value = value.trim().trim_start_matches('v');
    let core = value.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

pub fn notice() -> Option<String> {
    let (tag, url) = latest()?;
    if !is_newer(env!("CARGO_PKG_VERSION"), &tag) {
        return None;
    }
    Some(format!(
        "A newer Orbiscreen is available: {tag} (installed: v{}).\nDownload: {url}",
        env!("CARGO_PKG_VERSION")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_release_versions() {
        assert!(is_newer("0.41.2", "v0.42.0"));
        assert!(is_newer("0.41.2", "0.41.3"));
        assert!(!is_newer("0.42.0", "v0.42.0"));
        assert!(!is_newer("0.42.0", "0.41.9"));
        assert!(!is_newer("0.42.0", "not-a-version"));
    }

    #[test]
    fn parses_the_version_core() {
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
        assert_eq!(parse_version("2.0.0-beta.1"), Some((2, 0, 0)));
        assert_eq!(parse_version("x"), None);
    }
}

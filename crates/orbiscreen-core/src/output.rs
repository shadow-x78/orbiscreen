pub const BASE_PROTOCOL_NAME: &str = "ORBISCREEN";
pub const VIRTUAL_OUTPUT_CONNECTOR: &str = "Virtual-ORBISCREEN";
pub const VIRTUAL_CONNECTOR_PREFIX: &str = "Virtual-ORBISCREEN";
pub const CLIENT_PROTOCOL_PREFIX: &str = "Orbi-";
pub const CLIENT_CONNECTOR_PREFIX: &str = "Virtual-Orbi-";

pub fn connector_for(protocol_name: &str) -> String {
    format!("Virtual-{protocol_name}")
}

pub fn default_protocol_names() -> Vec<String> {
    vec![
        BASE_PROTOCOL_NAME.to_string(),
        format!("{BASE_PROTOCOL_NAME}-{}", std::process::id()),
    ]
}

pub fn sanitize_client_name(raw: &str) -> String {
    let mut slug = String::new();
    let mut dash = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            dash = false;
        } else if !dash && !slug.is_empty() {
            slug.push('-');
            dash = true;
        }
        if slug.len() >= 32 {
            break;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "client".into()
    } else {
        slug
    }
}

pub fn client_protocol_names(client_name: &str, device_key: Option<&str>) -> (String, Vec<String>) {
    let trimmed = client_name.trim();
    let description = if trimmed.is_empty() {
        "OrbiScreen".to_string()
    } else {
        trimmed.to_string()
    };
    let slug = device_key
        .map(sanitize_client_name)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| sanitize_client_name(trimmed));
    let base = format!("{CLIENT_PROTOCOL_PREFIX}{slug}");
    let names = vec![
        base.clone(),
        format!("{base}-2"),
        format!("{base}-{}", std::process::id()),
    ];
    (description, names)
}

pub fn device_label_for(connector: &str) -> String {
    let suffix = connector
        .strip_prefix(CLIENT_CONNECTOR_PREFIX)
        .map(|suffix| format!("OrbiScreen-{suffix}"))
        .unwrap_or_else(|| {
            format!(
                "OrbiScreen-{}",
                sanitize_client_name(connector.strip_prefix("Virtual-").unwrap_or(connector))
            )
        });
    suffix
}

pub fn is_our_connector(name: &str) -> bool {
    name.starts_with(VIRTUAL_CONNECTOR_PREFIX) || name.starts_with(CLIENT_CONNECTOR_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_client_name_slug() {
        assert_eq!(sanitize_client_name("My Tablet!"), "My-Tablet");
        assert_eq!(sanitize_client_name("   "), "client");
        assert!(sanitize_client_name(&"a".repeat(80)).len() <= 32);
    }

    #[test]
    fn client_names_are_stable_and_collision_resistant() {
        let (description, names) = client_protocol_names("Tablet", Some("abcd1234"));
        assert_eq!(description, "Tablet");
        assert_eq!(names[0], "Orbi-abcd1234");
        assert_eq!(names[1], "Orbi-abcd1234-2");
        assert_eq!(names[2], format!("Orbi-abcd1234-{}", std::process::id()));
        assert_eq!(connector_for(&names[0]), "Virtual-Orbi-abcd1234");
        assert!(is_our_connector(&connector_for(&names[0])));
        assert!(is_our_connector(VIRTUAL_OUTPUT_CONNECTOR));
        assert!(!is_our_connector("DP-1"));
    }

    #[test]
    fn device_label_matches_the_input_device_names() {
        assert_eq!(
            device_label_for("Virtual-Orbi-abcd1234"),
            "OrbiScreen-abcd1234"
        );
        assert_eq!(
            device_label_for(VIRTUAL_OUTPUT_CONNECTOR),
            "OrbiScreen-ORBISCREEN"
        );
    }

    #[test]
    fn default_protocol_names_include_the_process_id() {
        let names = default_protocol_names();
        assert_eq!(names[0], "ORBISCREEN");
        assert_eq!(names[1], format!("ORBISCREEN-{}", std::process::id()));
    }
}

//! Opt-in, worktree-local dependency-storage pilot. This is NOT a shared cache.
//! Keep separate cargo invocations: batching packages would change feature resolution.

pub fn pilot_enabled(value: Option<&str>) -> bool {
    match value {
        None | Some("0") => false,
        Some("1") => true,
        _ => panic!("VIBEOS_RUST_STORAGE_PILOT must be 0 or 1"),
    }
}

pub fn pilot_member(package: &str, body: &str) -> bool {
    matches!(
        (package, body),
        ("beep-cargo", "beep-rust") | ("null-cargo", "null-rust") | ("condrv-cargo", "condrv-rust")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_in_only() {
        assert!(!pilot_enabled(None));
        assert!(!pilot_enabled(Some("0")));
        assert!(pilot_enabled(Some("1")));
    }

    #[test]
    #[should_panic(expected = "must be 0 or 1")]
    fn invalid_setting_fails_loud() {
        pilot_enabled(Some("yes"));
    }

    #[test]
    fn exact_body_and_owner_allowlist() {
        for name in ["beep", "null", "condrv"] {
            assert!(pilot_member(
                &format!("{name}-cargo"),
                &format!("{name}-rust")
            ));
        }
        assert!(!pilot_member("beep-cargo", "cnt-rust-lib"));
        assert!(!pilot_member("ntoskrnl-cargo", "beep-rust"));
        assert!(!pilot_member("halmp-cargo", "hal-rust"));
    }
}

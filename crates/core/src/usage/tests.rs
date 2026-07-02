//! Expectations come from docs/SPECIFICATION.md §5.A 環境ごとの利用量表示:
//! opt-in per environment, existing statusLine settings are parked and chained,
//! enabled-state is derived from settings.json itself, disabling restores the
//! parked setting (but never overwrites a manual change), shared settings.json
//! is refused, and stale windows (resets_at passed) are never shown.

use super::*;
use crate::profile::{IsolationConfig, ProfileMeta, SharingConfig, SharingMode};
use std::fs;
use tempfile::tempdir;

fn setup() -> (UsagePaths, tempfile::TempDir) {
    let tmp = tempdir().unwrap();
    let app_data = tmp.path().join("app_data");
    let cli_default = tmp.path().join("cli_default");
    fs::create_dir_all(&app_data).unwrap();
    fs::create_dir_all(&cli_default).unwrap();
    (UsagePaths::new(app_data, cli_default), tmp)
}

/// A created (non-default) environment whose cli-data dir sits where the real
/// app puts it: <app_data>/profiles/<name>/cli-data. The generated script
/// derives the save target from that layout, so tests must mirror it.
fn env_profile(paths: &UsagePaths, name: &str, cli_settings: SharingMode) -> Profile {
    let dir = paths.app_data_dir().join("profiles").join(name);
    Profile {
        profile: ProfileMeta {
            name: name.to_string(),
            icon: String::new(),
            color: String::new(),
            is_default: false,
        },
        isolation: IsolationConfig {
            desktop_user_data_dir: dir.join("desktop-data"),
            cli_config_dir: dir.join("cli-data"),
        },
        sharing: SharingConfig {
            cli_settings,
            ..SharingConfig::default()
        },
    }
}

/// The default environment: its cli dir is the real ~/.claude stand-in, its
/// settings.json is its own real file (Share here means "is the source").
fn default_profile(paths: &UsagePaths, cli_default: &std::path::Path) -> Profile {
    let _ = paths;
    Profile {
        profile: ProfileMeta {
            name: "default".to_string(),
            icon: String::new(),
            color: String::new(),
            is_default: true,
        },
        isolation: IsolationConfig {
            desktop_user_data_dir: std::path::PathBuf::from("/nonexistent-desktop"),
            cli_config_dir: cli_default.to_path_buf(),
        },
        sharing: SharingConfig {
            cli_settings: SharingMode::Share,
            cli_claude_md: SharingMode::Share,
            cli_project_memory: SharingMode::Share,
            cli_plugins: SharingMode::Share,
            cli_skills: SharingMode::Share,
            cli_history: SharingMode::Share,
            desktop_worktrees: SharingMode::Share,
            source: Default::default(),
        },
    }
}

fn settings_path(profile: &Profile) -> std::path::PathBuf {
    profile.isolation.cli_config_dir.join("settings.json")
}

fn read_settings(profile: &Profile) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(settings_path(profile)).unwrap()).unwrap()
}

// --- enable -----------------------------------------------------------------

#[test]
fn enable_creates_script_and_settings_from_scratch() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Isolate);
    // No cli-data dir, no settings.json yet (Claude Code never ran here).
    enable(&paths, &profile).unwrap();

    let script = paths.script_path("work");
    assert!(script.exists(), "script must be generated");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&script).unwrap().permissions().mode();
        assert_ne!(mode & 0o111, 0, "script must be executable");
    }

    let settings = read_settings(&profile);
    assert_eq!(settings["statusLine"]["type"], "command");
    assert_eq!(
        settings["statusLine"]["command"],
        script.to_string_lossy().as_ref()
    );
    // Nothing was parked: there was no statusLine before.
    assert!(!paths.backup_path("work").exists());
}

#[test]
fn enable_preserves_unrelated_settings_keys() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Copy);
    fs::create_dir_all(&profile.isolation.cli_config_dir).unwrap();
    fs::write(
        settings_path(&profile),
        r#"{"permissions":{"allow":["Bash"]},"theme":"dark"}"#,
    )
    .unwrap();

    enable(&paths, &profile).unwrap();

    let settings = read_settings(&profile);
    assert_eq!(settings["permissions"]["allow"][0], "Bash");
    assert_eq!(settings["theme"], "dark");
    assert_eq!(settings["statusLine"]["type"], "command");
}

#[test]
fn enable_parks_existing_statusline_and_chains_it() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Copy);
    fs::create_dir_all(&profile.isolation.cli_config_dir).unwrap();
    fs::write(
        settings_path(&profile),
        r#"{"statusLine":{"type":"command","command":"echo my-own-line","padding":1}}"#,
    )
    .unwrap();

    enable(&paths, &profile).unwrap();

    // The original object is parked verbatim for later restore.
    let parked: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(paths.backup_path("work")).unwrap()).unwrap();
    assert_eq!(parked["command"], "echo my-own-line");
    assert_eq!(parked["padding"], 1);

    // The script chains the original command so its output keeps appearing.
    let script_body = fs::read_to_string(paths.script_path("work")).unwrap();
    assert!(script_body.contains("echo my-own-line"));

    // settings.json now points at the script.
    let settings = read_settings(&profile);
    assert_eq!(
        settings["statusLine"]["command"],
        paths.script_path("work").to_string_lossy().as_ref()
    );
}

#[test]
fn enable_twice_is_idempotent_and_never_parks_own_script() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Copy);
    fs::create_dir_all(&profile.isolation.cli_config_dir).unwrap();
    fs::write(
        settings_path(&profile),
        r#"{"statusLine":{"type":"command","command":"echo my-own-line"}}"#,
    )
    .unwrap();

    enable(&paths, &profile).unwrap();
    enable(&paths, &profile).unwrap();

    // The park still holds the user's command, not CSW's script.
    let parked: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(paths.backup_path("work")).unwrap()).unwrap();
    assert_eq!(parked["command"], "echo my-own-line");
    // And the chained script still carries the user's command.
    let script_body = fs::read_to_string(paths.script_path("work")).unwrap();
    assert!(script_body.contains("echo my-own-line"));
}

#[test]
fn enable_refuses_environment_sharing_settings_with_default() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Share);

    assert!(matches!(
        can_enable(&profile),
        Err(CswError::UsageSettingsShared(_))
    ));
    assert!(matches!(
        enable(&paths, &profile),
        Err(CswError::UsageSettingsShared(_))
    ));
    assert!(!paths.script_path("work").exists());
}

#[test]
fn enable_allows_the_default_environment_itself() {
    let (paths, tmp) = setup();
    let cli_default = tmp.path().join("cli_default");
    let profile = default_profile(&paths, &cli_default);

    can_enable(&profile).unwrap();
    enable(&paths, &profile).unwrap();

    let settings = read_settings(&profile);
    assert_eq!(
        settings["statusLine"]["command"],
        paths.script_path("default").to_string_lossy().as_ref()
    );
}

#[test]
fn enable_leaves_broken_settings_json_untouched() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Copy);
    fs::create_dir_all(&profile.isolation.cli_config_dir).unwrap();
    let broken = "{ this is not json";
    fs::write(settings_path(&profile), broken).unwrap();

    assert!(matches!(
        enable(&paths, &profile),
        Err(CswError::UsageSettingsUnreadable(_, _))
    ));
    assert_eq!(
        fs::read_to_string(settings_path(&profile)).unwrap(),
        broken,
        "a file we cannot parse must not be rewritten"
    );
}

// --- enabled-state derivation ------------------------------------------------

#[test]
fn enabled_state_is_derived_from_settings_json() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Isolate);

    assert!(!is_enabled(&paths, &profile), "no settings.json yet");

    enable(&paths, &profile).unwrap();
    assert!(is_enabled(&paths, &profile));

    // The user removes the key by hand -> derived state flips back, no flag drifts.
    fs::write(settings_path(&profile), "{}").unwrap();
    assert!(!is_enabled(&paths, &profile));
}

// --- disable ------------------------------------------------------------------

#[test]
fn disable_restores_the_parked_statusline() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Copy);
    fs::create_dir_all(&profile.isolation.cli_config_dir).unwrap();
    fs::write(
        settings_path(&profile),
        r#"{"statusLine":{"type":"command","command":"echo my-own-line","padding":1},"theme":"dark"}"#,
    )
    .unwrap();

    enable(&paths, &profile).unwrap();
    disable(&paths, &profile).unwrap();

    let settings = read_settings(&profile);
    assert_eq!(settings["statusLine"]["command"], "echo my-own-line");
    assert_eq!(settings["statusLine"]["padding"], 1);
    assert_eq!(settings["theme"], "dark");
    assert!(!paths.script_path("work").exists());
    assert!(!paths.backup_path("work").exists());
    assert!(!paths.usage_file_path("work").exists());
}

#[test]
fn disable_drops_the_key_when_nothing_was_parked() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Isolate);
    fs::create_dir_all(&profile.isolation.cli_config_dir).unwrap();
    fs::write(settings_path(&profile), r#"{"theme":"dark"}"#).unwrap();

    enable(&paths, &profile).unwrap();
    // A capture exists; disabling must clean it up.
    fs::create_dir_all(paths.usage_file_path("work").parent().unwrap()).unwrap();
    fs::write(paths.usage_file_path("work"), "{}").unwrap();

    disable(&paths, &profile).unwrap();

    let settings = read_settings(&profile);
    assert!(settings.get("statusLine").is_none());
    assert_eq!(settings["theme"], "dark");
    assert!(!paths.script_path("work").exists());
    assert!(!paths.usage_file_path("work").exists());
}

#[test]
fn disable_never_overwrites_a_manual_statusline_change() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Isolate);
    fs::create_dir_all(&profile.isolation.cli_config_dir).unwrap();

    enable(&paths, &profile).unwrap();
    // The user has since pointed statusLine at their own command.
    fs::write(
        settings_path(&profile),
        r#"{"statusLine":{"type":"command","command":"my-custom"}}"#,
    )
    .unwrap();

    disable(&paths, &profile).unwrap();

    let settings = read_settings(&profile);
    assert_eq!(
        settings["statusLine"]["command"], "my-custom",
        "a manual setting belongs to the user and must survive disable"
    );
    // CSW's own artifacts are still cleaned up.
    assert!(!paths.script_path("work").exists());
}

#[test]
fn remove_artifacts_clears_everything_and_tolerates_absence() {
    let (paths, _tmp) = setup();
    let profile = env_profile(&paths, "work", SharingMode::Isolate);
    enable(&paths, &profile).unwrap();
    fs::create_dir_all(paths.usage_file_path("work").parent().unwrap()).unwrap();
    fs::write(paths.usage_file_path("work"), "{}").unwrap();

    remove_artifacts(&paths, "work").unwrap();
    assert!(!paths.script_path("work").exists());
    assert!(!paths.backup_path("work").exists());
    assert!(!paths.usage_file_path("work").exists());

    // Second run: nothing left, still Ok.
    remove_artifacts(&paths, "work").unwrap();
}

// --- read_snapshot -------------------------------------------------------------

fn write_usage(paths: &UsagePaths, name: &str, body: &str) {
    let p = paths.usage_file_path(name);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, body).unwrap();
}

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn read_snapshot_returns_live_windows_with_age() {
    let (paths, _tmp) = setup();
    let now = now_epoch();
    let body = format!(
        r#"{{"model":{{"display_name":"Opus"}},"rate_limits":{{"five_hour":{{"used_percentage":23.4,"resets_at":{}}},"seven_day":{{"used_percentage":41.6,"resets_at":{}}}}}}}"#,
        now + 3600,
        now + 86400
    );
    write_usage(&paths, "work", &body);

    // Pretend 5 minutes have passed since the file was written.
    let snap = read_snapshot(&paths, "work", now + 300).unwrap();
    assert_eq!(
        snap.five_hour,
        Some(UsageWindow {
            used_percentage: 23.4,
            resets_at: now + 3600
        })
    );
    assert_eq!(
        snap.seven_day,
        Some(UsageWindow {
            used_percentage: 41.6,
            resets_at: now + 86400
        })
    );
    assert!(
        (298..=305).contains(&snap.age_seconds),
        "age must reflect time since capture, got {}",
        snap.age_seconds
    );
}

#[test]
fn read_snapshot_drops_expired_windows() {
    let (paths, _tmp) = setup();
    let now = now_epoch();
    // five_hour window already reset; seven_day still running.
    let body = format!(
        r#"{{"rate_limits":{{"five_hour":{{"used_percentage":90.0,"resets_at":{}}},"seven_day":{{"used_percentage":41.6,"resets_at":{}}}}}}}"#,
        now - 10,
        now + 86400
    );
    write_usage(&paths, "work", &body);

    let snap = read_snapshot(&paths, "work", now).unwrap();
    assert_eq!(snap.five_hour, None, "a reset window's value is unknowable");
    assert!(snap.seven_day.is_some());
}

#[test]
fn read_snapshot_is_none_when_everything_expired() {
    let (paths, _tmp) = setup();
    let now = now_epoch();
    let body = format!(
        r#"{{"rate_limits":{{"five_hour":{{"used_percentage":90.0,"resets_at":{}}},"seven_day":{{"used_percentage":41.6,"resets_at":{}}}}}}}"#,
        now - 10,
        now - 5
    );
    write_usage(&paths, "work", &body);
    assert!(read_snapshot(&paths, "work", now).is_none());
}

#[test]
fn read_snapshot_is_none_for_missing_invalid_or_limitless_payloads() {
    let (paths, _tmp) = setup();
    let now = now_epoch();

    assert!(read_snapshot(&paths, "nope", now).is_none(), "no file");

    write_usage(&paths, "bad", "{ not json");
    assert!(read_snapshot(&paths, "bad", now).is_none(), "invalid JSON");

    // A real payload from an API-key session: no rate_limits at all.
    write_usage(&paths, "api", r#"{"model":{"display_name":"Opus"}}"#);
    assert!(
        read_snapshot(&paths, "api", now).is_none(),
        "no rate_limits"
    );

    // rate_limits present but both windows absent (documented as possible).
    write_usage(&paths, "empty", r#"{"rate_limits":{}}"#);
    assert!(read_snapshot(&paths, "empty", now).is_none());
}

// --- the generated script, exercised for real (macOS: /bin/sh + plutil) --------

#[cfg(target_os = "macos")]
mod script {
    use super::*;
    use std::io::Write;
    use std::process::{Command, Stdio};

    const PAYLOAD: &str = r#"{"model":{"display_name":"Opus"},"rate_limits":{"five_hour":{"used_percentage":23.4,"resets_at":4102444800},"seven_day":{"used_percentage":41.6,"resets_at":4102444800}}}"#;

    fn run_script(
        paths: &UsagePaths,
        name: &str,
        config_dir: Option<&std::path::Path>,
        stdin_body: &str,
    ) -> std::process::Output {
        let mut cmd = Command::new("/bin/sh");
        cmd.arg(paths.script_path(name))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match config_dir {
            Some(dir) => {
                cmd.env("CLAUDE_CONFIG_DIR", dir);
            }
            None => {
                cmd.env_remove("CLAUDE_CONFIG_DIR");
            }
        }
        let mut child = cmd.spawn().unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin_body.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    #[test]
    fn script_saves_payload_and_prints_default_line() {
        let (paths, _tmp) = setup();
        let profile = env_profile(&paths, "work", SharingMode::Isolate);
        enable(&paths, &profile).unwrap();

        let out = run_script(
            &paths,
            "work",
            Some(&profile.isolation.cli_config_dir),
            PAYLOAD,
        );
        assert!(
            out.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );

        // The payload is saved verbatim for the GUI to read.
        let saved = fs::read_to_string(paths.usage_file_path("work")).unwrap();
        assert_eq!(saved, PAYLOAD);

        // And a concise default line is printed (5h first, then week).
        let line = String::from_utf8_lossy(&out.stdout);
        assert!(line.contains("5h 23%"), "got: {line}");
        assert!(line.contains("week 42%"), "got: {line}");
    }

    #[test]
    fn script_derives_default_env_when_config_dir_is_unset() {
        let (paths, tmp) = setup();
        let cli_default = tmp.path().join("cli_default");
        let profile = default_profile(&paths, &cli_default);
        enable(&paths, &profile).unwrap();

        let out = run_script(&paths, "default", None, PAYLOAD);
        assert!(out.status.success());
        assert_eq!(
            fs::read_to_string(paths.usage_file_path("default")).unwrap(),
            PAYLOAD
        );
    }

    #[test]
    fn script_saves_nothing_for_an_unknown_config_dir() {
        let (paths, tmp) = setup();
        let profile = env_profile(&paths, "work", SharingMode::Isolate);
        enable(&paths, &profile).unwrap();

        let alien = tmp.path().join("somewhere-else");
        fs::create_dir_all(&alien).unwrap();
        let out = run_script(&paths, "work", Some(&alien), PAYLOAD);
        assert!(
            out.status.success(),
            "an unknown dir must not fail the status line"
        );
        assert!(
            !paths.app_data_dir().join("usage").exists()
                || fs::read_dir(paths.app_data_dir().join("usage"))
                    .unwrap()
                    .next()
                    .is_none(),
            "nothing may be saved for a config dir CSW does not manage"
        );
    }

    #[test]
    fn script_chains_the_original_command_output() {
        let (paths, _tmp) = setup();
        let profile = env_profile(&paths, "work", SharingMode::Copy);
        fs::create_dir_all(&profile.isolation.cli_config_dir).unwrap();
        fs::write(
            settings_path(&profile),
            r#"{"statusLine":{"type":"command","command":"cat >/dev/null; echo original-line"}}"#,
        )
        .unwrap();
        enable(&paths, &profile).unwrap();

        let out = run_script(
            &paths,
            "work",
            Some(&profile.isolation.cli_config_dir),
            PAYLOAD,
        );
        assert!(out.status.success());
        let line = String::from_utf8_lossy(&out.stdout);
        assert!(
            line.contains("original-line"),
            "the user's own status line must keep appearing, got: {line}"
        );
        // The payload is still captured alongside.
        assert!(paths.usage_file_path("work").exists());
    }

    #[test]
    fn script_prints_nothing_notable_without_rate_limits() {
        let (paths, _tmp) = setup();
        let profile = env_profile(&paths, "work", SharingMode::Isolate);
        enable(&paths, &profile).unwrap();

        let out = run_script(
            &paths,
            "work",
            Some(&profile.isolation.cli_config_dir),
            r#"{"model":{"display_name":"Opus"}}"#,
        );
        assert!(out.status.success(), "missing rate_limits must not error");
        let line = String::from_utf8_lossy(&out.stdout);
        assert!(!line.contains('%'), "no percentages to show, got: {line}");
    }
}

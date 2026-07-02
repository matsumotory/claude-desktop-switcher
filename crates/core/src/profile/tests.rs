use super::*;
use crate::platform::PlatformProvider;
use crate::platform::mock::MockPlatformProvider;
use std::sync::Arc;
use tempfile::tempdir;

fn setup_test_manager() -> (Arc<MockPlatformProvider>, ProfileManager, tempfile::TempDir) {
    let tmp_dir = tempdir().unwrap();
    let app_data = tmp_dir.path().join("app_data");
    let desktop_default = tmp_dir.path().join("desktop_default");
    let cli_default = tmp_dir.path().join("cli_default");

    std::fs::create_dir_all(&app_data).unwrap();
    std::fs::create_dir_all(&desktop_default).unwrap();
    std::fs::create_dir_all(&cli_default).unwrap();

    let provider = Arc::new(MockPlatformProvider::new(
        desktop_default,
        cli_default,
        app_data,
    ));

    let manager = ProfileManager::new(provider.clone()).unwrap();

    (provider, manager, tmp_dir)
}

#[test]
fn test_default_profile() {
    let (_, manager, _tmp_dir) = setup_test_manager();

    let default_profile = manager
        .get_profile("default")
        .expect("Failed to get default profile");
    assert_eq!(default_profile.profile.name, "default");
    assert!(default_profile.profile.is_default);

    assert_eq!(manager.active_profile_name(), "default");
}

#[test]
fn test_create_profile() {
    let (_, manager, _tmp_dir) = setup_test_manager();

    let profile = manager
        .create_profile("test_profile", SharingConfig::default(), None)
        .expect("Failed to create profile");

    assert_eq!(profile.profile.name, "test_profile");
    assert!(!profile.profile.is_default);

    // Verify it shows up in list
    let list = manager.list_profiles().unwrap();
    assert!(list.contains(&"test_profile".to_string()));
    assert!(list.contains(&"default".to_string()));
}

#[test]
fn test_delete_profile() {
    let (_, manager, _tmp_dir) = setup_test_manager();

    manager
        .create_profile("to_delete", SharingConfig::default(), None)
        .expect("Failed to create profile");

    let list = manager.list_profiles().unwrap();
    assert!(list.contains(&"to_delete".to_string()));

    manager
        .delete_profile("to_delete")
        .expect("Failed to delete profile");

    let list_after = manager.list_profiles().unwrap();
    assert!(!list_after.contains(&"to_delete".to_string()));
}

#[test]
fn test_delete_default_or_active_fails() {
    let (_, manager, _tmp_dir) = setup_test_manager();

    // Deleting default fails
    let res = manager.delete_profile("default");
    assert!(res.is_err());

    // Deleting active fails
    manager
        .create_profile("active_prof", SharingConfig::default(), None)
        .unwrap();
    manager.switch_to("active_prof").unwrap();

    let res2 = manager.delete_profile("active_prof");
    assert!(res2.is_err());
}

#[test]
fn test_clone_profile() {
    let (_, manager, _tmp_dir) = setup_test_manager();

    // Create an original profile to clone from
    let original = manager
        .create_profile("original", SharingConfig::default(), None)
        .expect("Failed to create original profile");

    // Clone it
    let cloned = manager
        .clone_profile("original", "cloned")
        .expect("Failed to clone profile");

    assert_eq!(cloned.profile.name, "cloned");
    assert_eq!(cloned.profile.icon, original.profile.icon);
    assert_eq!(cloned.profile.color, original.profile.color);
    assert!(!cloned.profile.is_default);

    // Verify it exists in profiles list
    let list = manager.list_profiles().unwrap();
    assert!(list.contains(&"original".to_string()));
    assert!(list.contains(&"cloned".to_string()));
}

#[test]
fn test_clone_profile_rejects_default_source() {
    let (_, manager, _tmp_dir) = setup_test_manager();

    // The default profile stands for the user's real Claude data directories.
    // Cloning it would bulk-read them, which docs/PRIVACY.md promises never
    // happens, so the backend must reject it regardless of what the UI exposes.
    let res = manager.clone_profile("default", "copy-of-default");
    assert!(res.is_err());

    let list = manager.list_profiles().unwrap();
    assert!(!list.contains(&"copy-of-default".to_string()));
}

#[test]
fn validate_profile_name_accepts_japanese_and_safe_ascii() {
    for ok in [
        "研究用",
        "仕事用",
        "検証用",
        "Work",
        "test-1",
        "a_b",
        "プロジェクトA",
    ] {
        assert!(validate_profile_name(ok).is_ok(), "should accept {ok:?}");
    }
}

#[test]
fn validate_profile_name_rejects_traversal_and_shell_metacharacters() {
    for bad in [
        "", "../evil", "a/b", "a\\b", "a b", "a.b", "a$b", "a;b", "a`b`", "a|b", "..",
    ] {
        assert!(
            matches!(
                validate_profile_name(bad),
                Err(crate::error::CswError::InvalidProfileName(_))
            ),
            "should reject {bad:?}"
        );
    }
    // Over the length cap.
    let too_long = "a".repeat(65);
    assert!(validate_profile_name(&too_long).is_err());
}

#[test]
fn create_profile_rejects_path_traversal_name() {
    let (_provider, manager, _tmp) = setup_test_manager();
    let err = manager
        .create_profile("../escape", SharingConfig::default(), None)
        .unwrap_err();
    assert!(matches!(err, crate::error::CswError::InvalidProfileName(_)));
}

#[test]
fn default_roots_status_reports_present_when_dirs_have_content() {
    let (provider, manager, _tmp) = setup_test_manager();
    std::fs::write(provider.desktop_default.join("config.json"), "{}").unwrap();
    std::fs::write(provider.cli_default.join("settings.json"), "{}").unwrap();
    let s = manager.default_roots_status();
    assert!(s.desktop_present);
    assert!(s.cli_present);
}

#[test]
fn default_roots_status_reports_cli_absent_when_only_desktop_has_content() {
    // The CLAUDE_CONFIG_DIR case: Desktop is at the standard location, but the CLI
    // config was moved elsewhere, so the standard ~/.claude is empty.
    let (provider, manager, _tmp) = setup_test_manager();
    std::fs::write(provider.desktop_default.join("config.json"), "{}").unwrap();
    let s = manager.default_roots_status();
    assert!(s.desktop_present);
    assert!(!s.cli_present);
}

#[test]
fn default_roots_status_reports_both_absent_when_dirs_empty() {
    // Neither standard dir holds data: nothing to share, so the UI offers only
    // "すべて分ける".
    let (_provider, manager, _tmp) = setup_test_manager();
    let s = manager.default_roots_status();
    assert!(!s.desktop_present);
    assert!(!s.cli_present);
}

#[test]
fn share_settings_preset_shares_rules_copies_settings_isolates_conversations() {
    let p = SharingConfig::share_settings_preset();
    // Shared by symlink: the app reads these and the user is the single writer.
    assert_eq!(p.cli_claude_md, SharingMode::Share);
    assert_eq!(p.cli_plugins, SharingMode::Share);
    assert_eq!(p.cli_skills, SharingMode::Share);
    // Copied once: the user's own settings and worktree list, reused as a starting
    // point (a single file that the CLI may rewrite, so copy rather than symlink).
    assert_eq!(p.cli_settings, SharingMode::Copy);
    assert_eq!(p.desktop_worktrees, SharingMode::Copy);
    // Conversations stay separate in this mode.
    assert_eq!(p.cli_project_memory, SharingMode::Isolate);
    assert_eq!(p.cli_history, SharingMode::Isolate);
    // The account-keyed files (config.json, claude_desktop_config.json), sessions/
    // and the device id are not SharingConfig fields at all, so no preset can ever
    // express sharing them — the always-isolated invariant is structural.
}

#[test]
fn share_workspace_preset_also_shares_conversations_but_never_auth() {
    let p = SharingConfig::share_workspace_preset();
    // Inherits everything the settings preset shares/copies.
    assert_eq!(p.cli_claude_md, SharingMode::Share);
    assert_eq!(p.cli_plugins, SharingMode::Share);
    assert_eq!(p.cli_skills, SharingMode::Share);
    assert_eq!(p.cli_settings, SharingMode::Copy);
    assert_eq!(p.desktop_worktrees, SharingMode::Copy);
    // Plus the per-project conversation history, auto-memory and prompt history.
    assert_eq!(p.cli_project_memory, SharingMode::Share);
    assert_eq!(p.cli_history, SharingMode::Share);
    // sessions/, config.json, claude_desktop_config.json and the device id are not
    // SharingConfig fields, so even this loosest mode cannot express sharing the
    // login / account state — the isolation is structural (verified at the file
    // system level in create_with_share_workspace_preset_shares_conversations_isolates_auth).
}

#[test]
fn default_sharing_isolates_all_configurable_components() {
    // "すべて分ける" uses the default; nothing carries over. The account-keyed files,
    // sessions/ and the device id are not fields here at all (always isolated).
    let d = SharingConfig::default();
    for mode in [
        d.cli_settings,
        d.cli_claude_md,
        d.cli_project_memory,
        d.cli_plugins,
        d.cli_skills,
        d.cli_history,
        d.desktop_worktrees,
    ] {
        assert_eq!(mode, SharingMode::Isolate);
    }
}

#[test]
fn create_with_share_settings_preset_isolates_auth_and_shares_rules() {
    let (provider, manager, _tmp) = setup_test_manager();

    // Seed the source ("default") with files that mirror the real machine.
    std::fs::write(provider.cli_default.join("CLAUDE.md"), "global rules").unwrap();
    std::fs::create_dir_all(provider.cli_default.join("skills")).unwrap();
    std::fs::write(
        provider.cli_default.join("settings.json"),
        "{\"hooks\":\"rm -rf /\"}",
    )
    .unwrap();
    std::fs::write(
        provider.desktop_default.join("config.json"),
        "{\"oauth:tokenCache\":\"SECRET-TOKEN\"}",
    )
    .unwrap();
    std::fs::create_dir_all(provider.cli_default.join("projects")).unwrap();
    std::fs::write(
        provider.cli_default.join("projects").join("session.jsonl"),
        "private transcript",
    )
    .unwrap();
    std::fs::write(
        provider.desktop_default.join("git-worktrees.json"),
        "{\"w\":1}",
    )
    .unwrap();

    let profile = manager
        .create_profile("研究用", SharingConfig::share_settings_preset(), None)
        .expect("create should succeed");
    let cli = &profile.isolation.cli_config_dir;
    let desk = &profile.isolation.desktop_user_data_dir;

    // Shared rules/skills are symlinks back to the source.
    assert!(provider.is_symlink(&cli.join("CLAUDE.md")));
    assert!(provider.is_symlink(&cli.join("skills")));

    // settings.json is copied: a real file (not a symlink) so the two accounts'
    // settings diverge after creation rather than sharing one live file.
    assert!(cli.join("settings.json").exists());
    assert!(!provider.is_symlink(&cli.join("settings.json")));

    // config.json (OAuth token) must never appear in the profile.
    assert!(!desk.join("config.json").exists());

    // In 会話とメモリも分ける, conversations stay separate: no transcript leaks.
    assert!(!cli.join("projects").join("session.jsonl").exists());

    // git-worktrees.json is copied: a real file (not a symlink) with the content.
    assert!(desk.join("git-worktrees.json").exists());
    assert!(!provider.is_symlink(&desk.join("git-worktrees.json")));
    assert_eq!(
        std::fs::read_to_string(desk.join("git-worktrees.json")).unwrap(),
        "{\"w\":1}"
    );
}

#[test]
fn create_with_share_workspace_preset_shares_conversations_isolates_auth() {
    let (provider, manager, _tmp) = setup_test_manager();

    std::fs::write(provider.cli_default.join("CLAUDE.md"), "global rules").unwrap();
    std::fs::create_dir_all(provider.cli_default.join("projects")).unwrap();
    std::fs::write(
        provider.cli_default.join("projects").join("session.jsonl"),
        "shared transcript",
    )
    .unwrap();
    std::fs::create_dir_all(provider.cli_default.join("sessions")).unwrap();
    std::fs::write(
        provider.desktop_default.join("config.json"),
        "{\"oauth:tokenCache\":\"SECRET-TOKEN\"}",
    )
    .unwrap();

    let profile = manager
        .create_profile("研究用", SharingConfig::share_workspace_preset(), None)
        .expect("create should succeed");
    let cli = &profile.isolation.cli_config_dir;
    let desk = &profile.isolation.desktop_user_data_dir;

    // アカウントだけ分ける: conversation history + auto-memory is shared via a
    // directory symlink, so the transcript IS visible in the new environment
    // (same person, continuity).
    assert!(provider.is_symlink(&cli.join("projects")));
    assert!(cli.join("projects").join("session.jsonl").exists());
    // sessions/ holds runtime state, not conversation content, so it is isolated
    // (an empty dir, never a symlink) even in this loosest mode.
    assert!(!provider.is_symlink(&cli.join("sessions")));

    // And the login / OAuth token is still never shared, even in this loose mode.
    assert!(!desk.join("config.json").exists());
}

#[test]
fn always_isolated_files_are_never_linked_in_any_mode() {
    // SPECIFICATION.md §3 "常に分離する項目": config.json, claude_desktop_config.json,
    // sessions/ and the device id (ant-did) are isolated in EVERY mode. They are not
    // even SharingConfig fields, so "share them" is unrepresentable at COMPILE TIME —
    // no caller (GUI, CLI, or a hand-built config) can express it; there is no field
    // to set. Here we prove the filesystem effect in the loosest mode (share_workspace,
    // which shares the most): with all four present in the source, none leaks across.
    let (provider, manager, _tmp) = setup_test_manager();

    std::fs::write(
        provider.desktop_default.join("claude_desktop_config.json"),
        "{\"mcpServers\":{},\"bypassPermissionsGateByAccount\":{}}",
    )
    .unwrap();
    std::fs::write(
        provider.desktop_default.join("config.json"),
        "{\"oauth:tokenCache\":\"SECRET-TOKEN\"}",
    )
    .unwrap();
    std::fs::write(provider.desktop_default.join("ant-did"), "device-1234").unwrap();
    std::fs::create_dir_all(provider.cli_default.join("sessions")).unwrap();
    std::fs::write(
        provider.cli_default.join("sessions").join("live.json"),
        "{\"pid\":1}",
    )
    .unwrap();

    let profile = manager
        .create_profile("研究用", SharingConfig::share_workspace_preset(), None)
        .expect("create should succeed");
    let desk = &profile.isolation.desktop_user_data_dir;
    let cli = &profile.isolation.cli_config_dir;

    // None of the account-keyed files is symlinked to the source, and the OAuth token
    // / connector file never appear in the profile at all.
    assert!(!provider.is_symlink(&desk.join("claude_desktop_config.json")));
    assert!(!desk.join("claude_desktop_config.json").exists());
    assert!(!provider.is_symlink(&desk.join("config.json")));
    assert!(!desk.join("config.json").exists());
    assert!(!provider.is_symlink(&desk.join("ant-did")));
    // sessions/ is an isolated (empty) directory, never a symlink, even here.
    assert!(!provider.is_symlink(&cli.join("sessions")));
    assert!(!cli.join("sessions").join("live.json").exists());

    // The mode's legitimate carry-overs are unaffected.
    assert_eq!(profile.sharing.cli_project_memory, SharingMode::Share);
    assert_eq!(profile.sharing.cli_claude_md, SharingMode::Share);
}

// Spec: 完全分離環境の同時起動 — an environment may be launched in additional
// concurrent windows only when it shares nothing. is_fully_isolated() is the
// single source of truth: true iff every sharing component is Isolate (the
// "すべて分ける" preset). Any Share or Copy component disqualifies it.
#[test]
fn is_fully_isolated_only_when_every_component_is_isolate() {
    assert!(SharingConfig::default().is_fully_isolated());
    assert!(!SharingConfig::share_settings_preset().is_fully_isolated());
    assert!(!SharingConfig::share_workspace_preset().is_fully_isolated());

    // A single Share component disqualifies it.
    let one_share = SharingConfig {
        cli_skills: SharingMode::Share,
        ..SharingConfig::default()
    };
    assert!(!one_share.is_fully_isolated());

    // Even a one-time Copy (independent after creation, but still a carry-over)
    // disqualifies it: the button is offered strictly for "すべて分ける".
    let one_copy = SharingConfig {
        cli_settings: SharingMode::Copy,
        ..SharingConfig::default()
    };
    assert!(!one_copy.is_fully_isolated());
}

// The default profile ("既存の Claude") shares everything, so it is never eligible.
#[test]
fn default_profile_is_not_fully_isolated() {
    let (_, manager, _tmp) = setup_test_manager();
    let default_profile = manager.get_profile("default").unwrap();
    assert!(!default_profile.sharing.is_fully_isolated());
}

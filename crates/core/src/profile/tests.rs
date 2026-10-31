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
    // rules/ holds the same kind of always-on rules as CLAUDE.md (issue #190).
    assert_eq!(p.cli_rules, SharingMode::Share);
    assert_eq!(p.cli_plugins, SharingMode::Share);
    assert_eq!(p.cli_skills, SharingMode::Share);
    // Copied once: the user's own settings and worktree list, reused as a starting
    // point (a single file that the CLI may rewrite, so copy rather than symlink).
    assert_eq!(p.cli_settings, SharingMode::Copy);
    assert_eq!(p.desktop_worktrees, SharingMode::Copy);
    // The Desktop app's SSH connection list, copied like the worktree list
    // (issue #239); remembered passwords and tokens are never link items.
    assert_eq!(p.desktop_ssh_connections, SharingMode::Copy);
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
    assert_eq!(p.cli_rules, SharingMode::Share);
    assert_eq!(p.cli_plugins, SharingMode::Share);
    assert_eq!(p.cli_skills, SharingMode::Share);
    assert_eq!(p.cli_settings, SharingMode::Copy);
    assert_eq!(p.desktop_worktrees, SharingMode::Copy);
    assert_eq!(p.desktop_ssh_connections, SharingMode::Copy);
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
        d.cli_rules,
        d.cli_project_memory,
        d.cli_plugins,
        d.cli_skills,
        d.cli_history,
        d.desktop_worktrees,
        d.desktop_ssh_connections,
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

    // The rules directory is a carry-over like every other component.
    let rules_shared = SharingConfig {
        cli_rules: SharingMode::Share,
        ..SharingConfig::default()
    };
    assert!(!rules_shared.is_fully_isolated());
}

// --- ~/.claude/rules/ (issue #190) ------------------------------------------
//
// Claude Code loads the user-level rules in ~/.claude/rules/ in every project,
// next to ~/.claude/CLAUDE.md. The mode descriptions promise that "global
// rules" carry over, so the linker must manage rules/ exactly like CLAUDE.md.

use crate::profile::linker::{ItemDir, LINK_ITEMS};

#[test]
fn link_items_manage_the_rules_directory_next_to_claude_md() {
    let index = |key: &str| {
        LINK_ITEMS
            .iter()
            .position(|i| i.key == key)
            .unwrap_or_else(|| panic!("{key} is not a link item"))
    };
    let rules = &LINK_ITEMS[index("cli_rules")];
    assert_eq!(rules.rel_path, "rules");
    assert!(rules.is_directory);
    assert!(matches!(rules.dir, ItemDir::Cli));
    assert!(
        rules.fixed_mode.is_none(),
        "rules/ is tunable (share / copy / isolate) like CLAUDE.md"
    );
    // Listed right after CLAUDE.md: the two make up the user's global rules.
    assert_eq!(index("cli_rules"), index("cli_claude_md") + 1);
}

#[test]
fn create_with_share_workspace_preset_links_rules_directory() {
    let (provider, manager, _tmp) = setup_test_manager();
    let source_rules = provider.claude_cli_default_dir().join("rules");
    std::fs::create_dir_all(&source_rules).unwrap();
    std::fs::write(source_rules.join("preferences.md"), "always use pnpm").unwrap();

    let profile = manager
        .create_profile("研究用", SharingConfig::share_workspace_preset(), None)
        .unwrap();
    assert_eq!(profile.sharing.cli_rules, SharingMode::Share);

    let rules = profile.isolation.cli_config_dir.join("rules");
    assert!(
        provider.is_symlink(&rules),
        "rules/ must be shared by symlink, like CLAUDE.md"
    );
    assert_eq!(std::fs::read_link(&rules).unwrap(), source_rules);
    assert_eq!(
        std::fs::read_to_string(rules.join("preferences.md")).unwrap(),
        "always use pnpm",
        "the rule files must be visible through the link"
    );
}

#[test]
fn create_with_copy_rules_copies_the_directory_once() {
    let (provider, manager, _tmp) = setup_test_manager();
    let source_rules = provider.claude_cli_default_dir().join("rules");
    std::fs::create_dir_all(source_rules.join("frontend")).unwrap();
    std::fs::write(source_rules.join("frontend").join("style.md"), "2 spaces").unwrap();

    let sharing = SharingConfig {
        cli_rules: SharingMode::Copy,
        ..SharingConfig::default()
    };
    let profile = manager.create_profile("env", sharing, None).unwrap();

    let rules = profile.isolation.cli_config_dir.join("rules");
    assert!(rules.is_dir());
    assert!(!provider.is_symlink(&rules), "copy means a real directory");
    assert_eq!(
        std::fs::read_to_string(rules.join("frontend").join("style.md")).unwrap(),
        "2 spaces",
        "nested rule files are copied recursively"
    );
}

#[test]
fn create_isolated_profile_keeps_rules_directory_empty_and_unlinked() {
    let (provider, manager, _tmp) = setup_test_manager();
    let source_rules = provider.claude_cli_default_dir().join("rules");
    std::fs::create_dir_all(&source_rules).unwrap();
    std::fs::write(source_rules.join("preferences.md"), "private").unwrap();

    let profile = manager
        .create_profile("env", SharingConfig::default(), None)
        .unwrap();

    let rules = profile.isolation.cli_config_dir.join("rules");
    assert!(
        rules.is_dir(),
        "isolate creates an empty directory of its own"
    );
    assert!(!provider.is_symlink(&rules));
    assert_eq!(std::fs::read_dir(&rules).unwrap().count(), 0);
}

#[test]
fn default_profile_declares_rules_shared() {
    // The existing Claude is the source of every share, so its own rules/ is
    // reported as shared, like every other component.
    let (_, manager, _tmp) = setup_test_manager();
    let default_profile = manager.get_profile("default").unwrap();
    assert_eq!(default_profile.sharing.cli_rules, SharingMode::Share);
}

// The default profile ("既存の Claude") shares everything, so it is never eligible.
#[test]
fn default_profile_is_not_fully_isolated() {
    let (_, manager, _tmp) = setup_test_manager();
    let default_profile = manager.get_profile("default").unwrap();
    assert!(!default_profile.sharing.is_fully_isolated());
}

// --- csw doctor (isolation inspector) ---------------------------------------

use crate::profile::inspector::ItemHealth;
use std::path::Path;

/// Put shareable sources into the default dirs so Share links have targets.
fn populate_default_sources(cli_default: &Path, desktop_default: &Path) {
    std::fs::write(cli_default.join("CLAUDE.md"), "rules").unwrap();
    std::fs::write(cli_default.join("settings.json"), "{}").unwrap();
    std::fs::create_dir_all(cli_default.join("plugins")).unwrap();
    std::fs::create_dir_all(cli_default.join("skills")).unwrap();
    std::fs::create_dir_all(cli_default.join("rules")).unwrap();
    std::fs::write(cli_default.join("rules").join("preferences.md"), "rules").unwrap();
    std::fs::create_dir_all(cli_default.join("projects")).unwrap();
    std::fs::write(cli_default.join("history.jsonl"), "").unwrap();
    std::fs::write(desktop_default.join("git-worktrees.json"), "{}").unwrap();
}

#[test]
fn doctor_reports_healthy_share_profile() {
    let (provider, manager, _tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    manager
        .create_profile("env", SharingConfig::share_settings_preset(), None)
        .unwrap();

    let report = manager.inspect_profile_isolation("env").unwrap();

    assert_eq!(
        report.items.len(),
        linker::LINK_ITEMS.len() + 1,
        "every linker-managed link point and the Code-tab list are checked"
    );
    assert_eq!(
        report.issue_count, 0,
        "healthy profile must have no issues: {:?}",
        report.items
    );
    let claude_md = report
        .items
        .iter()
        .find(|i| i.key == "cli_claude_md")
        .unwrap();
    assert!(matches!(claude_md.health, ItemHealth::SharedOk { .. }));
    let app_config = report
        .items
        .iter()
        .find(|i| i.key == "desktop_app_config")
        .unwrap();
    assert!(matches!(app_config.health, ItemHealth::IsolatedOk));
    assert!(!report.running);
}

#[test]
fn doctor_accepts_share_profile_without_sources() {
    // Creating a share profile with an empty default is legal (nothing to
    // share yet); the inspector must not call that drift.
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("env", SharingConfig::share_settings_preset(), None)
        .unwrap();

    let report = manager.inspect_profile_isolation("env").unwrap();
    assert_eq!(
        report.issue_count, 0,
        "missing sources are not issues: {:?}",
        report.items
    );
    // Nothing to link to, so --fix has nothing to create either.
    let fixed = manager.doctor_fix_links("env").unwrap();
    assert!(fixed.is_empty(), "{fixed:?}");
    assert!(
        !manager
            .get_profile("env")
            .unwrap()
            .isolation
            .cli_config_dir
            .join("rules")
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false),
        "no link is created towards an absent source"
    );
}

#[test]
fn doctor_creates_the_missing_rules_link_of_an_environment_from_before_cli_rules() {
    // An environment created before rules/ was a link item: its profile.toml
    // carries no cli_rules key (read through the manager, the load fallback
    // makes it follow the shared CLAUDE.md) and the link was never made. The
    // inspector reports the missing link, and --fix creates it: nothing
    // exists at the link point, so nothing can be lost.
    let (provider, manager, _tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    manager
        .create_profile("env", SharingConfig::share_workspace_preset(), None)
        .unwrap();
    let profile = manager.get_profile("env").unwrap();
    let link = profile.isolation.cli_config_dir.join("rules");
    assert!(provider.is_symlink(&link));
    std::fs::remove_file(&link).unwrap();

    // Rewrite profile.toml the way v0.24.0 wrote it: no cli_rules key at all.
    let toml_path = provider
        .app_data_dir()
        .join("profiles")
        .join("env")
        .join("profile.toml");
    let old_format: String = std::fs::read_to_string(&toml_path)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("cli_rules"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(!old_format.contains("cli_rules"));
    std::fs::write(&toml_path, old_format).unwrap();
    assert_eq!(
        manager.get_profile("env").unwrap().sharing.cli_rules,
        SharingMode::Share,
        "the old profile follows its shared CLAUDE.md"
    );

    let report = manager.inspect_profile_isolation("env").unwrap();
    let rules = report.items.iter().find(|i| i.key == "cli_rules").unwrap();
    assert!(matches!(rules.health, ItemHealth::MissingLink { .. }));
    assert!(rules.is_issue);

    let fixed = manager.doctor_fix_links("env").unwrap();
    assert_eq!(fixed, vec!["cli_rules"]);
    assert!(
        provider.is_symlink(&link),
        "--fix creates the missing share link"
    );
    assert_eq!(
        std::fs::read_link(&link).unwrap(),
        provider.claude_cli_default_dir().join("rules")
    );
    let report = manager.inspect_profile_isolation("env").unwrap();
    assert_eq!(report.issue_count, 0, "{:?}", report.items);
}

#[test]
fn doctor_detects_missing_share_source() {
    let (provider, manager, _tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    manager
        .create_profile("env", SharingConfig::share_settings_preset(), None)
        .unwrap();

    // The shared source disappears after the link was created.
    std::fs::remove_file(provider.claude_cli_default_dir().join("CLAUDE.md")).unwrap();

    let report = manager.inspect_profile_isolation("env").unwrap();
    let claude_md = report
        .items
        .iter()
        .find(|i| i.key == "cli_claude_md")
        .unwrap();
    assert!(matches!(claude_md.health, ItemHealth::SourceMissing { .. }));
    assert!(claude_md.is_issue);
    assert!(report.issue_count >= 1);
}

#[test]
fn doctor_detects_and_fixes_wrong_target() {
    let (provider, manager, _tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    manager
        .create_profile("env", SharingConfig::share_settings_preset(), None)
        .unwrap();

    // Re-point the CLAUDE.md link at a bogus location.
    let profile = manager.get_profile("env").unwrap();
    let link = profile.isolation.cli_config_dir.join("CLAUDE.md");
    std::fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(
        provider.claude_cli_default_dir().join("no-such-file.md"),
        &link,
    )
    .unwrap();

    let report = manager.inspect_profile_isolation("env").unwrap();
    let claude_md = report
        .items
        .iter()
        .find(|i| i.key == "cli_claude_md")
        .unwrap();
    assert!(matches!(
        claude_md.health,
        ItemHealth::WrongTarget { fixable: true, .. }
    ));

    // --fix re-points it; afterwards the profile is healthy again.
    let fixed = manager.doctor_fix_links("env").unwrap();
    assert!(fixed.contains(&"cli_claude_md"));
    let report = manager.inspect_profile_isolation("env").unwrap();
    assert_eq!(report.issue_count, 0, "{:?}", report.items);
}

#[test]
fn doctor_detects_materialized_drift_and_never_fixes_it() {
    let (provider, manager, _tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    manager
        .create_profile("env", SharingConfig::share_settings_preset(), None)
        .unwrap();

    // The share link got replaced by a real file (temp+rename style drift).
    let profile = manager.get_profile("env").unwrap();
    let link = profile.isolation.cli_config_dir.join("CLAUDE.md");
    std::fs::remove_file(&link).unwrap();
    std::fs::write(&link, "materialized local copy").unwrap();

    let report = manager.inspect_profile_isolation("env").unwrap();
    let claude_md = report
        .items
        .iter()
        .find(|i| i.key == "cli_claude_md")
        .unwrap();
    assert!(matches!(
        claude_md.health,
        ItemHealth::Materialized {
            is_directory: false
        }
    ));
    assert!(claude_md.is_issue);

    // --fix must not touch real files: data-loss risk.
    let fixed = manager.doctor_fix_links("env").unwrap();
    assert!(!fixed.contains(&"cli_claude_md"));
    assert_eq!(
        std::fs::read_to_string(&link).unwrap(),
        "materialized local copy",
        "the real file must survive --fix untouched"
    );
}

/// A share environment created while the existing Claude had no `rules/` yet,
/// after which the user created `~/.claude/rules/`. With no source to link to,
/// the linker left an empty real directory at the link point. Returns the
/// environment's `rules` path.
fn share_env_created_before_rules_source(
    provider: &MockPlatformProvider,
    manager: &ProfileManager,
) -> std::path::PathBuf {
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    let source_rules = provider.claude_cli_default_dir().join("rules");
    std::fs::remove_dir_all(&source_rules).unwrap();
    manager
        .create_profile("env", SharingConfig::share_settings_preset(), None)
        .unwrap();
    let rules = manager
        .get_profile("env")
        .unwrap()
        .isolation
        .cli_config_dir
        .join("rules");
    assert!(
        rules.is_dir() && !provider.is_symlink(&rules),
        "the linker leaves an empty directory when there is no source yet"
    );
    std::fs::create_dir_all(&source_rules).unwrap();
    std::fs::write(source_rules.join("preferences.md"), "rules").unwrap();
    rules
}

#[test]
fn doctor_fix_replaces_an_empty_directory_left_before_the_source_existed() {
    // The inspector sees a real directory where the share link belongs while
    // the source exists. --fix removes the directory with rmdir, which fails on
    // any entry, so only an empty directory goes away, and links it instead.
    let (provider, manager, _tmp) = setup_test_manager();
    let rules = share_env_created_before_rules_source(&provider, &manager);

    let report = manager.inspect_profile_isolation("env").unwrap();
    let item = report.items.iter().find(|i| i.key == "cli_rules").unwrap();
    assert!(
        matches!(item.health, ItemHealth::Materialized { is_directory: true }),
        "{:?}",
        item.health
    );
    assert_eq!(report.issue_count, 1, "{:?}", report.items);

    let fixed = manager.doctor_fix_links("env").unwrap();
    assert_eq!(fixed, vec!["cli_rules"]);
    assert!(
        provider.is_symlink(&rules),
        "the empty directory is replaced with the share link"
    );
    assert_eq!(
        std::fs::read_link(&rules).unwrap(),
        provider.claude_cli_default_dir().join("rules")
    );
    let report = manager.inspect_profile_isolation("env").unwrap();
    assert_eq!(report.issue_count, 0, "{:?}", report.items);
}

#[test]
fn doctor_fix_leaves_a_directory_with_contents_untouched() {
    // rmdir fails on any entry, a file or even an empty subdirectory, so --fix
    // leaves such a directory and its contents as they are, and the inspector
    // keeps reporting it.
    for entry_is_dir in [false, true] {
        let (provider, manager, _tmp) = setup_test_manager();
        let rules = share_env_created_before_rules_source(&provider, &manager);
        let entry = rules.join("local");
        if entry_is_dir {
            std::fs::create_dir(&entry).unwrap();
        } else {
            std::fs::write(&entry, "local rule").unwrap();
        }

        let fixed = manager.doctor_fix_links("env").unwrap();
        assert!(fixed.is_empty(), "{fixed:?}");
        assert!(!provider.is_symlink(&rules));
        if entry_is_dir {
            assert!(entry.is_dir(), "the subdirectory must survive --fix");
        } else {
            assert_eq!(
                std::fs::read_to_string(&entry).unwrap(),
                "local rule",
                "the file must survive --fix untouched"
            );
        }
        let report = manager.inspect_profile_isolation("env").unwrap();
        let item = report.items.iter().find(|i| i.key == "cli_rules").unwrap();
        assert!(
            matches!(item.health, ItemHealth::Materialized { is_directory: true }),
            "{:?}",
            item.health
        );
        assert!(item.is_issue);
    }
}

#[test]
fn doctor_report_tells_the_gui_whether_a_materialized_item_is_a_folder() {
    // The GUI (crates/desktop/ui/main.js doctorStatus) reads health.state and
    // health.is_directory to say that `csw doctor --fix` can replace an empty
    // folder, but never a file.
    assert_eq!(
        serde_json::to_value(ItemHealth::Materialized { is_directory: true }).unwrap(),
        serde_json::json!({ "state": "materialized", "is_directory": true })
    );
    assert_eq!(
        serde_json::to_value(ItemHealth::Materialized {
            is_directory: false
        })
        .unwrap(),
        serde_json::json!({ "state": "materialized", "is_directory": false })
    );
}

#[test]
fn doctor_fix_restores_the_empty_directory_when_the_link_cannot_be_created() {
    let (provider, manager, _tmp) = setup_test_manager();
    let rules = share_env_created_before_rules_source(&provider, &manager);
    let profile = manager.get_profile("env").unwrap();
    let source_profile = manager.get_profile("default").unwrap();

    // Same directories, but creating a symlink fails.
    let failing = MockPlatformProvider::new(
        provider.claude_desktop_default_dir(),
        provider.claude_cli_default_dir(),
        provider.app_data_dir(),
    )
    .with_symlink_failure(true);
    let result = crate::profile::inspector::Inspector::new(&failing)
        .fix_relinkable(&profile, &source_profile);

    assert!(result.is_err(), "a failed symlink must surface an error");
    assert!(
        rules.is_dir() && !provider.is_symlink(&rules),
        "the empty directory is put back, not left missing"
    );
}

#[test]
fn doctor_detects_unexpected_link_on_always_isolated_item() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("env", SharingConfig::default(), None)
        .unwrap();

    // Someone (or a bug) turned the always-isolated config.json into a link.
    std::fs::write(
        provider.claude_desktop_default_dir().join("config.json"),
        "{}",
    )
    .unwrap();
    let profile = manager.get_profile("env").unwrap();
    let path = profile.isolation.desktop_user_data_dir.join("config.json");
    if path.exists() {
        std::fs::remove_file(&path).unwrap();
    }
    std::os::unix::fs::symlink(
        provider.claude_desktop_default_dir().join("config.json"),
        &path,
    )
    .unwrap();

    let report = manager.inspect_profile_isolation("env").unwrap();
    let app_config = report
        .items
        .iter()
        .find(|i| i.key == "desktop_app_config")
        .unwrap();
    assert!(matches!(
        app_config.health,
        ItemHealth::UnexpectedLink { .. }
    ));
    assert!(app_config.is_issue);

    // --fix only re-points share links; it must not silently remove this.
    let fixed = manager.doctor_fix_links("env").unwrap();
    assert!(fixed.is_empty());
    assert!(
        provider.is_symlink(&path),
        "detected, reported, not auto-removed"
    );
}

#[test]
fn doctor_rejects_default_profile() {
    let (_, manager, _tmp) = setup_test_manager();
    assert!(manager.inspect_profile_isolation("default").is_err());
    assert!(manager.doctor_fix_links("default").is_err());
}

#[test]
fn doctor_flags_running_environment() {
    let tmp_dir = tempdir().unwrap();
    let app_data = tmp_dir.path().join("app_data");
    let desktop_default = tmp_dir.path().join("desktop_default");
    let cli_default = tmp_dir.path().join("cli_default");
    std::fs::create_dir_all(&app_data).unwrap();
    std::fs::create_dir_all(&desktop_default).unwrap();
    std::fs::create_dir_all(&cli_default).unwrap();

    // First pass: create the environment.
    let provider = Arc::new(MockPlatformProvider::new(
        desktop_default.clone(),
        cli_default.clone(),
        app_data.clone(),
    ));
    let manager = ProfileManager::new(provider.clone()).unwrap();
    manager
        .create_profile("env", SharingConfig::default(), None)
        .unwrap();
    let desktop_dir = manager
        .get_profile("env")
        .unwrap()
        .isolation
        .desktop_user_data_dir;

    // Second pass: same app data, but a Claude main process now runs on it.
    let provider = Arc::new(
        MockPlatformProvider::new(desktop_default, cli_default, app_data).with_running_args(vec![
            format!(
                "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={}",
                desktop_dir.display()
            ),
        ]),
    );
    let manager = ProfileManager::new(provider).unwrap();
    let report = manager.inspect_profile_isolation("env").unwrap();
    assert!(report.running, "a running environment must be flagged");
}

// --- environment data map ----------------------------------------------------

use crate::profile::data_map::DataMap;

fn data_map_setup() -> (
    Arc<MockPlatformProvider>,
    ProfileManager,
    tempfile::TempDir,
    DataMap,
) {
    let (provider, manager, tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    // A big shared source: if the walk ever followed links, totals would jump.
    std::fs::write(
        provider.claude_cli_default_dir().join("CLAUDE.md"),
        vec![b'x'; 4096],
    )
    .unwrap();
    manager
        .create_profile("env", SharingConfig::share_settings_preset(), None)
        .unwrap();

    let profile = manager.get_profile("env").unwrap();
    // Real isolated data owned by the environment: 100 bytes.
    std::fs::write(
        profile
            .isolation
            .cli_config_dir
            .join("projects")
            .join("a.jsonl"),
        vec![b'y'; 100],
    )
    .unwrap();
    // A stray symlink inside the environment pointing at a big external file.
    let big = tmp.path().join("big.bin");
    std::fs::write(&big, vec![0u8; 8192]).unwrap();
    std::os::unix::fs::symlink(
        &big,
        profile
            .isolation
            .cli_config_dir
            .join("projects")
            .join("link.bin"),
    )
    .unwrap();

    let map = manager.profile_data_map("env").unwrap();
    (provider, manager, tmp, map)
}

#[test]
fn data_map_does_not_follow_symlinks() {
    let (_, _, _tmp, map) = data_map_setup();

    // The 100-byte real file must be counted; the 4096-byte shared source and
    // the 8192-byte stray link target must not be (links are never followed).
    assert!(map.cli_size_bytes >= 100, "{:?}", map);
    assert!(
        map.cli_size_bytes < 4096,
        "symlink targets must never be followed: {} bytes",
        map.cli_size_bytes
    );
    assert_eq!(
        map.total_size_bytes,
        map.cli_size_bytes + map.desktop_size_bytes
    );
}

#[test]
fn data_map_reports_share_targets_without_size() {
    let (provider, _, _tmp, map) = data_map_setup();

    let claude_md = map.items.iter().find(|i| i.key == "cli_claude_md").unwrap();
    assert_eq!(
        claude_md.link_target,
        Some(
            provider
                .claude_cli_default_dir()
                .join("CLAUDE.md")
                .display()
                .to_string()
        )
    );
    assert!(
        claude_md.size_bytes.is_none(),
        "share items show the target, not a size"
    );
    assert!(claude_md.exists);
}

#[test]
fn data_map_sign_in_item_is_metadata_only() {
    let (_, _, _tmp, map) = data_map_setup();

    // Minimal exposure for the sign-in item: presence only, no size, no mtime.
    let sign_in = map
        .items
        .iter()
        .find(|i| i.key == "desktop_app_config")
        .unwrap();
    assert!(sign_in.size_bytes.is_none());
    assert!(sign_in.modified_epoch.is_none());
    assert!(sign_in.link_target.is_none());
}

#[test]
fn data_map_counts_isolated_items() {
    let (_, _, _tmp, map) = data_map_setup();

    let projects = map
        .items
        .iter()
        .find(|i| i.key == "cli_project_memory")
        .unwrap();
    assert!(projects.size_bytes.unwrap_or(0) >= 100, "{:?}", projects);
    assert!(projects.modified_epoch.is_some());
}

#[test]
fn data_map_rejects_default() {
    let (_, manager, _tmp) = setup_test_manager();
    assert!(manager.profile_data_map("default").is_err());
}

#[test]
fn data_map_scales_to_thousands_of_files() {
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("env", SharingConfig::default(), None)
        .unwrap();
    let profile = manager.get_profile("env").unwrap();
    let dir = profile.isolation.cli_config_dir.join("projects");
    for i in 0..5000 {
        std::fs::write(dir.join(format!("f{i}.jsonl")), b"0123456789").unwrap();
    }

    let started = std::time::Instant::now();
    let map = manager.profile_data_map("env").unwrap();
    let elapsed = started.elapsed();
    println!("data_map over 5000 files took {elapsed:?}");

    assert!(map.cli_size_bytes >= 50_000);
    // Generous bound: catches pathological recursion, not normal variance.
    assert!(elapsed.as_secs() < 5, "took {elapsed:?}");
}

#[test]
fn get_profile_rejects_traversal_names() {
    let (_, manager, _tmp) = setup_test_manager();
    assert!(manager.get_profile("../escape").is_err());
    assert!(manager.profile_data_map("../escape").is_err());
}

#[test]
fn data_map_sizes_materialized_share_item() {
    let (provider, manager, _tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    manager
        .create_profile("env", SharingConfig::share_settings_preset(), None)
        .unwrap();

    // The share link drifted into a real local file: it occupies real bytes,
    // so the item row must report them (consistent with the folder totals).
    let profile = manager.get_profile("env").unwrap();
    let link = profile.isolation.cli_config_dir.join("CLAUDE.md");
    std::fs::remove_file(&link).unwrap();
    std::fs::write(&link, vec![b'm'; 777]).unwrap();

    let map = manager.profile_data_map("env").unwrap();
    let claude_md = map.items.iter().find(|i| i.key == "cli_claude_md").unwrap();
    assert!(claude_md.link_target.is_none());
    assert_eq!(claude_md.size_bytes, Some(777));
}

// --- duplicate (clone) --------------------------------------------------------
//
// Duplicating an environment copies its folder as it is (docs/PRIVACY.md), so
// every item the user keeps in the environment (isolated or copied) must
// arrive in the duplicate as real data, and every shared item must be a link
// to the declared sharing source (the existing Claude), never to the source
// environment's own link point.

#[test]
fn clone_copies_isolated_and_copied_items_and_relinks_shares_to_the_declared_source() {
    let (provider, manager, _tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    std::fs::write(
        provider
            .claude_cli_default_dir()
            .join("skills")
            .join("base.md"),
        "shared skill",
    )
    .unwrap();

    let sharing = SharingConfig {
        cli_claude_md: SharingMode::Share,
        cli_rules: SharingMode::Isolate,
        cli_plugins: SharingMode::Share,
        cli_skills: SharingMode::Copy,
        cli_settings: SharingMode::Isolate,
        cli_project_memory: SharingMode::Isolate,
        cli_history: SharingMode::Isolate,
        desktop_worktrees: SharingMode::Isolate,
        desktop_ssh_connections: SharingMode::Isolate,
        ..SharingConfig::default()
    };
    let orig = manager.create_profile("orig", sharing, None).unwrap();
    let cli = &orig.isolation.cli_config_dir;
    let desk = &orig.isolation.desktop_user_data_dir;

    // Real data the user accumulated inside the isolated / copied items.
    std::fs::create_dir_all(cli.join("projects").join("p1")).unwrap();
    std::fs::write(
        cli.join("projects").join("p1").join("conv.jsonl"),
        "history",
    )
    .unwrap();
    std::fs::write(cli.join("settings.json"), "{\"hooks\":{}}").unwrap();
    std::fs::write(cli.join("history.jsonl"), "prompt\n").unwrap();
    std::fs::write(cli.join("skills").join("mine.md"), "own skill").unwrap();
    std::fs::write(cli.join("rules").join("style.md"), "own rule").unwrap();
    std::fs::write(desk.join("git-worktrees.json"), "{\"wt\":1}").unwrap();
    // Data outside the linker's items travels too (sign-in and app state).
    std::fs::write(cli.join(".credentials.json"), "cli-login").unwrap();
    std::fs::write(desk.join("Cookies"), "desktop-login").unwrap();

    let clone = manager.clone_profile("orig", "copy").unwrap();
    let ccli = &clone.isolation.cli_config_dir;
    let cdesk = &clone.isolation.desktop_user_data_dir;

    // Isolated and copied items arrive as real data.
    for (path, expected) in [
        (
            ccli.join("projects").join("p1").join("conv.jsonl"),
            "history",
        ),
        (ccli.join("settings.json"), "{\"hooks\":{}}"),
        (ccli.join("history.jsonl"), "prompt\n"),
        (ccli.join("skills").join("mine.md"), "own skill"),
        (ccli.join("skills").join("base.md"), "shared skill"),
        (ccli.join("rules").join("style.md"), "own rule"),
        (cdesk.join("git-worktrees.json"), "{\"wt\":1}"),
        (ccli.join(".credentials.json"), "cli-login"),
        (cdesk.join("Cookies"), "desktop-login"),
    ] {
        assert_eq!(
            std::fs::read_to_string(&path).unwrap_or_default(),
            expected,
            "{} must be duplicated as real data",
            path.display()
        );
        assert!(
            !provider.is_symlink(&path),
            "{} must not be a link",
            path.display()
        );
    }
    assert!(!provider.is_symlink(&ccli.join("projects")));
    assert!(!provider.is_symlink(&ccli.join("skills")));

    // Shared items link to the declared source, not to the source environment.
    for (item, source) in [("CLAUDE.md", "CLAUDE.md"), ("plugins", "plugins")] {
        let link = ccli.join(item);
        assert!(provider.is_symlink(&link), "{item} must stay shared");
        assert_eq!(
            std::fs::read_link(&link).unwrap(),
            provider.claude_cli_default_dir().join(source),
            "{item} must point at the existing Claude, not at the source environment"
        );
    }

    // Always-isolated items start fresh: never copied from the source.
    assert!(!cdesk.join("config.json").exists());
    assert!(!cdesk.join("claude_desktop_config.json").exists());
    assert!(!cdesk.join("ant-did").exists());
    assert!(ccli.join("sessions").is_dir());
    assert_eq!(std::fs::read_dir(ccli.join("sessions")).unwrap().count(), 0);

    // The duplicate is healthy right away and its sharing config is inherited.
    let report = manager.inspect_profile_isolation("copy").unwrap();
    assert_eq!(report.issue_count, 0, "{:?}", report.items);
    assert_eq!(clone.sharing.cli_skills, SharingMode::Copy);
    assert_eq!(clone.profile.cloned_from.as_deref(), Some("orig"));
}

#[test]
fn clone_of_fully_isolated_environment_keeps_its_conversations_rules_and_settings() {
    // The user's own environment data must survive duplication even when
    // nothing is shared: an empty projects/ or a missing CLAUDE.md would be
    // silent data loss for the copy.
    let (provider, manager, _tmp) = setup_test_manager();
    let orig = manager
        .create_profile("iso", SharingConfig::default(), None)
        .unwrap();
    let cli = &orig.isolation.cli_config_dir;
    std::fs::write(cli.join("projects").join("conv.jsonl"), "hello").unwrap();
    std::fs::write(cli.join("CLAUDE.md"), "my rules").unwrap();
    std::fs::write(cli.join("settings.json"), "{}").unwrap();
    std::fs::write(cli.join("history.jsonl"), "h").unwrap();
    std::fs::write(cli.join("skills").join("s.md"), "s").unwrap();
    std::fs::write(cli.join("rules").join("r.md"), "r").unwrap();
    std::fs::write(cli.join("plugins").join("p.js"), "p").unwrap();

    let clone = manager.clone_profile("iso", "iso2").unwrap();
    let c = &clone.isolation.cli_config_dir;
    for (path, expected) in [
        (c.join("projects").join("conv.jsonl"), "hello"),
        (c.join("CLAUDE.md"), "my rules"),
        (c.join("settings.json"), "{}"),
        (c.join("history.jsonl"), "h"),
        (c.join("skills").join("s.md"), "s"),
        (c.join("rules").join("r.md"), "r"),
    ] {
        assert_eq!(
            std::fs::read_to_string(&path).unwrap_or_default(),
            expected,
            "{} must be duplicated",
            path.display()
        );
        assert!(!provider.is_symlink(&path));
    }
    // plugins/ is the one item never taken from the source environment: its
    // registry files carry absolute paths into that environment's cache. An
    // isolated duplicate starts with an empty plugins/ of its own.
    let plugins = c.join("plugins");
    assert!(plugins.is_dir());
    assert!(!provider.is_symlink(&plugins));
    assert_eq!(std::fs::read_dir(&plugins).unwrap().count(), 0);
    assert!(clone.sharing.is_fully_isolated());
    let report = manager.inspect_profile_isolation("iso2").unwrap();
    assert_eq!(report.issue_count, 0, "{:?}", report.items);
}

#[test]
fn clone_sets_up_copied_plugins_from_the_existing_claude_not_the_source_environment() {
    let (provider, manager, _tmp) = setup_test_manager();
    let default_plugins = provider.claude_cli_default_dir().join("plugins");
    std::fs::create_dir_all(&default_plugins).unwrap();
    std::fs::write(default_plugins.join("base.js"), "base").unwrap();

    let sharing = SharingConfig {
        cli_plugins: SharingMode::Copy,
        ..SharingConfig::default()
    };
    let orig = manager.create_profile("orig", sharing, None).unwrap();
    // The source environment's plugins/ diverged after creation (it is a copy).
    std::fs::write(
        orig.isolation
            .cli_config_dir
            .join("plugins")
            .join("mine.js"),
        "mine",
    )
    .unwrap();

    let clone = manager.clone_profile("orig", "copy").unwrap();
    let plugins = clone.isolation.cli_config_dir.join("plugins");
    assert!(
        !provider.is_symlink(&plugins),
        "copy means a real directory"
    );
    assert!(
        plugins.join("base.js").exists(),
        "a copied plugins/ is set up again from the existing Claude"
    );
    assert!(
        !plugins.join("mine.js").exists(),
        "the source environment's plugin state is never copied (absolute paths inside)"
    );
}

// --- trash on delete ----------------------------------------------------------

#[test]
fn delete_profile_moves_to_trash_and_keeps_data() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("bin-me", SharingConfig::default(), None)
        .unwrap();
    let profile = manager.get_profile("bin-me").unwrap();
    std::fs::write(
        profile
            .isolation
            .cli_config_dir
            .join("projects")
            .join("keep.jsonl"),
        "precious",
    )
    .unwrap();

    manager.delete_profile("bin-me").unwrap();

    assert!(
        !manager
            .list_profiles()
            .unwrap()
            .contains(&"bin-me".to_string())
    );
    let trashed = provider.mock_trash_dir().join("bin-me");
    assert!(trashed.exists(), "the folder must land in the trash");
    assert_eq!(
        std::fs::read_to_string(trashed.join("cli-data").join("projects").join("keep.jsonl"))
            .unwrap(),
        "precious",
        "trashed data must stay intact and restorable"
    );
}

#[test]
fn restored_profile_is_recognized_again() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("bin-me", SharingConfig::default(), None)
        .unwrap();
    manager.delete_profile("bin-me").unwrap();

    // The documented restore procedure: move the folder back under profiles/.
    let profiles_dir = provider.app_data_dir().join("profiles");
    std::fs::rename(
        provider.mock_trash_dir().join("bin-me"),
        profiles_dir.join("bin-me"),
    )
    .unwrap();

    assert!(
        manager
            .list_profiles()
            .unwrap()
            .contains(&"bin-me".to_string())
    );
    assert_eq!(
        manager.get_profile("bin-me").unwrap().profile.name,
        "bin-me"
    );
}

#[test]
fn purge_profile_deletes_permanently() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("gone", SharingConfig::default(), None)
        .unwrap();

    manager.purge_profile("gone").unwrap();

    assert!(
        !manager
            .list_profiles()
            .unwrap()
            .contains(&"gone".to_string())
    );
    assert!(
        !provider.mock_trash_dir().join("gone").exists(),
        "purge must not go through the trash"
    );
}

#[test]
fn delete_profile_errors_without_fallback_when_trash_fails() {
    let tmp_dir = tempdir().unwrap();
    let app_data = tmp_dir.path().join("app_data");
    let desktop_default = tmp_dir.path().join("desktop_default");
    let cli_default = tmp_dir.path().join("cli_default");
    std::fs::create_dir_all(&app_data).unwrap();
    std::fs::create_dir_all(&desktop_default).unwrap();
    std::fs::create_dir_all(&cli_default).unwrap();
    let provider = Arc::new(
        MockPlatformProvider::new(desktop_default, cli_default, app_data.clone())
            .with_trash_failure(true),
    );
    let manager = ProfileManager::new(provider).unwrap();
    manager
        .create_profile("stuck", SharingConfig::default(), None)
        .unwrap();

    let res = manager.delete_profile("stuck");

    assert!(
        res.is_err(),
        "trash failure must surface, never silently purge"
    );
    assert!(
        app_data.join("profiles").join("stuck").exists(),
        "the environment must survive a failed trash move"
    );
}

#[test]
fn delete_profile_trashes_share_links_without_touching_originals() {
    let (provider, manager, _tmp) = setup_test_manager();
    populate_default_sources(
        &provider.claude_cli_default_dir(),
        &provider.claude_desktop_default_dir(),
    );
    manager
        .create_profile("linked", SharingConfig::share_settings_preset(), None)
        .unwrap();

    manager.delete_profile("linked").unwrap();

    // The link moved to the trash as a link; the shared original is untouched,
    // so restoring the folder brings the environment back completely.
    let trashed_link = provider
        .mock_trash_dir()
        .join("linked")
        .join("cli-data")
        .join("CLAUDE.md");
    assert!(provider.is_symlink(&trashed_link));
    assert!(
        provider.claude_cli_default_dir().join("CLAUDE.md").exists(),
        "the existing Claude's original must never be affected"
    );
}

#[test]
fn purge_profile_rejects_default_and_active() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("busy", SharingConfig::default(), None)
        .unwrap();
    manager.switch_to("busy").unwrap();

    // The worst possible regression is purging the existing Claude or the
    // active environment; pin both rejections on the purge path itself.
    assert!(manager.purge_profile("default").is_err());
    assert!(manager.purge_profile("busy").is_err());
    assert!(
        provider
            .app_data_dir()
            .join("profiles")
            .join("busy")
            .exists(),
        "a rejected purge must leave the environment untouched"
    );
}

// Issue #223: deleting an environment must depend on live usage, not only
// active_profile. All paths and the mock trash remain inside a TempDir.
fn assert_deletion_refused_without_changes(purge: bool, inspection_fails: bool) {
    for name in ["busy", "仕事"] {
        for sharing in [
            SharingConfig::default(),
            SharingConfig::share_settings_preset(),
        ] {
            let (initial_provider, initial_manager, _tmp) = setup_test_manager();
            populate_default_sources(
                &initial_provider.claude_cli_default_dir(),
                &initial_provider.claude_desktop_default_dir(),
            );
            let profile = initial_manager.create_profile(name, sharing, None).unwrap();
            let desktop_data = profile.isolation.desktop_user_data_dir.join("keep.txt");
            let cli_data = profile.isolation.cli_config_dir.join("projects/keep.jsonl");
            fs::write(&desktop_data, "desktop data").unwrap();
            fs::write(&cli_data, "cli data").unwrap();
            let profile_dir = initial_provider.app_data_dir().join("profiles").join(name);
            let metadata = fs::read(profile_dir.join("profile.toml")).unwrap();
            let shared_link = profile.isolation.cli_config_dir.join("CLAUDE.md");
            let original_link = fs::read_link(&shared_link).ok();
            assert_eq!(
                original_link.is_some(),
                profile.sharing.cli_claude_md == SharingMode::Share,
                "the shared fixture must contain a link before deletion is attempted",
            );
            let source = initial_provider.claude_cli_default_dir().join("CLAUDE.md");
            let original_source = fs::read(&source).unwrap();

            let provider = Arc::new(
                MockPlatformProvider::new(
                    initial_provider.claude_desktop_default_dir(),
                    initial_provider.claude_cli_default_dir(),
                    initial_provider.app_data_dir(),
                )
                .with_running_args(vec![
                    "/Applications/Claude.app/Contents/MacOS/Claude".into(),
                    format!(
                        "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={}",
                        profile.isolation.desktop_user_data_dir.display()
                    ),
                ])
                .with_running_args_failure(inspection_fails),
            );
            let manager = ProfileManager::new(provider.clone()).unwrap();
            assert_eq!(manager.active_profile_name(), "default");
            if !inspection_fails {
                assert!(
                    crate::switcher::desktop_dir_running(
                        &profile.isolation.desktop_user_data_dir,
                        &provider.running_desktop_args().unwrap(),
                        &provider.claude_desktop_default_dir(),
                    ),
                    "the fixture must really match the running target"
                );
            }

            let result = if purge {
                manager.purge_profile(name)
            } else {
                manager.delete_profile(name)
            };
            assert!(
                result.is_err(),
                "running/unknown environment must survive (purge={purge})"
            );
            if inspection_fails {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("process inspection failed")
                );
            } else {
                assert!(
                    matches!(result.unwrap_err(), CswError::RunningProfileCannotBeDeleted(refused_name) if refused_name == name)
                );
            }
            assert_eq!(fs::read_to_string(&desktop_data).unwrap(), "desktop data");
            assert_eq!(fs::read_to_string(&cli_data).unwrap(), "cli data");
            assert_eq!(
                fs::read(profile_dir.join("profile.toml")).unwrap(),
                metadata
            );
            assert_eq!(fs::read_link(&shared_link).ok(), original_link);
            assert_eq!(fs::read(&source).unwrap(), original_source);
            assert!(!provider.mock_trash_dir().exists());
            assert_eq!(manager.active_profile_name(), "default");
        }
    }
}

#[test]
fn delete_running_non_active_profile_is_refused() {
    assert_deletion_refused_without_changes(false, false);
}

#[test]
fn purge_running_non_active_profile_is_refused() {
    assert_deletion_refused_without_changes(true, false);
}

#[test]
fn delete_profile_is_refused_when_process_inspection_fails() {
    assert_deletion_refused_without_changes(false, true);
}

#[test]
fn purge_profile_is_refused_when_process_inspection_fails() {
    assert_deletion_refused_without_changes(true, true);
}

#[test]
fn deletion_allows_stopped_target_while_other_environments_run() {
    for purge in [false, true] {
        let (initial_provider, manager, _tmp) = setup_test_manager();
        let target = manager
            .create_profile("stopped", SharingConfig::default(), None)
            .unwrap();
        let other = manager
            .create_profile("other", SharingConfig::default(), None)
            .unwrap();
        let other_data = other.isolation.desktop_user_data_dir.join("keep.txt");
        fs::write(&other_data, "other environment").unwrap();
        let provider = Arc::new(
            MockPlatformProvider::new(
                initial_provider.claude_desktop_default_dir(),
                initial_provider.claude_cli_default_dir(),
                initial_provider.app_data_dir(),
            )
            .with_desktop_running(true)
            .with_running_args(vec![
                "/Applications/Claude.app/Contents/MacOS/Claude".into(),
                format!(
                    "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={}",
                    other.isolation.desktop_user_data_dir.display()
                ),
                // A directory with the target as a string prefix is not the target.
                format!(
                    "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={}-backup",
                    target.isolation.desktop_user_data_dir.display()
                ),
            ]),
        );
        let manager = ProfileManager::new(provider.clone()).unwrap();
        if purge {
            manager.purge_profile("stopped").unwrap();
        } else {
            manager.delete_profile("stopped").unwrap();
        }
        assert!(!target.isolation.desktop_user_data_dir.exists());
        assert_eq!(provider.mock_trash_dir().join("stopped").exists(), !purge);
        assert_eq!(fs::read_to_string(other_data).unwrap(), "other environment");
        assert!(manager.get_profile("other").is_ok());
    }
}

// ---- Notes, history and last launch ----
// Spec: docs/proposals/environment-notes-and-last-launch.md. Expectations are
// derived from the plan, not from the implementation (RED first).

#[test]
fn legacy_profile_toml_without_new_fields_still_loads() {
    let (_, manager, _tmp) = setup_test_manager();
    let created = manager
        .create_profile("legacy", SharingConfig::default(), None)
        .unwrap();
    // Rewrite profile.toml in the v0.19.1 shape: no note / created_at /
    // cloned_from. Existing environments on disk must keep loading.
    let toml_path = created
        .isolation
        .desktop_user_data_dir
        .parent()
        .unwrap()
        .join("profile.toml");
    let legacy = format!(
        "[profile]\nname = \"legacy\"\nicon = \"\"\ncolor = \"#4A90D9\"\nis_default = false\n\n\
         [isolation]\ndesktop_user_data_dir = \"{}\"\ncli_config_dir = \"{}\"\n\n\
         [sharing]\n[sharing.source]\nprofile = \"default\"\n",
        created.isolation.desktop_user_data_dir.display(),
        created.isolation.cli_config_dir.display()
    );
    std::fs::write(&toml_path, legacy).unwrap();

    let p = manager.get_profile("legacy").unwrap();
    assert_eq!(p.profile.note, "");
    assert!(p.profile.created_at.is_none());
    assert!(p.profile.cloned_from.is_none());
}

#[test]
fn create_profile_stamps_created_at() {
    let (_, manager, _tmp) = setup_test_manager();
    let p = manager
        .create_profile("stamped", SharingConfig::default(), None)
        .unwrap();
    assert!(p.profile.created_at.is_some());
    assert!(p.profile.cloned_from.is_none());

    let reloaded = manager.get_profile("stamped").unwrap();
    assert!(reloaded.profile.created_at.is_some());
}

#[test]
fn clone_profile_stamps_cloned_from_and_carries_note() {
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("origin", SharingConfig::default(), None)
        .unwrap();
    manager
        .set_profile_note("origin", "研究室の共有アカウントでサインイン")
        .unwrap();

    let cloned = manager.clone_profile("origin", "copy1").unwrap();
    assert_eq!(cloned.profile.cloned_from.as_deref(), Some("origin"));
    assert!(cloned.profile.created_at.is_some());
    // SPECIFICATION §5.A: the duplicate starts with the source's note.
    assert_eq!(cloned.profile.note, "研究室の共有アカウントでサインイン");

    let reloaded = manager.get_profile("copy1").unwrap();
    assert_eq!(reloaded.profile.cloned_from.as_deref(), Some("origin"));
    assert_eq!(reloaded.profile.note, "研究室の共有アカウントでサインイン");
}

#[test]
fn set_profile_note_saves_overwrites_and_clears() {
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("noted", SharingConfig::default(), None)
        .unwrap();

    manager
        .set_profile_note("noted", "B社納品用。b-corp の Google でサインイン")
        .unwrap();
    assert_eq!(
        manager.get_profile("noted").unwrap().profile.note,
        "B社納品用。b-corp の Google でサインイン"
    );

    manager.set_profile_note("noted", "個人用に変更").unwrap();
    assert_eq!(
        manager.get_profile("noted").unwrap().profile.note,
        "個人用に変更"
    );

    manager.set_profile_note("noted", "").unwrap();
    assert_eq!(manager.get_profile("noted").unwrap().profile.note, "");
}

#[test]
fn set_profile_note_rejects_default_too_long_and_control_chars() {
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("guard", SharingConfig::default(), None)
        .unwrap();

    // The default profile is synthetic; there is no profile.toml to hold a note.
    assert!(manager.set_profile_note("default", "x").is_err());

    let too_long = "あ".repeat(201);
    assert!(manager.set_profile_note("guard", &too_long).is_err());
    // Single-line spec: newlines and other control characters are rejected.
    assert!(manager.set_profile_note("guard", "line1\nline2").is_err());
    assert!(manager.set_profile_note("guard", "tab\there").is_err());

    let exactly_200 = "あ".repeat(200);
    assert!(manager.set_profile_note("guard", &exactly_200).is_ok());
}

#[test]
fn record_last_launch_writes_state_and_leaves_profile_toml_untouched() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("launchy", SharingConfig::default(), None)
        .unwrap();
    let profile_dir = provider.app_data_dir().join("profiles").join("launchy");
    let toml_before = std::fs::read(profile_dir.join("profile.toml")).unwrap();

    assert_eq!(manager.last_launched_at("launchy").unwrap(), None);

    manager.record_last_launch("launchy").unwrap();
    let ts = manager.last_launched_at("launchy").unwrap();
    assert!(ts.is_some());
    assert!(!ts.unwrap().is_empty());
    assert!(
        profile_dir.join("state.toml").exists(),
        "launch state lives in its own file, not in profile.toml"
    );

    // The safety declaration (profile.toml) must not be rewritten on launch.
    let toml_after = std::fs::read(profile_dir.join("profile.toml")).unwrap();
    assert_eq!(toml_before, toml_after);
}

#[test]
fn record_last_launch_rejects_default() {
    let (_, manager, _tmp) = setup_test_manager();
    assert!(manager.record_last_launch("default").is_err());
    assert_eq!(manager.last_launched_at("default").unwrap(), None);
}

// ---- First launch (sign-in signpost) ----
// Spec: docs/proposals/first-launch-signpost.md.

#[test]
fn first_record_stamps_first_and_last_equally() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("fresh", SharingConfig::default(), None)
        .unwrap();

    assert_eq!(manager.first_launched_at("fresh").unwrap(), None);

    manager.record_last_launch("fresh").unwrap();
    let first = manager.first_launched_at("fresh").unwrap();
    let last = manager.last_launched_at("fresh").unwrap();
    assert!(first.is_some());
    assert_eq!(first, last, "the very first launch is both first and last");

    // The stamp lives in state.toml, never in the safety declaration.
    let profile_dir = provider.app_data_dir().join("profiles").join("fresh");
    assert!(profile_dir.join("state.toml").exists());
}

#[test]
fn second_record_moves_last_but_keeps_first() {
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("again", SharingConfig::default(), None)
        .unwrap();

    manager.record_last_launch("again").unwrap();
    let first = manager.first_launched_at("again").unwrap();
    // Stamps carry millisecond precision, so a short pause is enough for the
    // second launch to get a distinct timestamp.
    std::thread::sleep(std::time::Duration::from_millis(30));
    manager.record_last_launch("again").unwrap();

    assert_eq!(
        manager.first_launched_at("again").unwrap(),
        first,
        "the first-launch stamp must never move"
    );
    assert_ne!(manager.last_launched_at("again").unwrap(), first);
}

#[test]
fn clone_does_not_carry_launch_state() {
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("mother", SharingConfig::default(), None)
        .unwrap();
    manager.record_last_launch("mother").unwrap();

    // The launch state belongs to each environment and does not travel with
    // the copy: the duplicate's "last launched" starts empty.
    let cloned = manager.clone_profile("mother", "child").unwrap();
    assert_eq!(cloned.profile.name, "child");
    assert_eq!(manager.first_launched_at("child").unwrap(), None);
    assert_eq!(manager.last_launched_at("child").unwrap(), None);
}

#[test]
fn signin_signpost_is_due_only_at_the_first_launch() {
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("newcomer", SharingConfig::default(), None)
        .unwrap();
    assert!(
        !manager.signin_signpost_due("newcomer").unwrap(),
        "not before the first launch"
    );

    manager.record_last_launch("newcomer").unwrap();
    assert!(
        manager.signin_signpost_due("newcomer").unwrap(),
        "due right after the first launch"
    );

    std::thread::sleep(std::time::Duration::from_millis(30));
    manager.record_last_launch("newcomer").unwrap();
    assert!(
        !manager.signin_signpost_due("newcomer").unwrap(),
        "gone from the second launch on"
    );
    assert!(!manager.signin_signpost_due("default").unwrap());
}

#[test]
fn signin_signpost_is_not_due_for_a_duplicate() {
    // A duplicate starts with the source's data, including the desktop app's
    // sign-in cookies, so its first launch opens signed in to the source's
    // account. The card's "an empty Claude asks you to sign in" would be false.
    let (_, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("source-env", SharingConfig::default(), None)
        .unwrap();
    manager.record_last_launch("source-env").unwrap();
    manager.clone_profile("source-env", "dup-env").unwrap();

    manager.record_last_launch("dup-env").unwrap();
    assert!(!manager.signin_signpost_due("dup-env").unwrap());
}

#[test]
fn state_toml_with_only_last_launched_still_loads() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("legacy2", SharingConfig::default(), None)
        .unwrap();
    // The shape a build with only PR #159 would write: last only, no first.
    let profile_dir = provider.app_data_dir().join("profiles").join("legacy2");
    std::fs::write(
        profile_dir.join("state.toml"),
        "last_launched_at = \"2026-07-03T00:00:00Z\"\n",
    )
    .unwrap();

    assert_eq!(
        manager.last_launched_at("legacy2").unwrap().as_deref(),
        Some("2026-07-03T00:00:00Z")
    );
    assert_eq!(manager.first_launched_at("legacy2").unwrap(), None);

    // A launched-before environment must not get a backfilled first launch:
    // its user is already signed in and must never see the signpost.
    manager.record_last_launch("legacy2").unwrap();
    assert_eq!(manager.first_launched_at("legacy2").unwrap(), None);
    assert!(manager.last_launched_at("legacy2").unwrap().is_some());
}

#[test]
fn launch_desktop_stamps_last_launch_but_never_for_default() {
    let (provider, manager, _tmp) = setup_test_manager();
    manager
        .create_profile("runner", SharingConfig::default(), None)
        .unwrap();
    let runner = manager.get_profile("runner").unwrap();
    crate::switcher::desktop::launch_desktop(&runner, provider.as_ref()).unwrap();
    assert!(manager.last_launched_at("runner").unwrap().is_some());

    // Zero-impact: launching the default environment must not drop a
    // state.toml next to the user's real Claude data directory.
    let default = manager.get_profile("default").unwrap();
    crate::switcher::desktop::launch_desktop(&default, provider.as_ref()).unwrap();
    let default_dir = provider.claude_desktop_default_dir();
    let real_parent = default_dir.parent().unwrap();
    assert!(!real_parent.join("state.toml").exists());
    assert_eq!(manager.last_launched_at("default").unwrap(), None);
}

// --- Code-tab conversation list (SPECIFICATION.md §3「Codeタブの会話一覧」) ---

const CS_ACCT_A: &str = "11111111-1111-4111-8111-111111111111";
const CS_ORG_A: &str = "22222222-2222-4222-8222-222222222222";
const CS_ACCT_B: &str = "33333333-3333-4333-8333-333333333333";
const CS_ORG_B: &str = "44444444-4444-4444-8444-444444444444";

/// `<desktop>/claude-code-sessions/<acct>/<org>/` holding one list entry.
fn cs_account_dir(desktop: &std::path::Path, acct: &str, org: &str, entry: &str) -> PathBuf {
    let dir = desktop.join(code_sessions::INDEX_DIR).join(acct).join(org);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(entry), "{}").unwrap();
    dir
}

/// The state v0.24.8 left in an "account only" environment: its account
/// folder is a link to the existing Claude's list, and its own list is kept
/// beside it under `<org>.csw-local`. Returns (link path, existing list).
fn cs_v0248_state(provider: &MockPlatformProvider, profile: &Profile) -> (PathBuf, PathBuf) {
    let source = cs_account_dir(
        &provider.claude_desktop_default_dir(),
        CS_ACCT_A,
        CS_ORG_A,
        "local_a.json",
    );
    let own = cs_account_dir(
        &profile.isolation.desktop_user_data_dir,
        CS_ACCT_B,
        CS_ORG_B,
        "local_b.json",
    );
    let kept = own.with_file_name(format!("{CS_ORG_B}{}", code_sessions::LOCAL_SUFFIX));
    std::fs::rename(&own, &kept).unwrap();
    std::os::unix::fs::symlink(&source, &own).unwrap();
    (own, source)
}

#[test]
fn launch_restores_the_list_a_v0248_link_replaced() {
    let (provider, manager, _tmp) = setup_test_manager();
    let profile = manager
        .create_profile("research", SharingConfig::share_workspace_preset(), None)
        .unwrap();
    let (own, source) = cs_v0248_state(&provider, &profile);

    crate::switcher::desktop::launch_desktop(&profile, provider.as_ref()).unwrap();

    assert!(!provider.is_symlink(&own), "the link is gone");
    assert!(own.join("local_b.json").exists(), "the own list is back");
    assert!(source.join("local_a.json").exists());
    assert!(!source.join("local_b.json").exists());
}

#[test]
fn launch_keeps_the_code_tab_list_of_every_mode() {
    let (provider, manager, _tmp) = setup_test_manager();
    cs_account_dir(
        &provider.claude_desktop_default_dir(),
        CS_ACCT_A,
        CS_ORG_A,
        "local_a.json",
    );
    for (name, sharing) in [
        ("workspace", SharingConfig::share_workspace_preset()),
        ("settings", SharingConfig::share_settings_preset()),
        ("isolated", SharingConfig::default()),
    ] {
        let profile = manager.create_profile(name, sharing, None).unwrap();
        let own = cs_account_dir(
            &profile.isolation.desktop_user_data_dir,
            CS_ACCT_B,
            CS_ORG_B,
            "local_b.json",
        );
        crate::switcher::desktop::launch_desktop(&profile, provider.as_ref()).unwrap();
        assert!(!provider.is_symlink(&own), "{name} must keep its own list");
        assert!(own.join("local_b.json").exists());
    }
}

#[test]
fn isolation_check_reports_the_code_tab_list_after_the_link_items() {
    let (provider, manager, _tmp) = setup_test_manager();
    let profile = manager
        .create_profile("research", SharingConfig::share_workspace_preset(), None)
        .unwrap();
    cs_account_dir(
        &profile.isolation.desktop_user_data_dir,
        CS_ACCT_B,
        CS_ORG_B,
        "local_b.json",
    );
    let report = manager.inspect_profile_isolation("research").unwrap();
    assert_eq!(report.items.len(), linker::LINK_ITEMS.len() + 1);
    let last = report.items.last().unwrap();
    assert_eq!(last.key, code_sessions::KEY);
    assert_eq!(last.mode, SharingMode::Isolate);
    assert_eq!(last.health, inspector::ItemHealth::IsolatedOk);
    assert!(!last.is_issue);

    std::fs::remove_dir_all(
        profile
            .isolation
            .desktop_user_data_dir
            .join(code_sessions::INDEX_DIR),
    )
    .unwrap();
    cs_v0248_state(&provider, &profile);
    let report = manager.inspect_profile_isolation("research").unwrap();
    let last = report.items.last().unwrap();
    assert!(matches!(
        last.health,
        inspector::ItemHealth::UnexpectedLink { .. }
    ));
    assert!(last.is_issue);
}

#[test]
fn doctor_fix_restores_the_code_tab_list_unless_running() {
    let (provider, manager, _tmp) = setup_test_manager();
    let profile = manager
        .create_profile("research", SharingConfig::share_workspace_preset(), None)
        .unwrap();
    let (own, _source) = cs_v0248_state(&provider, &profile);

    // While the environment's Claude runs, its open list must not move.
    let running = Arc::new(
        MockPlatformProvider::new(
            provider.claude_desktop_default_dir(),
            provider.claude_cli_default_dir(),
            provider.app_data_dir(),
        )
        .with_running_args(vec![format!(
            "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={}",
            profile.isolation.desktop_user_data_dir.display()
        )]),
    );
    let running_manager = ProfileManager::new(running).unwrap();
    let fixed = running_manager.doctor_fix_links("research").unwrap();
    assert!(!fixed.contains(&code_sessions::KEY));
    assert!(provider.is_symlink(&own));

    let fixed = manager.doctor_fix_links("research").unwrap();
    assert!(fixed.contains(&code_sessions::KEY));
    assert!(!provider.is_symlink(&own));
    assert!(own.join("local_b.json").exists());
    let report = manager.inspect_profile_isolation("research").unwrap();
    assert_eq!(
        report.items.last().unwrap().health,
        inspector::ItemHealth::IsolatedOk
    );
}

#[test]
fn clone_of_a_v0248_environment_gets_its_own_list_back() {
    let (provider, manager, _tmp) = setup_test_manager();
    let research = manager
        .create_profile("research", SharingConfig::share_workspace_preset(), None)
        .unwrap();
    let (_own, source) = cs_v0248_state(&provider, &research);

    let copy = manager.clone_profile("research", "research2").unwrap();

    let list = copy
        .isolation
        .desktop_user_data_dir
        .join(code_sessions::INDEX_DIR)
        .join(CS_ACCT_B)
        .join(CS_ORG_B);
    assert!(!provider.is_symlink(&list));
    assert!(list.join("local_b.json").exists());
    assert!(
        !list
            .with_file_name(format!("{CS_ORG_B}{}", code_sessions::LOCAL_SUFFIX))
            .exists()
    );
    assert!(source.join("local_a.json").exists());
}

/// Every symlink under `dir`, at any depth. Links are listed, never followed.
fn cs_links_under(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut links = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return links;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            links.push(path);
        } else if meta.is_dir() {
            links.extend(cs_links_under(&path));
        }
    }
    links
}

/// The rule behind the v0.24.8 regression: the Claude Desktop App never saves
/// through a link in its data folder, so CSW must never leave one in the
/// Code-tab list, in any mode, after any operation that touches an
/// environment (create, launch, csw doctor --fix, duplicate). An environment
/// that still carries v0.24.8's link must come out of each of them without it.
#[test]
fn no_operation_leaves_a_link_in_the_code_tab_list() {
    let (provider, manager, _tmp) = setup_test_manager();
    cs_account_dir(
        &provider.claude_desktop_default_dir(),
        CS_ACCT_A,
        CS_ORG_A,
        "local_a.json",
    );
    let index = |p: &Profile| {
        p.isolation
            .desktop_user_data_dir
            .join(code_sessions::INDEX_DIR)
    };

    for (name, sharing, from_v0248) in [
        ("workspace", SharingConfig::share_workspace_preset(), false),
        (
            "workspace-v0248",
            SharingConfig::share_workspace_preset(),
            true,
        ),
        ("settings", SharingConfig::share_settings_preset(), false),
        (
            "settings-v0248",
            SharingConfig::share_settings_preset(),
            true,
        ),
        ("isolated", SharingConfig::default(), false),
    ] {
        let profile = manager.create_profile(name, sharing, None).unwrap();
        assert!(
            cs_links_under(&index(&profile)).is_empty(),
            "{name}: create"
        );

        // Signed in: the app made its own list. Optionally as v0.24.8 left it.
        if from_v0248 {
            cs_v0248_state(&provider, &profile);
        } else {
            cs_account_dir(
                &profile.isolation.desktop_user_data_dir,
                CS_ACCT_B,
                CS_ORG_B,
                "local_b.json",
            );
        }

        crate::switcher::desktop::launch_desktop(&profile, provider.as_ref()).unwrap();
        assert!(
            cs_links_under(&index(&profile)).is_empty(),
            "{name}: launch"
        );

        if from_v0248 {
            std::fs::remove_dir_all(index(&profile)).unwrap();
            cs_v0248_state(&provider, &profile);
        }
        manager.doctor_fix_links(name).unwrap();
        assert!(
            cs_links_under(&index(&profile)).is_empty(),
            "{name}: doctor --fix"
        );

        if from_v0248 {
            std::fs::remove_dir_all(index(&profile)).unwrap();
            cs_v0248_state(&provider, &profile);
        }
        let copy = manager
            .clone_profile(name, &format!("{name}-copy"))
            .unwrap();
        assert!(
            cs_links_under(&index(&copy)).is_empty(),
            "{name}: duplicate"
        );
        crate::switcher::desktop::launch_desktop(&copy, provider.as_ref()).unwrap();
        assert!(
            cs_links_under(&index(&copy)).is_empty(),
            "{name}: duplicate launch"
        );

        // Duplicating leaves the source as it was; its next launch repairs it.
        crate::switcher::desktop::launch_desktop(&profile, provider.as_ref()).unwrap();
        assert!(
            cs_links_under(&index(&profile)).is_empty(),
            "{name}: relaunch"
        );

        // The environment's own list is a real folder holding its own entry.
        let own = index(&profile).join(CS_ACCT_B).join(CS_ORG_B);
        assert!(std::fs::symlink_metadata(&own).unwrap().is_dir(), "{name}");
        assert!(own.join("local_b.json").exists(), "{name}");
    }

    // The existing Claude's list was never written to.
    let source = provider
        .claude_desktop_default_dir()
        .join(code_sessions::INDEX_DIR);
    assert!(cs_links_under(&source).is_empty());
    let entries: Vec<_> = std::fs::read_dir(source.join(CS_ACCT_A).join(CS_ORG_A))
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("local_a.json")]);
}

#[test]
fn clone_skips_nested_symlinks_instead_of_failing() {
    // SPECIFICATION.md §5.A「環境の複製」: links are never copied nor followed,
    // at any depth, whether they point at a file or at a folder. Before this
    // rule a folder link made the whole duplication fail in fs::copy.
    let (_provider, manager, tmp) = setup_test_manager();
    let profile = manager
        .create_profile("isolated", SharingConfig::default(), None)
        .unwrap();
    let rules = profile.isolation.cli_config_dir.join("rules");
    std::fs::create_dir_all(&rules).unwrap();
    std::fs::write(rules.join("own.md"), "own").unwrap();
    let outside = tmp.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("shared.md"), "outside").unwrap();
    std::os::unix::fs::symlink(outside.join("shared.md"), rules.join("linked.md")).unwrap();
    std::os::unix::fs::symlink(&outside, rules.join("linked-dir")).unwrap();

    let copy = manager.clone_profile("isolated", "isolated2").unwrap();

    let copied = copy.isolation.cli_config_dir.join("rules");
    assert_eq!(
        std::fs::read_to_string(copied.join("own.md")).unwrap(),
        "own"
    );
    assert!(
        std::fs::symlink_metadata(copied.join("linked.md")).is_err(),
        "a link to a file is not copied"
    );
    assert!(
        std::fs::symlink_metadata(copied.join("linked-dir")).is_err(),
        "a link to a folder is not copied"
    );
    assert_eq!(
        std::fs::read_to_string(outside.join("shared.md")).unwrap(),
        "outside",
        "the link target is untouched"
    );
}

// --- SSH connections (SPECIFICATION.md §3, issue #239) ---

/// The Desktop app's SSH files in `desktop`: the two that describe the
/// connections, plus the remembered passwords and the per-host tokens.
fn ssh_files(desktop: &std::path::Path) {
    std::fs::write(
        desktop.join("ssh_configs.json"),
        r#"{"configs":[{"id":"c1","name":"Lab","sshHost":"me@lab"}],"trustedHosts":["me@lab"]}"#,
    )
    .unwrap();
    std::fs::write(
        desktop.join("ssh_worktree_locations.json"),
        r#"{"ssh:me@lab:22":"/home/me"}"#,
    )
    .unwrap();
    std::fs::write(desktop.join("ssh-remembered-passwords.json"), "secret").unwrap();
    std::fs::write(desktop.join("ssh-remote-server-state.json"), "tokens").unwrap();
}

#[test]
fn ssh_connections_are_copied_by_the_setup_modes_and_never_their_secrets() {
    let (provider, manager, _tmp) = setup_test_manager();
    let default_desktop = provider.claude_desktop_default_dir();
    ssh_files(&default_desktop);

    for (name, sharing, copied) in [
        ("workspace", SharingConfig::share_workspace_preset(), true),
        ("settings", SharingConfig::share_settings_preset(), true),
        ("isolated", SharingConfig::default(), false),
    ] {
        let profile = manager.create_profile(name, sharing, None).unwrap();
        let desktop = &profile.isolation.desktop_user_data_dir;
        for file in ["ssh_configs.json", "ssh_worktree_locations.json"] {
            let path = desktop.join(file);
            if copied {
                assert!(!provider.is_symlink(&path), "{name}: {file} is a copy");
                assert_eq!(
                    std::fs::read(&path).unwrap(),
                    std::fs::read(default_desktop.join(file)).unwrap(),
                    "{name}: {file}"
                );
            } else {
                assert!(std::fs::symlink_metadata(&path).is_err(), "{name}: {file}");
            }
        }
        for secret in [
            "ssh-remembered-passwords.json",
            "ssh-remote-server-state.json",
        ] {
            assert!(
                std::fs::symlink_metadata(desktop.join(secret)).is_err(),
                "{name}: {secret} must never be carried over"
            );
        }
        let report = manager.inspect_profile_isolation(name).unwrap();
        assert_eq!(report.issue_count, 0, "{name}: {:?}", report.items);
    }

    // Copied once: a later change on either side stays on that side.
    std::fs::write(default_desktop.join("ssh_configs.json"), "{}").unwrap();
    let settings = manager.get_profile("settings").unwrap();
    assert_ne!(
        std::fs::read_to_string(
            settings
                .isolation
                .desktop_user_data_dir
                .join("ssh_configs.json")
        )
        .unwrap(),
        "{}"
    );
}

#[test]
fn ssh_connections_make_an_environment_not_fully_isolated() {
    let mut sharing = SharingConfig::default();
    assert!(sharing.is_fully_isolated());
    sharing.desktop_ssh_connections = SharingMode::Copy;
    assert!(!sharing.is_fully_isolated());
}

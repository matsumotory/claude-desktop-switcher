use csw_core::platform::PlatformProvider;
use csw_core::platform::mock::MockPlatformProvider;
use csw_core::profile::{ProfileManager, SharingConfig, SharingMode, SharingSource};
use csw_core::switcher::ContextSwitcher;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

fn setup_dummy_claude_data(desktop_path: &Path, cli_path: &Path) {
    fs::create_dir_all(desktop_path).unwrap();
    fs::create_dir_all(cli_path).unwrap();

    // 1. Desktop (GUI) Dummy Configuration Files
    let config_path = desktop_path.join("claude_desktop_config.json");
    fs::write(
        &config_path,
        "{\"mcpServers\":{\"dummy\":{\"command\":\"dummy-cmd\"}},\"theme\":\"dark\"}",
    )
    .unwrap();

    let app_config_path = desktop_path.join("config.json");
    fs::write(&app_config_path, "{\"device_id\":\"dummy-device-id-1234\"}").unwrap();

    let worktree_path = desktop_path.join("worktrees");
    fs::create_dir_all(&worktree_path).unwrap();
    fs::write(worktree_path.join("active_worktree.json"), "{}").unwrap();

    // 2. CLI (Claude Code) Dummy Configuration Files
    let settings_path = cli_path.join("settings.json");
    fs::write(&settings_path, "{\"primary_identity\":\"personal\"}").unwrap();

    let claude_md_path = cli_path.join("CLAUDE.md");
    fs::write(&claude_md_path, "# CLAUDE.md\nRules for AI assistant").unwrap();

    let project_mem_path = cli_path.join("MEMORY.md");
    fs::write(&project_mem_path, "# MEMORY.md\nProject memory context").unwrap();

    let plugins_path = cli_path.join("plugins");
    fs::create_dir_all(&plugins_path).unwrap();
    fs::write(
        plugins_path.join("dummy_plugin.js"),
        "console.log('dummy');",
    )
    .unwrap();

    let skills_path = cli_path.join("skills");
    fs::create_dir_all(&skills_path).unwrap();
    fs::write(
        skills_path.join("git_helper.json"),
        "{\"name\":\"git master assistant\",\"description\":\"helper for git commands\"}",
    )
    .unwrap();

    let sessions_path = cli_path.join("sessions");
    fs::create_dir_all(&sessions_path).unwrap();
    fs::write(
        sessions_path.join("session_99.json"),
        "{\"session_id\":\"99\",\"chat_history\":[]}",
    )
    .unwrap();

    let history_path = cli_path.join("history.jsonl");
    fs::write(
        &history_path,
        "{\"timestamp\":\"2026-06-27T00:00:00Z\",\"command\":\"claude test\"}\n",
    )
    .unwrap();
}

// Account/session isolation is achieved purely via per-profile directories
// (--user-data-dir / CLAUDE_CONFIG_DIR), so switching only updates the active
// profile. CSW does not touch the Keychain, hence no keychain assertions here.
#[test]
fn test_full_profile_switch_workflow() {
    let desktop_dir = tempdir().unwrap();
    let cli_dir = tempdir().unwrap();
    let app_dir = tempdir().unwrap();

    let platform = Arc::new(MockPlatformProvider::new(
        desktop_dir.path().to_path_buf(),
        cli_dir.path().to_path_buf(),
        app_dir.path().to_path_buf(),
    ));

    let profile_manager = Arc::new(ProfileManager::new(platform.clone()).unwrap());
    let switcher = ContextSwitcher::new(platform.clone(), profile_manager.clone());

    // Verify default active profile
    assert_eq!(profile_manager.active_profile_name(), "default");

    // Create a new profile (default sharing config = Isolated)
    profile_manager
        .create_profile("Work", SharingConfig::default(), None)
        .unwrap();

    // Switch to the new profile
    switcher.switch_to("Work").unwrap();
    assert_eq!(profile_manager.active_profile_name(), "Work");

    // Switch back to default
    switcher.switch_to("default").unwrap();
    assert_eq!(profile_manager.active_profile_name(), "default");

    // Switching to a non-existent profile must fail and leave the active
    // profile unchanged.
    assert!(switcher.switch_to("DoesNotExist").is_err());
    assert_eq!(profile_manager.active_profile_name(), "default");
}

#[test]
fn test_profile_sharing_and_isolation_matrix() {
    let desktop_dir = tempdir().unwrap();
    let cli_dir = tempdir().unwrap();
    let app_dir = tempdir().unwrap();

    // Setup original files
    setup_dummy_claude_data(desktop_dir.path(), cli_dir.path());

    let platform = Arc::new(MockPlatformProvider::new(
        desktop_dir.path().to_path_buf(),
        cli_dir.path().to_path_buf(),
        app_dir.path().to_path_buf(),
    ));

    let profile_manager = ProfileManager::new(platform.clone()).unwrap();

    // Configure a matrix of Share, Copy, and Isolate components
    let matrix_sharing = SharingConfig {
        desktop_config: SharingMode::Share, // MCP server config -> Shared (symlink)
        desktop_app_config: SharingMode::Copy, // App preferences -> Copied
        cli_settings: SharingMode::Isolate, // CLI settings -> Isolated (none initially)
        cli_claude_md: SharingMode::Share,  // CLAUDE.md -> Shared (symlink)
        cli_project_memory: SharingMode::Share, // Project memory -> Shared (symlink)
        cli_plugins: SharingMode::Isolate,  // Plugins -> Isolated (empty dir)
        cli_skills: SharingMode::Copy,      // Skills -> Copied (physical copy)
        cli_sessions: SharingMode::Isolate, // Sessions -> Isolated (empty dir)
        cli_history: SharingMode::Isolate,  // History -> Isolated (none)
        desktop_worktrees: SharingMode::Copy, // Worktrees -> Copied
        desktop_device_id: SharingMode::Share, // Device ID -> Shared
        source: SharingSource::default(),
    };

    let profile = profile_manager
        .create_profile("CustomMatrix", matrix_sharing, None)
        .unwrap();
    let target_desktop = &profile.isolation.desktop_user_data_dir;
    let target_cli = &profile.isolation.cli_config_dir;

    // --- ASSERTIONS FOR SHARE MODE ---
    let target_desktop_config = target_desktop.join("claude_desktop_config.json");
    assert!(target_desktop_config.exists());
    assert!(platform.is_symlink(&target_desktop_config));

    let target_claude_md = target_cli.join("CLAUDE.md");
    assert!(target_claude_md.exists());
    assert!(platform.is_symlink(&target_claude_md));

    // --- ASSERTIONS FOR COPY MODE ---
    let target_config_json = target_desktop.join("config.json");
    assert!(target_config_json.exists());
    assert!(!platform.is_symlink(&target_config_json));
    let orig_content = fs::read_to_string(desktop_dir.path().join("config.json")).unwrap();
    let copied_content = fs::read_to_string(&target_config_json).unwrap();
    assert_eq!(orig_content, copied_content);

    let target_skills_dir = target_cli.join("skills");
    assert!(target_skills_dir.is_dir());
    assert!(!platform.is_symlink(&target_skills_dir));
    let target_skill_file = target_skills_dir.join("git_helper.json");
    assert!(target_skill_file.exists());
    assert!(
        fs::read_to_string(target_skill_file)
            .unwrap()
            .contains("git master assistant")
    );

    // --- ASSERTIONS FOR ISOLATE MODE ---
    let target_settings = target_cli.join("settings.json");
    assert!(!target_settings.exists());

    let target_sessions_dir = target_cli.join("sessions");
    assert!(target_sessions_dir.is_dir());
    assert!(!platform.is_symlink(&target_sessions_dir));
    let entries = fs::read_dir(target_sessions_dir).unwrap().count();
    assert_eq!(entries, 0, "Isolated sessions directory must be empty");

    let target_history = target_cli.join("history.jsonl");
    assert!(!target_history.exists());
}

// Cloning duplicates a profile's sharing config and physical data. Credentials
// are never stored by CSW (each profile logs in within its own directory), so
// cloning does not duplicate any keychain backup file.
#[test]
fn test_profile_cloning_with_data() {
    let desktop_dir = tempdir().unwrap();
    let cli_dir = tempdir().unwrap();
    let app_dir = tempdir().unwrap();

    // Setup default mock structure
    setup_dummy_claude_data(desktop_dir.path(), cli_dir.path());

    let platform = Arc::new(MockPlatformProvider::new(
        desktop_dir.path().to_path_buf(),
        cli_dir.path().to_path_buf(),
        app_dir.path().to_path_buf(),
    ));

    let profile_manager = Arc::new(ProfileManager::new(platform.clone()).unwrap());

    // Create OriginalWork that copies skills from the default environment.
    let sharing = SharingConfig {
        cli_skills: SharingMode::Copy,
        cli_sessions: SharingMode::Isolate,
        ..Default::default()
    };
    profile_manager
        .create_profile("OriginalWork", sharing, None)
        .unwrap();

    // Clone OriginalWork to ClonedWork.
    let cloned_profile = profile_manager
        .clone_profile("OriginalWork", "ClonedWork")
        .unwrap();

    // --- VERIFY SHARING CONFIG INHERITED ---
    assert_eq!(cloned_profile.sharing.cli_skills, SharingMode::Copy);
    assert_eq!(cloned_profile.sharing.cli_sessions, SharingMode::Isolate);

    // --- VERIFY PHYSICAL DATA DUPLICATION ---
    // Copy-mode skills should be physically present in the clone.
    let cloned_skill = cloned_profile
        .isolation
        .cli_config_dir
        .join("skills")
        .join("git_helper.json");
    assert!(
        cloned_skill.exists(),
        "Copy-mode skills must be duplicated into the cloned profile"
    );

    // No credential/keychain backup file should ever be produced.
    let leaked_backup = cloned_profile
        .isolation
        .cli_config_dir
        .join("keychain_backup.json");
    assert!(
        !leaked_backup.exists(),
        "CSW must not write any keychain backup file"
    );
}

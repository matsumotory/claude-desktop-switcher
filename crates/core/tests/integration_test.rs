use csw_core::keychain::{create_keychain_provider, KeychainProvider};
use csw_core::platform::mock::MockPlatformProvider;
use csw_core::profile::{ProfileManager, SharingConfig, SharingMode, SharingSource};
use csw_core::switcher::ContextSwitcher;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;

// Helper to construct dummy Claude Desktop/CLI data structure
// (Strictly artificial/dummy content for privacy protection)
fn setup_dummy_claude_data(desktop_root: &Path, cli_root: &Path) {
    // 1. Setup Desktop Default Data
    let desktop_config_path = desktop_root.join("claude_desktop_config.json");
    fs::write(
        &desktop_config_path,
        r#"{"mcpServers":{"dummy-sqlite":{"command":"uv","args":["run","mcp-sqlite"]}}}"#,
    ).unwrap();

    let app_config_path = desktop_root.join("config.json");
    fs::write(
        &app_config_path,
        r#"{"theme":"dark","fontSize":13,"sendTelemetry":false}"#,
    ).unwrap();

    let worktrees_path = desktop_root.join("git-worktrees.json");
    fs::write(
        &worktrees_path,
        r#"{"active_worktrees":["/users/dummy/worktree-1"]}"#,
    ).unwrap();

    let ant_did_path = desktop_root.join("ant-did");
    fs::write(&ant_did_path, "ant-did-dummy-device-token-abc123xyz").unwrap();

    // 2. Setup CLI Default Data
    let settings_path = cli_root.join("settings.json");
    fs::write(
        &settings_path,
        r#"{"autoApprove":false,"theme":"light","editor":"vim"}"#,
    ).unwrap();

    let claude_md_path = cli_root.join("CLAUDE.md");
    fs::write(
        &claude_md_path,
        "# Claude Personal Rules\n- Maintain clean code\n- Always add tests\n",
    ).unwrap();

    let projects_dir = cli_root.join("projects");
    fs::create_dir_all(&projects_dir).unwrap();
    let proj_a_dir = projects_dir.join("project-alpha");
    fs::create_dir_all(&proj_a_dir).unwrap();
    fs::write(
        proj_a_dir.join("MEMORY.md"),
        "# Memory Index\n- Language: Rust\n- Framework: Tauri\n",
    ).unwrap();

    let skills_dir = cli_root.join("skills");
    fs::create_dir_all(&skills_dir).unwrap();
    fs::write(
        skills_dir.join("git_helper.json"),
        r#"{"name":"git_helper","prompt":"You are a git master assistant."}"#,
    ).unwrap();

    let sessions_dir = cli_root.join("sessions");
    fs::create_dir_all(&sessions_dir).unwrap();
    fs::write(
        sessions_dir.join("session_1001.json"),
        r#"{"id":"session_1001","messages":[{"role":"user","content":"Hello"}]}"#,
    ).unwrap();

    let history_path = cli_root.join("history.jsonl");
    fs::write(
        &history_path,
        "{\"timestamp\":\"2026-06-27T00:00:00Z\",\"command\":\"claude test\"}\n",
    ).unwrap();
}

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

    let keychain = create_keychain_provider();
    keychain.set_password("Claude Safe Storage", "Claude Key", "default-secret-token").unwrap();

    assert_eq!(profile_manager.active_profile_name(), "default");

    let sharing_config = SharingConfig::default(); // default is Isolated
    profile_manager.create_profile("Work", sharing_config).unwrap();

    switcher.switch_to("Work").unwrap();

    assert_eq!(profile_manager.active_profile_name(), "Work");

    let pwd = keychain.get_password("Claude Safe Storage", "Claude Key").unwrap();
    assert_eq!(pwd, None, "Keychain should be empty for the new profile");

    keychain.set_password("Claude Safe Storage", "Claude Key", "work-secret-token").unwrap();

    switcher.switch_to("default").unwrap();

    assert_eq!(profile_manager.active_profile_name(), "default");

    let restored_pwd = keychain.get_password("Claude Safe Storage", "Claude Key").unwrap();
    assert_eq!(restored_pwd.unwrap(), "default-secret-token");
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
        desktop_config: SharingMode::Share,       // MCP server config -> Shared (symlink)
        desktop_app_config: SharingMode::Copy,    // App preferences -> Copied
        cli_settings: SharingMode::Isolate,       // CLI settings -> Isolated (none initially)
        cli_claude_md: SharingMode::Share,        // CLAUDE.md -> Shared (symlink)
        cli_project_memory: SharingMode::Share,   // Project memory -> Shared (symlink)
        cli_plugins: SharingMode::Isolate,        // Plugins -> Isolated (empty dir)
        cli_skills: SharingMode::Copy,            // Skills -> Copied (physical copy)
        cli_sessions: SharingMode::Isolate,       // Sessions -> Isolated (empty dir)
        cli_history: SharingMode::Isolate,        // History -> Isolated (none)
        desktop_worktrees: SharingMode::Copy,     // Worktrees -> Copied
        desktop_device_id: SharingMode::Share,     // Device ID -> Shared
        source: SharingSource::default(),
    };

    let profile = profile_manager.create_profile("CustomMatrix", matrix_sharing).unwrap();
    let target_desktop = &profile.isolation.desktop_user_data_dir;
    let target_cli = &profile.isolation.cli_config_dir;

    // --- ASSERTIONS FOR SHARE MODE ---
    // 1. claude_desktop_config.json should be a symlink
    let target_desktop_config = target_desktop.join("claude_desktop_config.json");
    assert!(target_desktop_config.exists());
    assert!(platform.is_symlink(&target_desktop_config));

    // 2. CLAUDE.md should be a symlink
    let target_claude_md = target_cli.join("CLAUDE.md");
    assert!(target_claude_md.exists());
    assert!(platform.is_symlink(&target_claude_md));

    // --- ASSERTIONS FOR COPY MODE ---
    // 3. config.json should be a physical file, NOT a symlink, but with same content
    let target_config_json = target_desktop.join("config.json");
    assert!(target_config_json.exists());
    assert!(!platform.is_symlink(&target_config_json));
    let orig_content = fs::read_to_string(desktop_dir.path().join("config.json")).unwrap();
    let copied_content = fs::read_to_string(&target_config_json).unwrap();
    assert_eq!(orig_content, copied_content);

    // 4. cli_skills (skills/ folder) should be a physical directory copy
    let target_skills_dir = target_cli.join("skills");
    assert!(target_skills_dir.is_dir());
    assert!(!platform.is_symlink(&target_skills_dir));
    let target_skill_file = target_skills_dir.join("git_helper.json");
    assert!(target_skill_file.exists());
    assert!(fs::read_to_string(target_skill_file).unwrap().contains("git master assistant"));

    // --- ASSERTIONS FOR ISOLATE MODE ---
    // 5. cli_settings (settings.json) should NOT exist initially
    let target_settings = target_cli.join("settings.json");
    assert!(!target_settings.exists());

    // 6. cli_sessions (sessions/ folder) should be an empty directory
    let target_sessions_dir = target_cli.join("sessions");
    assert!(target_sessions_dir.is_dir());
    assert!(!platform.is_symlink(&target_sessions_dir));
    let entries = fs::read_dir(target_sessions_dir).unwrap().count();
    assert_eq!(entries, 0, "Isolated sessions directory must be empty");

    // 7. cli_history (history.jsonl) should NOT exist
    let target_history = target_cli.join("history.jsonl");
    assert!(!target_history.exists());
}

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
    let switcher = ContextSwitcher::new(platform.clone(), profile_manager.clone());

    // Switch/create to set credentials and create a credential backup
    let keychain = create_keychain_provider();
    keychain.set_password("Claude Code-credentials", "CloudFlare", "original-flare-token").unwrap();

    // Create Work profile
    let mut sharing = SharingConfig::default();
    sharing.cli_skills = SharingMode::Copy;
    sharing.cli_sessions = SharingMode::Isolate;
    let original_profile = profile_manager.create_profile("OriginalWork", sharing).unwrap();

    // Switch to generate credential backup inside OriginalWork
    switcher.switch_to("OriginalWork").unwrap();
    // Keychain for OriginalWork set
    keychain.set_password("Claude Code-credentials", "CloudFlare", "work-flare-token").unwrap();
    // Switch away to trigger backup serialization
    switcher.switch_to("default").unwrap();

    // Now, clone OriginalWork to cloned_work
    let cloned_profile = profile_manager.clone_profile("OriginalWork", "ClonedWork").unwrap();

    // --- VERIFY DATA DUPLICATION ---
    let target_cli = &cloned_profile.isolation.cli_config_dir;
    let target_backup_keychain = target_cli.join("keychain_backup.json");
    
    // Verify keychain backup was cloned
    assert!(target_backup_keychain.exists(), "Keychain backup file must be duplicated");
    let backup_content = fs::read_to_string(target_backup_keychain).unwrap();
    assert!(backup_content.contains("work-flare-token"));

    // Verify sharing configuration was inherited
    assert_eq!(cloned_profile.sharing.cli_skills, SharingMode::Copy);
    assert_eq!(cloned_profile.sharing.cli_sessions, SharingMode::Isolate);

    // Switch to ClonedWork and verify that credentials were correctly restored
    switcher.switch_to("ClonedWork").unwrap();
    let restored_cred = keychain.get_password("Claude Code-credentials", "CloudFlare").unwrap();
    assert_eq!(restored_cred.unwrap(), "work-flare-token", "Cloned credentials must be restored upon switching");
}


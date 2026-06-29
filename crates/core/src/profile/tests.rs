use super::*;
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

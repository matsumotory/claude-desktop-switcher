use csw_core::keychain::{create_keychain_provider, KeychainProvider};
use csw_core::platform::mock::MockPlatformProvider;
use csw_core::profile::{ProfileManager, SharingConfig};
use csw_core::switcher::ContextSwitcher;
use std::sync::Arc;
use tempfile::tempdir;

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

    // 1. Initialize Managers
    let profile_manager = Arc::new(ProfileManager::new(platform.clone()).unwrap());
    let switcher = ContextSwitcher::new(platform.clone(), profile_manager.clone());

    // 2. Set default credentials
    let keychain = create_keychain_provider();
    keychain.set_password("Claude Safe Storage", "Claude Key", "default-secret-token").unwrap();

    // Verify default active profile
    assert_eq!(profile_manager.active_profile_name(), "default");

    // 3. Create a new profile
    let sharing_config = SharingConfig::default(); // default is Isolated
    profile_manager.create_profile("Work", sharing_config).unwrap();

    // 4. Switch to the new profile
    switcher.switch_to("Work").unwrap();

    // 5. Verify the active profile changed
    assert_eq!(profile_manager.active_profile_name(), "Work");

    // 6. Verify that the keychain was cleared for the new isolated profile
    let pwd = keychain.get_password("Claude Safe Storage", "Claude Key").unwrap();
    assert_eq!(pwd, None, "Keychain should be empty for the new profile");

    // Set a password for the Work profile
    keychain.set_password("Claude Safe Storage", "Claude Key", "work-secret-token").unwrap();

    // 7. Switch back to default
    switcher.switch_to("default").unwrap();

    // 8. Verify the active profile changed back
    assert_eq!(profile_manager.active_profile_name(), "default");

    // 9. Verify the original default credentials were restored
    let restored_pwd = keychain.get_password("Claude Safe Storage", "Claude Key").unwrap();
    assert_eq!(restored_pwd.unwrap(), "default-secret-token");
}

use std::sync::Arc;
use tempfile::tempdir;
use crate::platform::mock::MockPlatformProvider;
use super::*;

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
    
    let default_profile = manager.get_profile("default").expect("Failed to get default profile");
    assert_eq!(default_profile.profile.name, "default");
    assert!(default_profile.profile.is_default);
    
    assert_eq!(manager.active_profile_name(), "default");
}

#[test]
fn test_create_profile() {
    let (_, manager, _tmp_dir) = setup_test_manager();
    
    let profile = manager.create_profile("test_profile", SharingConfig::default(), None)
        .expect("Failed to create profile");
        
    assert_eq!(profile.profile.name, "test_profile");
    assert_eq!(profile.profile.is_default, false);
    
    // Verify it shows up in list
    let list = manager.list_profiles().unwrap();
    assert!(list.contains(&"test_profile".to_string()));
    assert!(list.contains(&"default".to_string()));
}

#[test]
fn test_delete_profile() {
    let (_, manager, _tmp_dir) = setup_test_manager();
    
    manager.create_profile("to_delete", SharingConfig::default(), None)
        .expect("Failed to create profile");
        
    let list = manager.list_profiles().unwrap();
    assert!(list.contains(&"to_delete".to_string()));
    
    manager.delete_profile("to_delete").expect("Failed to delete profile");
    
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
    manager.create_profile("active_prof", SharingConfig::default(), None).unwrap();
    manager.switch_to("active_prof").unwrap();
    
    let res2 = manager.delete_profile("active_prof");
    assert!(res2.is_err());
}

#[test]
fn test_clone_profile() {
    let (_, manager, _tmp_dir) = setup_test_manager();
    
    // Create an original profile to clone from
    let original = manager.create_profile("original", SharingConfig::default(), None)
        .expect("Failed to create original profile");

    // Clone it
    let cloned = manager.clone_profile("original", "cloned")
        .expect("Failed to clone profile");

    assert_eq!(cloned.profile.name, "cloned");
    assert_eq!(cloned.profile.icon, original.profile.icon);
    assert_eq!(cloned.profile.color, original.profile.color);
    assert_eq!(cloned.profile.is_default, false);

    // Verify it exists in profiles list
    let list = manager.list_profiles().unwrap();
    assert!(list.contains(&"original".to_string()));
    assert!(list.contains(&"cloned".to_string()));
}


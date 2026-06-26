pub mod desktop;
pub mod shell;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::Result;
use crate::keychain::{create_keychain_provider, KeychainCredential, KeychainProvider};
use crate::platform::PlatformProvider;
use crate::profile::{Profile, ProfileManager};

pub struct ContextSwitcher {
    _provider: Arc<dyn PlatformProvider>,
    profile_manager: Arc<ProfileManager>,
    keychain: Box<dyn KeychainProvider>,
}

impl ContextSwitcher {
    pub fn new(
        provider: Arc<dyn PlatformProvider>,
        profile_manager: Arc<ProfileManager>,
    ) -> Self {
        Self {
            _provider: provider,
            profile_manager,
            keychain: create_keychain_provider(),
        }
    }

    /// Backup the currently active profile's credentials from Keychain to files.
    pub fn backup_active_credentials(&self) -> Result<()> {
        let active_profile = self.profile_manager.active_profile()?;
        self.backup_credentials_for(&active_profile)
    }

    /// Restore the target profile's credentials to Keychain.
    pub fn restore_credentials_for(&self, profile: &Profile) -> Result<()> {
        let backup_path = self.credentials_backup_path(profile);

        // 1. Desktop keychain: "Claude Safe Storage" / "Claude Key"
        // 2. CLI keychain: "Claude Code-credentials" / "CloudFlare" (or user specific account)
        if backup_path.exists() {
            let content = fs::read_to_string(&backup_path)?;
            let credentials: Vec<KeychainCredential> = serde_json::from_str(&content)?;

            // Apply all credentials in backup
            for cred in credentials {
                self.keychain.set_password(&cred.service, &cred.account, &cred.password)?;
            }
        } else {
            // If no backup exists, we clear the keychain entries so the user starts clean (or logs in)
            // instead of inheriting the previous profile's keys.
            self.keychain.delete_password("Claude Safe Storage", "Claude Key")?;
            // Note: CloudFlare account name might vary, but default is "CloudFlare"
            self.keychain.delete_password("Claude Code-credentials", "CloudFlare")?;
        }

        Ok(())
    }

    /// Perform a full context switch to the target profile.
    pub fn switch_to(&self, profile_name: &str) -> Result<()> {
        let target_profile = self.profile_manager.get_profile(profile_name)?;
        
        // 1. Backup current credentials
        self.backup_active_credentials()?;

        // 2. Clear current Keychain entries (to prevent bleed)
        self.keychain.delete_password("Claude Safe Storage", "Claude Key")?;
        self.keychain.delete_password("Claude Code-credentials", "CloudFlare")?;

        // 3. Restore target credentials
        self.restore_credentials_for(&target_profile)?;

        // 4. Update the active profile name in configuration
        self.profile_manager.switch_to(profile_name)?;

        Ok(())
    }

    fn backup_credentials_for(&self, profile: &Profile) -> Result<()> {
        let mut credentials = Vec::new();

        // Backup Desktop key
        if let Some(password) = self.keychain.get_password("Claude Safe Storage", "Claude Key")? {
            credentials.push(KeychainCredential {
                service: "Claude Safe Storage".to_string(),
                account: "Claude Key".to_string(),
                password,
            });
        }

        // Backup CLI credentials
        if let Some(password) = self.keychain.get_password("Claude Code-credentials", "CloudFlare")? {
            credentials.push(KeychainCredential {
                service: "Claude Code-credentials".to_string(),
                account: "CloudFlare".to_string(),
                password,
            });
        }

        if !credentials.is_empty() {
            let backup_path = self.credentials_backup_path(profile);
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(&credentials)?;
            fs::write(backup_path, json)?;
        }

        Ok(())
    }

    fn credentials_backup_path(&self, profile: &Profile) -> PathBuf {
        // We store it inside the isolated cli_config_dir to keep it clean.
        profile.isolation.cli_config_dir.join("keychain_backup.json")
    }
}

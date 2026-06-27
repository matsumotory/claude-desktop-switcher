pub mod desktop;
pub mod shell;

use std::sync::Arc;

use crate::error::{CswError, Result};
use crate::platform::PlatformProvider;
use crate::profile::ProfileManager;

/// Coordinates switching the active profile.
///
/// Account/session isolation is achieved purely by per-profile directories:
/// the Desktop app is launched with `--user-data-dir` and the CLI runs with
/// `CLAUDE_CONFIG_DIR` pointing at the profile's own directories. Each profile
/// keeps its own login inside its own directory, so CSW never reads, writes, or
/// deletes Keychain credentials.
///
/// This was verified empirically: a fresh `CLAUDE_CONFIG_DIR` reports
/// `loggedIn: false` independently of the default profile, i.e. Claude Code
/// scopes its credentials per config directory. Likewise the Desktop app keeps
/// its session inside its `--user-data-dir`. Copying secrets out of the OS
/// Keychain into files would only weaken security without being necessary, so
/// that mechanism has been removed.
pub struct ContextSwitcher {
    provider: Arc<dyn PlatformProvider>,
    profile_manager: Arc<ProfileManager>,
}

impl ContextSwitcher {
    pub fn new(provider: Arc<dyn PlatformProvider>, profile_manager: Arc<ProfileManager>) -> Self {
        Self {
            provider,
            profile_manager,
        }
    }

    /// Switch the active profile.
    ///
    /// Isolation is handled at launch time via per-profile directories
    /// (`--user-data-dir` / `CLAUDE_CONFIG_DIR`), so this only validates the
    /// target exists and records it as the active profile. No Keychain or
    /// credential files are touched.
    ///
    /// Switching is refused while Claude Desktop is running: a live instance can
    /// write cached state back into the currently active profile's directory and
    /// race with shared (symlinked) files during the switch, so the user must
    /// quit Claude Desktop first.
    pub fn switch_to(&self, profile_name: &str) -> Result<()> {
        // Validate the profile exists before recording it as active.
        let _ = self.profile_manager.get_profile(profile_name)?;

        // Refuse to switch while Claude Desktop is running to avoid cache
        // write-back and symlink data races.
        if self.provider.is_claude_desktop_running()? {
            return Err(CswError::DesktopRunning);
        }

        self.profile_manager.switch_to(profile_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::mock::MockPlatformProvider;
    use crate::profile::SharingConfig;
    use tempfile::tempdir;

    /// Switching while Claude Desktop is running must be refused so a live
    /// instance cannot write cached state back / race with shared files.
    #[test]
    fn switch_to_refuses_while_desktop_running() {
        let desktop_dir = tempdir().unwrap();
        let cli_dir = tempdir().unwrap();
        let app_dir = tempdir().unwrap();

        let provider = Arc::new(
            MockPlatformProvider::new(
                desktop_dir.path().to_path_buf(),
                cli_dir.path().to_path_buf(),
                app_dir.path().to_path_buf(),
            )
            .with_desktop_running(true),
        );
        let pm = Arc::new(ProfileManager::new(provider.clone()).unwrap());
        pm.create_profile("Work", SharingConfig::default(), None)
            .unwrap();

        let switcher = ContextSwitcher::new(provider, pm.clone());
        let err = switcher.switch_to("Work").unwrap_err();
        assert!(matches!(err, CswError::DesktopRunning));
        // Active profile must be unchanged after a refused switch.
        assert_eq!(pm.active_profile_name(), "default");
    }

    #[test]
    fn switch_to_succeeds_when_desktop_not_running() {
        let desktop_dir = tempdir().unwrap();
        let cli_dir = tempdir().unwrap();
        let app_dir = tempdir().unwrap();

        let provider = Arc::new(MockPlatformProvider::new(
            desktop_dir.path().to_path_buf(),
            cli_dir.path().to_path_buf(),
            app_dir.path().to_path_buf(),
        ));
        let pm = Arc::new(ProfileManager::new(provider.clone()).unwrap());
        pm.create_profile("Work", SharingConfig::default(), None)
            .unwrap();

        let switcher = ContextSwitcher::new(provider, pm.clone());
        switcher.switch_to("Work").unwrap();
        assert_eq!(pm.active_profile_name(), "Work");
    }
}

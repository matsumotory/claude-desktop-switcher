pub mod desktop;
pub mod shell;

use std::path::Path;
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

/// Whether a Claude Desktop instance for `target_dir` is currently running.
///
/// "In use" (利用中) is a live runtime state. Because fully-isolated environments
/// can now run side by side, it can no longer be derived from "the active
/// environment while Claude runs" (that would mislabel which environment is up).
/// Instead it is read from the live processes: `running_args` are the argument
/// strings of the running Claude *main* processes (helpers are filtered out by the
/// platform layer). An environment is in use when a main process was launched with
/// its `--user-data-dir`. A main process launched without the flag (a plain
/// Finder/Dock launch) uses the default data dir, so it counts as the existing
/// Claude (`default`).
pub fn desktop_dir_running(target_dir: &Path, running_args: &[String], default_dir: &Path) -> bool {
    let needle = format!("--user-data-dir={}", target_dir.display());
    if running_args.iter().any(|a| flag_value_present(a, &needle)) {
        return true;
    }
    // No explicit --user-data-dir means the default location.
    target_dir == default_dir && running_args.iter().any(|a| !a.contains("--user-data-dir="))
}

/// True if `args` contains `needle` as a whole token: followed by a space or the
/// end of the string. This keeps a value like `.../Claude` from matching a longer
/// sibling `.../Claude2`, while still allowing the value itself to contain spaces
/// (the default dir lives under "Application Support").
fn flag_value_present(args: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(rel) = args[from..].find(needle) {
        let end = from + rel + needle.len();
        if end == args.len() || args.as_bytes()[end] == b' ' {
            return true;
        }
        from = end;
    }
    false
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

    use std::path::Path;

    /// An environment is running when a live Claude main process was launched with
    /// its `--user-data-dir`.
    #[test]
    fn desktop_dir_running_matches_user_data_dir_arg() {
        let default_dir = Path::new("/Users/x/Library/Application Support/Claude");
        let work = Path::new("/Users/x/.context-switcher-claude/profiles/Work/desktop-data");
        let args = vec![format!(
            "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={}",
            work.display()
        )];
        assert!(desktop_dir_running(work, &args, default_dir));
        assert!(!desktop_dir_running(default_dir, &args, default_dir));
    }

    /// A plain Finder/Dock launch carries no `--user-data-dir`, so it maps to the
    /// default data dir (the existing Claude), not to any created environment.
    #[test]
    fn desktop_dir_running_no_flag_counts_as_default() {
        let default_dir = Path::new("/Users/x/Library/Application Support/Claude");
        let work = Path::new("/Users/x/.context-switcher-claude/profiles/Work/desktop-data");
        let args = vec!["/Applications/Claude.app/Contents/MacOS/Claude".to_string()];
        assert!(desktop_dir_running(default_dir, &args, default_dir));
        assert!(!desktop_dir_running(work, &args, default_dir));
    }

    /// CSW launches even the default profile with an explicit `--user-data-dir`
    /// that contains spaces ("Application Support"); the match must still work and
    /// must not spuriously match a longer sibling path ("Claude2").
    #[test]
    fn desktop_dir_running_handles_spaces_and_prefix_siblings() {
        let default_dir = Path::new("/Users/x/Library/Application Support/Claude");
        let sibling = "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir=/Users/x/Library/Application Support/Claude2".to_string();
        assert!(!desktop_dir_running(default_dir, &[sibling], default_dir));

        let exact = format!(
            "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={} --enable-logging",
            default_dir.display()
        );
        assert!(desktop_dir_running(default_dir, &[exact], default_dir));
    }

    /// Two fully-isolated environments can run at once; both count as running.
    #[test]
    fn desktop_dir_running_supports_multiple_concurrent() {
        let default_dir = Path::new("/d/Application Support/Claude");
        let a = Path::new("/p/A/desktop-data");
        let b = Path::new("/p/B/desktop-data");
        let args = vec![
            format!("...MacOS/Claude --user-data-dir={}", a.display()),
            format!("...MacOS/Claude --user-data-dir={}", b.display()),
        ];
        assert!(desktop_dir_running(a, &args, default_dir));
        assert!(desktop_dir_running(b, &args, default_dir));
        assert!(!desktop_dir_running(default_dir, &args, default_dir));
    }

    #[test]
    fn desktop_dir_running_false_when_nothing_running() {
        let default_dir = Path::new("/d");
        let work = Path::new("/w");
        assert!(!desktop_dir_running(work, &[], default_dir));
        assert!(!desktop_dir_running(default_dir, &[], default_dir));
    }
}

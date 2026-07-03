use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CswError, Result};
use crate::platform::PlatformProvider;
use crate::profile::{Profile, SharingMode};

/// Which of a profile's two data directories an item lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemDir {
    Desktop,
    Cli,
}

/// One linker-managed link point inside a profile. This table is the single
/// source of truth for what the linker touches: `link_profile`,
/// `unlink_profile` and the isolation inspector all iterate it, so they can
/// never drift apart. Keys follow docs/SPECIFICATION.md §3.
pub struct LinkItem {
    pub key: &'static str,
    pub dir: ItemDir,
    pub rel_path: &'static str,
    pub is_directory: bool,
    /// `Some(mode)` for structurally fixed items (always isolated: they hold
    /// account-keyed state, runtime state or the device id, and no SharingConfig
    /// field can express sharing them). `None` reads the profile's SharingConfig.
    pub fixed_mode: Option<SharingMode>,
}

/// All link points, in the order `link_profile` applies them.
pub const LINK_ITEMS: &[LinkItem] = &[
    // claude_desktop_config.json — ALWAYS isolated. It holds account-keyed
    // permission gates and is rewritten via temp+rename on launch (breaking any
    // symlink). The isolation is structural, not a runtime guard.
    LinkItem {
        key: "desktop_config",
        dir: ItemDir::Desktop,
        rel_path: "claude_desktop_config.json",
        is_directory: false,
        fixed_mode: Some(SharingMode::Isolate),
    },
    LinkItem {
        key: "cli_settings",
        dir: ItemDir::Cli,
        rel_path: "settings.json",
        is_directory: false,
        fixed_mode: None,
    },
    LinkItem {
        key: "cli_claude_md",
        dir: ItemDir::Cli,
        rel_path: "CLAUDE.md",
        is_directory: false,
        fixed_mode: None,
    },
    LinkItem {
        key: "cli_project_memory",
        dir: ItemDir::Cli,
        rel_path: "projects",
        is_directory: true,
        fixed_mode: None,
    },
    LinkItem {
        key: "cli_plugins",
        dir: ItemDir::Cli,
        rel_path: "plugins",
        is_directory: true,
        fixed_mode: None,
    },
    LinkItem {
        key: "cli_skills",
        dir: ItemDir::Cli,
        rel_path: "skills",
        is_directory: true,
        fixed_mode: None,
    },
    // sessions/ — ALWAYS isolated. Per-environment runtime state (pid, cwd,
    // start time), not conversation content.
    LinkItem {
        key: "cli_sessions",
        dir: ItemDir::Cli,
        rel_path: "sessions",
        is_directory: true,
        fixed_mode: Some(SharingMode::Isolate),
    },
    LinkItem {
        key: "cli_history",
        dir: ItemDir::Cli,
        rel_path: "history.jsonl",
        is_directory: false,
        fixed_mode: None,
    },
    LinkItem {
        key: "desktop_worktrees",
        dir: ItemDir::Desktop,
        rel_path: "git-worktrees.json",
        is_directory: false,
        fixed_mode: None,
    },
    // ant-did — ALWAYS isolated. Sharing the device id would correlate two
    // accounts as one device.
    LinkItem {
        key: "desktop_device_id",
        dir: ItemDir::Desktop,
        rel_path: "ant-did",
        is_directory: false,
        fixed_mode: Some(SharingMode::Isolate),
    },
    // config.json — ALWAYS isolated. Holds OAuth token caches and per-account
    // state; sharing it would mix two logins into one file.
    LinkItem {
        key: "desktop_app_config",
        dir: ItemDir::Desktop,
        rel_path: "config.json",
        is_directory: false,
        fixed_mode: Some(SharingMode::Isolate),
    },
];

/// The sharing mode a profile declares (or the structure fixes) for an item.
pub fn item_mode(profile: &Profile, item: &LinkItem) -> SharingMode {
    if let Some(fixed) = &item.fixed_mode {
        return fixed.clone();
    }
    match item.key {
        "cli_settings" => profile.sharing.cli_settings.clone(),
        "cli_claude_md" => profile.sharing.cli_claude_md.clone(),
        "cli_project_memory" => profile.sharing.cli_project_memory.clone(),
        "cli_plugins" => profile.sharing.cli_plugins.clone(),
        "cli_skills" => profile.sharing.cli_skills.clone(),
        "cli_history" => profile.sharing.cli_history.clone(),
        "desktop_worktrees" => profile.sharing.desktop_worktrees.clone(),
        _ => SharingMode::Isolate,
    }
}

/// The absolute path of an item inside the given profile's data directories.
pub fn item_path(profile: &Profile, item: &LinkItem) -> PathBuf {
    match item.dir {
        ItemDir::Desktop => profile.isolation.desktop_user_data_dir.join(item.rel_path),
        ItemDir::Cli => profile.isolation.cli_config_dir.join(item.rel_path),
    }
}

/// Manages symlinks and files setup for a profile based on its sharing configuration.
pub struct Linker<'a> {
    provider: &'a dyn PlatformProvider,
}

impl<'a> Linker<'a> {
    pub fn new(provider: &'a dyn PlatformProvider) -> Self {
        Self { provider }
    }

    /// Links or copies all components of a target profile from a source profile.
    /// Iterates `LINK_ITEMS`, so the applied set always matches what
    /// `unlink_profile` cleans up and what the isolation inspector checks.
    pub fn link_profile(&self, target_profile: &Profile, source_profile: &Profile) -> Result<()> {
        // Ensure base target directories exist
        fs::create_dir_all(&target_profile.isolation.desktop_user_data_dir)?;
        fs::create_dir_all(&target_profile.isolation.cli_config_dir)?;

        for item in LINK_ITEMS {
            self.apply_link(
                &item_path(source_profile, item),
                &item_path(target_profile, item),
                item_mode(target_profile, item),
                item.is_directory,
            )?;
        }

        Ok(())
    }

    /// Cleans up any symlinks created in the target profile.
    pub fn unlink_profile(&self, target_profile: &Profile) -> Result<()> {
        let paths_to_unlink: Vec<PathBuf> = LINK_ITEMS
            .iter()
            .map(|item| item_path(target_profile, item))
            .collect();

        for path in paths_to_unlink {
            if path.exists() || self.provider.is_symlink(&path) {
                if self.provider.is_symlink(&path) {
                    self.provider.remove_symlink(&path)?;
                } else if path.is_file() {
                    self.assert_safe_to_delete(&path)?;
                    fs::remove_file(&path)?;
                } else if path.is_dir() {
                    self.assert_safe_to_delete(&path)?;
                    fs::remove_dir_all(&path)?;
                }
            }
        }

        Ok(())
    }

    /// Refuse destructive operations on the user's real default Claude data.
    ///
    /// Profiles other than "default" always live under their own per-profile
    /// directories, so the linker should never delete anything inside the real
    /// default Desktop/CLI dirs. This guard is defense-in-depth: if a profile is
    /// ever misconfigured (or a future change resolves a target into the default
    /// dir), refuse rather than wipe the user's environment.
    fn assert_safe_to_delete(&self, path: &Path) -> Result<()> {
        let defaults = [
            self.provider.claude_desktop_default_dir(),
            self.provider.claude_cli_default_dir(),
        ];
        if defaults
            .iter()
            .any(|root| path == root.as_path() || path.starts_with(root))
        {
            return Err(CswError::RefusedDefaultDataDeletion(
                path.display().to_string(),
            ));
        }
        Ok(())
    }

    fn apply_link(
        &self,
        source: &Path,
        target: &Path,
        mode: SharingMode,
        is_directory: bool,
    ) -> Result<()> {
        // A pre-existing symlink points elsewhere, so removing it never loses
        // real data; clear it before re-applying.
        if self.provider.is_symlink(target) {
            self.provider.remove_symlink(target)?;
        } else if target.exists() && mode != SharingMode::Share {
            // For Copy/Isolate, an existing real target is left untouched.
            return Ok(());
        }

        match mode {
            SharingMode::Share => {
                if source.exists() {
                    // Replace the target with a symlink without a destructive
                    // gap: a real target is moved aside and only dropped once the
                    // symlink exists (restored on failure).
                    self.replace_with_symlink(source, target)?;
                } else if is_directory && !target.exists() {
                    // No source to share yet: just ensure the directory exists.
                    fs::create_dir_all(target)?;
                }
                // Source missing but a real target exists: leave it as-is rather
                // than destroying real data for nothing.
            }
            SharingMode::Copy => {
                if source.exists() {
                    if is_directory {
                        self.copy_dir_all(source, target)?;
                    } else {
                        fs::copy(source, target)?;
                    }
                }
            }
            SharingMode::Isolate => {
                if is_directory {
                    fs::create_dir_all(target)?;
                }
            }
        }

        Ok(())
    }

    /// Replace `target` with a symlink to `source`. If `target` is an existing
    /// real (non-symlink) file or directory, move it aside first and only delete
    /// the backup once the symlink is created — restoring it on failure — so a
    /// crash mid-operation never loses the data.
    fn replace_with_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        if !target.exists() {
            self.provider.create_symlink(source, target)?;
            return Ok(());
        }

        // A real target is about to be replaced; never touch the real default data.
        self.assert_safe_to_delete(target)?;

        let backup = Self::backup_path(target);
        Self::remove_path(&backup)?; // clear any stale backup from an interrupted run
        fs::rename(target, &backup)?;

        match self.provider.create_symlink(source, target) {
            Ok(()) => {
                Self::remove_path(&backup)?;
                Ok(())
            }
            Err(e) => {
                // Roll back: restore the original target.
                let _ = fs::rename(&backup, target);
                Err(e)
            }
        }
    }

    /// Sibling backup path used while atomically replacing a target.
    fn backup_path(target: &Path) -> PathBuf {
        let mut name = target
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        name.push(".csw-backup");
        target.with_file_name(name)
    }

    /// Remove a file or directory if it exists (no-op otherwise).
    fn remove_path(path: &Path) -> Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else if path.symlink_metadata().is_ok() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn copy_dir_all(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                self.copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::mock::MockPlatformProvider;
    use crate::profile::{IsolationConfig, ProfileMeta, SharingConfig};
    use std::fs;
    use tempfile::tempdir;

    /// The linker must never destroy the user's real default Claude data.
    /// Even if a profile is (mis)configured so its isolation directories point
    /// straight at the real default dirs, `unlink_profile` must refuse to delete
    /// anything inside them rather than wiping the user's environment.
    #[test]
    fn unlink_profile_refuses_to_delete_real_default_data() {
        let desktop_dir = tempdir().unwrap();
        let cli_dir = tempdir().unwrap();
        let app_dir = tempdir().unwrap();

        // Real user memory living in the default CLI dir.
        let projects = cli_dir.path().join("projects");
        fs::create_dir_all(&projects).unwrap();
        let memory = projects.join("important.md");
        fs::write(&memory, "real user memory").unwrap();

        let provider = MockPlatformProvider::new(
            desktop_dir.path().to_path_buf(),
            cli_dir.path().to_path_buf(),
            app_dir.path().to_path_buf(),
        );

        // Profile whose isolation dirs ARE the real default dirs.
        let profile = Profile {
            profile: ProfileMeta {
                name: "danger".to_string(),
                icon: String::new(),
                color: String::new(),
                is_default: false,
                note: String::new(),
                created_at: None,
                cloned_from: None,
            },
            isolation: IsolationConfig {
                desktop_user_data_dir: desktop_dir.path().to_path_buf(),
                cli_config_dir: cli_dir.path().to_path_buf(),
            },
            sharing: SharingConfig::default(),
        };

        let linker = Linker::new(&provider);
        let result = linker.unlink_profile(&profile);

        assert!(
            result.is_err(),
            "unlink_profile must refuse to operate on the real default data dirs"
        );
        assert!(
            memory.exists(),
            "real user data inside the default dir must never be deleted"
        );
    }

    /// `apply_link` in Share mode clears a pre-existing non-symlink target
    /// before linking. That clear must also refuse to touch the real default
    /// dirs (the most dangerous remove_dir_all path).
    #[test]
    fn apply_link_share_refuses_to_clear_real_default_dir() {
        let desktop_dir = tempdir().unwrap();
        let cli_dir = tempdir().unwrap();
        let app_dir = tempdir().unwrap();

        // Real user plugins dir in the default CLI location.
        let plugins = cli_dir.path().join("plugins");
        fs::create_dir_all(&plugins).unwrap();
        let plugin = plugins.join("real_plugin.js");
        fs::write(&plugin, "// real").unwrap();

        // A source elsewhere to share from.
        let source_dir = tempdir().unwrap();
        let source_plugins = source_dir.path().join("plugins");
        fs::create_dir_all(&source_plugins).unwrap();

        let provider = MockPlatformProvider::new(
            desktop_dir.path().to_path_buf(),
            cli_dir.path().to_path_buf(),
            app_dir.path().to_path_buf(),
        );

        let linker = Linker::new(&provider);
        let result = linker.apply_link(&source_plugins, &plugins, SharingMode::Share, true);

        assert!(
            result.is_err(),
            "apply_link must refuse to clear a real default directory"
        );
        assert!(plugin.exists(), "real user plugin must survive");
    }

    /// The guard must not break the normal case: deleting a genuine per-profile
    /// directory (outside the default dirs) still works.
    #[test]
    fn apply_link_share_clears_non_default_target() {
        let desktop_dir = tempdir().unwrap();
        let cli_dir = tempdir().unwrap();
        let app_dir = tempdir().unwrap();
        let profile_dir = tempdir().unwrap();

        // A per-profile target dir (NOT under the default dirs).
        let target = profile_dir.path().join("plugins");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("old.js"), "// stale").unwrap();

        let source_dir = tempdir().unwrap();
        let source_plugins = source_dir.path().join("plugins");
        fs::create_dir_all(&source_plugins).unwrap();

        let provider = MockPlatformProvider::new(
            desktop_dir.path().to_path_buf(),
            cli_dir.path().to_path_buf(),
            app_dir.path().to_path_buf(),
        );

        let linker = Linker::new(&provider);
        linker
            .apply_link(&source_plugins, &target, SharingMode::Share, true)
            .expect("clearing a non-default profile dir must succeed");

        // After Share, the target becomes a symlink to the source.
        assert!(provider.is_symlink(&target));
    }

    /// If creating the symlink fails mid-operation, a pre-existing real target
    /// (and its data) must be restored rather than left deleted.
    #[test]
    fn apply_link_share_restores_target_when_symlink_fails() {
        let desktop_dir = tempdir().unwrap();
        let cli_dir = tempdir().unwrap();
        let app_dir = tempdir().unwrap();
        let profile_dir = tempdir().unwrap();

        // Real per-profile target dir (NOT under the default dirs) with data.
        let target = profile_dir.path().join("plugins");
        fs::create_dir_all(&target).unwrap();
        let data = target.join("keep.js");
        fs::write(&data, "important").unwrap();

        let source_dir = tempdir().unwrap();
        let source_plugins = source_dir.path().join("plugins");
        fs::create_dir_all(&source_plugins).unwrap();

        // Provider whose create_symlink always fails.
        let provider = MockPlatformProvider::new(
            desktop_dir.path().to_path_buf(),
            cli_dir.path().to_path_buf(),
            app_dir.path().to_path_buf(),
        )
        .with_symlink_failure(true);

        let linker = Linker::new(&provider);
        let result = linker.apply_link(&source_plugins, &target, SharingMode::Share, true);

        assert!(result.is_err(), "a failed symlink must surface an error");
        assert!(target.exists(), "the original target must be restored");
        assert!(data.exists(), "the original data must be preserved");
        assert_eq!(fs::read_to_string(&data).unwrap(), "important");
        let backup = target.with_file_name("plugins.csw-backup");
        assert!(!backup.exists(), "no backup should be left behind");
    }
}

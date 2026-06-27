use std::fs;
use std::path::Path;

use crate::error::{CswError, Result};
use crate::platform::PlatformProvider;
use crate::profile::{Profile, SharingMode};

/// Manages symlinks and files setup for a profile based on its sharing configuration.
pub struct Linker<'a> {
    provider: &'a dyn PlatformProvider,
}

impl<'a> Linker<'a> {
    pub fn new(provider: &'a dyn PlatformProvider) -> Self {
        Self { provider }
    }

    /// Links or copies all components of a target profile from a source profile.
    pub fn link_profile(&self, target_profile: &Profile, source_profile: &Profile) -> Result<()> {
        let target_desktop_dir = &target_profile.isolation.desktop_user_data_dir;
        let target_cli_dir = &target_profile.isolation.cli_config_dir;

        // Ensure base target directories exist
        fs::create_dir_all(target_desktop_dir)?;
        fs::create_dir_all(target_cli_dir)?;

        let source_desktop_dir = &source_profile.isolation.desktop_user_data_dir;
        let source_cli_dir = &source_profile.isolation.cli_config_dir;

        // 1. Desktop Config (claude_desktop_config.json)
        self.apply_link(
            &source_desktop_dir.join("claude_desktop_config.json"),
            &target_desktop_dir.join("claude_desktop_config.json"),
            target_profile.sharing.desktop_config.clone(),
            false, // is_directory
        )?;

        // 2. CLI Settings (settings.json)
        self.apply_link(
            &source_cli_dir.join("settings.json"),
            &target_cli_dir.join("settings.json"),
            target_profile.sharing.cli_settings.clone(),
            false,
        )?;

        // 3. CLI Claude.md (CLAUDE.md)
        self.apply_link(
            &source_cli_dir.join("CLAUDE.md"),
            &target_cli_dir.join("CLAUDE.md"),
            target_profile.sharing.cli_claude_md.clone(),
            false,
        )?;

        // 4. CLI Project Memory (projects/ directory)
        self.apply_link(
            &source_cli_dir.join("projects"),
            &target_cli_dir.join("projects"),
            target_profile.sharing.cli_project_memory.clone(),
            true, // is_directory
        )?;

        // 5. CLI Plugins (plugins/ directory)
        self.apply_link(
            &source_cli_dir.join("plugins"),
            &target_cli_dir.join("plugins"),
            target_profile.sharing.cli_plugins.clone(),
            true,
        )?;

        // 5b. CLI Skills (skills/ directory)
        self.apply_link(
            &source_cli_dir.join("skills"),
            &target_cli_dir.join("skills"),
            target_profile.sharing.cli_skills.clone(),
            true,
        )?;

        // 5c. CLI Sessions (sessions/ directory)
        self.apply_link(
            &source_cli_dir.join("sessions"),
            &target_cli_dir.join("sessions"),
            target_profile.sharing.cli_sessions.clone(),
            true,
        )?;

        // 5d. CLI Command History (history.jsonl file)
        self.apply_link(
            &source_cli_dir.join("history.jsonl"),
            &target_cli_dir.join("history.jsonl"),
            target_profile.sharing.cli_history.clone(),
            false,
        )?;

        // 6. Desktop Worktrees (git-worktrees.json)
        self.apply_link(
            &source_desktop_dir.join("git-worktrees.json"),
            &target_desktop_dir.join("git-worktrees.json"),
            target_profile.sharing.desktop_worktrees.clone(),
            false,
        )?;

        // 7. Desktop Device ID (ant-did)
        self.apply_link(
            &source_desktop_dir.join("ant-did"),
            &target_desktop_dir.join("ant-did"),
            target_profile.sharing.desktop_device_id.clone(),
            false,
        )?;

        // 8. Desktop App Configuration (config.json)
        self.apply_link(
            &source_desktop_dir.join("config.json"),
            &target_desktop_dir.join("config.json"),
            target_profile.sharing.desktop_app_config.clone(),
            false,
        )?;

        Ok(())
    }

    /// Cleans up any symlinks created in the target profile.
    pub fn unlink_profile(&self, target_profile: &Profile) -> Result<()> {
        let target_desktop_dir = &target_profile.isolation.desktop_user_data_dir;
        let target_cli_dir = &target_profile.isolation.cli_config_dir;

        let paths_to_unlink = vec![
            target_desktop_dir.join("claude_desktop_config.json"),
            target_desktop_dir.join("git-worktrees.json"),
            target_desktop_dir.join("ant-did"),
            target_desktop_dir.join("config.json"),
            target_cli_dir.join("settings.json"),
            target_cli_dir.join("CLAUDE.md"),
            target_cli_dir.join("projects"),
            target_cli_dir.join("plugins"),
            target_cli_dir.join("skills"),
            target_cli_dir.join("sessions"),
            target_cli_dir.join("history.jsonl"),
        ];

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
        // Clear target if it exists and is a symlink (avoid duplicates/errors)
        if self.provider.is_symlink(target) {
            self.provider.remove_symlink(target)?;
        } else if target.exists() {
            // Keep existing files if they were manually created or copies,
            // but if we are switching to Share, we must clear them.
            if mode == SharingMode::Share {
                self.assert_safe_to_delete(target)?;
                if target.is_file() {
                    fs::remove_file(target)?;
                } else {
                    fs::remove_dir_all(target)?;
                }
            } else {
                // If target exists and mode is copy or isolate, leave it as is
                return Ok(());
            }
        }

        // Apply based on mode
        match mode {
            SharingMode::Share => {
                if source.exists() {
                    self.provider.create_symlink(source, target)?;
                } else {
                    // Source doesn't exist, we don't symlink yet.
                    // Instead, we might create the directory structure if it is a directory.
                    if is_directory {
                        fs::create_dir_all(target)?;
                    }
                }
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
                } else {
                    // Create an empty file or basic default config if needed,
                    // but for most, leaving it empty/absent is correct.
                }
            }
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
}

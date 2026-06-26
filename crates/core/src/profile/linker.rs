use std::fs;
use std::path::{Path, PathBuf};

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
            target_cli_dir.join("settings.json"),
            target_cli_dir.join("CLAUDE.md"),
            target_cli_dir.join("projects"),
            target_cli_dir.join("plugins"),
        ];

        for path in paths_to_unlink {
            if path.exists() || self.provider.is_symlink(&path) {
                if self.provider.is_symlink(&path) {
                    self.provider.remove_symlink(&path)?;
                } else if path.is_file() {
                    fs::remove_file(&path)?;
                } else if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                }
            }
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

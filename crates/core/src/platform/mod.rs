#[cfg(target_os = "macos")]
pub mod macos;

use std::path::PathBuf;

use crate::error::Result;

/// Platform-specific operations abstracted behind a common trait.
pub trait PlatformProvider: Send + Sync {
    // --- Path resolution ---

    /// Default Claude Desktop user-data-dir.
    /// macOS: ~/Library/Application Support/Claude/
    fn claude_desktop_default_dir(&self) -> PathBuf;

    /// Default Claude Code (CLI) config dir.
    /// macOS: ~/.claude/
    fn claude_cli_default_dir(&self) -> PathBuf;

    /// Path to the Claude Desktop application binary.
    /// macOS: /Applications/Claude.app
    fn claude_desktop_app_path(&self) -> PathBuf;

    /// Root directory for ContextSwitcher's own data.
    /// macOS: ~/.context-switcher-claude/
    fn app_data_dir(&self) -> PathBuf;

    // --- Symlink operations ---

    /// Create a symbolic link from `link_path` pointing to `target_path`.
    /// The link_path is in the NEW profile directory.
    /// The target_path is the EXISTING file/directory to share.
    fn create_symlink(
        &self,
        target_path: &std::path::Path,
        link_path: &std::path::Path,
    ) -> Result<()>;

    /// Remove a symbolic link. Errors if the path is not a symlink.
    fn remove_symlink(&self, link_path: &std::path::Path) -> Result<()>;

    /// Check if a path is a symbolic link.
    fn is_symlink(&self, path: &std::path::Path) -> bool;

    /// Move a file or directory to the OS trash (macOS: the Trash). Never a
    /// silent fallback to permanent deletion: on failure the caller reports
    /// the error and offers an explicit purge instead.
    fn move_to_trash(&self, path: &std::path::Path) -> Result<()>;

    // --- Process control ---

    /// Launch Claude Desktop with a specific user-data-dir.
    fn launch_claude_desktop(
        &self,
        user_data_dir: &std::path::Path,
        cli_config_dir: Option<&std::path::Path>,
    ) -> Result<()>;

    /// Check if Claude Desktop is currently running.
    fn is_claude_desktop_running(&self) -> Result<bool>;

    /// Get the PID(s) of running Claude Desktop processes.
    fn claude_desktop_pids(&self) -> Result<Vec<u32>>;

    /// The argument strings of running Claude Desktop *main* processes (Electron
    /// renderer / GPU / utility helpers excluded). Each entry is one instance's
    /// command line, used to tell which environment (`--user-data-dir`) is running.
    /// Empty when no Claude is running.
    fn running_desktop_args(&self) -> Result<Vec<String>>;

    /// What owns the frontmost window right now. Read on demand only (the
    /// tray's "which environment is this?" action), never polled: reading the
    /// frontmost application is a privacy-relevant lookup and stays tied to an
    /// explicit user action (docs/PRIVACY.md).
    fn frontmost_app(&self) -> Result<FrontmostApp>;
}

/// The owner of the frontmost window, as far as CSW needs to know: another
/// app, CSW itself (the settings window), or a Claude Desktop main process
/// carrying its args line for environment resolution.
#[derive(Debug, Clone, PartialEq)]
pub enum FrontmostApp {
    OtherApp,
    ThisApp,
    Claude(String),
}

/// Create the platform provider for the current OS.
pub fn create_provider() -> Result<Box<dyn PlatformProvider>> {
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacOsProvider::new()))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(crate::error::CswError::UnsupportedPlatform(
            std::env::consts::OS.to_string(),
        ))
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod mock;

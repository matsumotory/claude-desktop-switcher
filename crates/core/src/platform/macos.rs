use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{CswError, Result};
use crate::platform::PlatformProvider;

pub struct MacOsProvider {
    home_dir: PathBuf,
}

impl Default for MacOsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOsProvider {
    pub fn new() -> Self {
        Self {
            home_dir: dirs::home_dir().expect("Failed to determine home directory"),
        }
    }
}

impl PlatformProvider for MacOsProvider {
    fn claude_desktop_default_dir(&self) -> PathBuf {
        self.home_dir.join("Library/Application Support/Claude")
    }

    fn claude_cli_default_dir(&self) -> PathBuf {
        self.home_dir.join(".claude")
    }

    fn claude_desktop_app_path(&self) -> PathBuf {
        PathBuf::from("/Applications/Claude.app")
    }

    fn app_data_dir(&self) -> PathBuf {
        self.home_dir.join(".context-switcher-claude")
    }

    fn create_symlink(&self, target_path: &Path, link_path: &Path) -> Result<()> {
        // Guard: target must exist
        if !target_path.exists() {
            return Err(CswError::Other(format!(
                "Symlink target does not exist: {}",
                target_path.display()
            )));
        }

        // Guard: link_path must not already exist (non-destructive)
        if link_path.exists() && !link_path.is_symlink() {
            return Err(CswError::NonDestructiveViolation(
                link_path.display().to_string(),
            ));
        }

        // Remove existing symlink if present
        if link_path.is_symlink() {
            std::fs::remove_file(link_path)?;
        }

        // Create parent directories if needed
        if let Some(parent) = link_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::os::unix::fs::symlink(target_path, link_path).map_err(|e| CswError::SymlinkFailed {
            source: link_path.display().to_string(),
            target: target_path.display().to_string(),
            cause: e,
        })
    }

    fn remove_symlink(&self, link_path: &Path) -> Result<()> {
        if !link_path.is_symlink() {
            return Err(CswError::Other(format!(
                "Not a symlink: {}",
                link_path.display()
            )));
        }
        std::fs::remove_file(link_path)?;
        Ok(())
    }

    fn is_symlink(&self, path: &Path) -> bool {
        path.is_symlink()
    }

    fn launch_claude_desktop(
        &self,
        user_data_dir: &Path,
        cli_config_dir: Option<&Path>,
    ) -> Result<()> {
        let app_path = self.claude_desktop_app_path();
        if !app_path.exists() {
            return Err(CswError::DesktopNotInstalled);
        }

        let mut cmd = Command::new("open");
        cmd.arg("-n").arg("-a").arg(&app_path);

        if let Some(cli_dir) = cli_config_dir {
            cmd.arg("--env")
                .arg(format!("CLAUDE_CONFIG_DIR={}", cli_dir.display()));
        }

        cmd.arg("--args")
            .arg(format!("--user-data-dir={}", user_data_dir.display()))
            .spawn()
            .map_err(|e| CswError::Other(format!("Failed to launch Claude Desktop: {e}")))?;

        Ok(())
    }

    fn is_claude_desktop_running(&self) -> Result<bool> {
        Ok(!self.claude_desktop_pids()?.is_empty())
    }

    fn claude_desktop_pids(&self) -> Result<Vec<u32>> {
        let output = Command::new("pgrep")
            .arg("-f")
            .arg("Claude.app/Contents/MacOS/Claude")
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let pids: Vec<u32> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect();

        Ok(pids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_paths() {
        let provider = MacOsProvider::new();
        let desktop_dir = provider.claude_desktop_default_dir();
        assert!(
            desktop_dir
                .to_string_lossy()
                .contains("Application Support/Claude")
        );

        let cli_dir = provider.claude_cli_default_dir();
        assert!(cli_dir.to_string_lossy().ends_with(".claude"));

        let app_data = provider.app_data_dir();
        assert!(
            app_data
                .to_string_lossy()
                .ends_with(".context-switcher-claude")
        );
    }

    #[test]
    fn test_symlink_operations() {
        let provider = MacOsProvider::new();
        let tmp = std::env::temp_dir().join("csw-test-symlink");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let target = tmp.join("target.txt");
        std::fs::write(&target, "hello").unwrap();

        let link = tmp.join("link.txt");
        provider.create_symlink(&target, &link).unwrap();

        assert!(provider.is_symlink(&link));
        assert_eq!(std::fs::read_to_string(&link).unwrap(), "hello");

        provider.remove_symlink(&link).unwrap();
        assert!(!link.exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_non_destructive_guard() {
        let provider = MacOsProvider::new();
        let tmp = std::env::temp_dir().join("csw-test-guard");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let target = tmp.join("target.txt");
        std::fs::write(&target, "original").unwrap();

        // Existing real file at link location should fail
        let existing = tmp.join("existing.txt");
        std::fs::write(&existing, "do not overwrite").unwrap();

        let result = provider.create_symlink(&target, &existing);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CswError::NonDestructiveViolation(_)
        ));

        // Verify existing file was not modified
        assert_eq!(
            std::fs::read_to_string(&existing).unwrap(),
            "do not overwrite"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

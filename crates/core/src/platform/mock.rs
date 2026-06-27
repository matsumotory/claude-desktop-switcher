use crate::error::Result;
use crate::platform::PlatformProvider;
use std::path::PathBuf;

pub struct MockPlatformProvider {
    pub desktop_default: PathBuf,
    pub cli_default: PathBuf,
    pub app_data: PathBuf,
}

impl MockPlatformProvider {
    pub fn new(desktop_default: PathBuf, cli_default: PathBuf, app_data: PathBuf) -> Self {
        Self {
            desktop_default,
            cli_default,
            app_data,
        }
    }
}

impl PlatformProvider for MockPlatformProvider {
    fn claude_desktop_default_dir(&self) -> PathBuf {
        self.desktop_default.clone()
    }

    fn claude_cli_default_dir(&self) -> PathBuf {
        self.cli_default.clone()
    }

    fn claude_desktop_app_path(&self) -> PathBuf {
        PathBuf::from("/Applications/Claude.app")
    }

    fn app_data_dir(&self) -> PathBuf {
        self.app_data.clone()
    }

    fn create_symlink(
        &self,
        target_path: &std::path::Path,
        link_path: &std::path::Path,
    ) -> Result<()> {
        #[cfg(unix)]
        std::os::unix::fs::symlink(target_path, link_path)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(target_path, link_path)?;
        Ok(())
    }

    fn remove_symlink(&self, link_path: &std::path::Path) -> Result<()> {
        std::fs::remove_file(link_path)?;
        Ok(())
    }

    fn is_symlink(&self, path: &std::path::Path) -> bool {
        path.symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }

    fn launch_claude_desktop(
        &self,
        _user_data_dir: &std::path::Path,
        _cli_config_dir: Option<&std::path::Path>,
    ) -> Result<()> {
        Ok(())
    }

    fn is_claude_desktop_running(&self) -> Result<bool> {
        Ok(false)
    }

    fn claude_desktop_pids(&self) -> Result<Vec<u32>> {
        Ok(vec![])
    }
}

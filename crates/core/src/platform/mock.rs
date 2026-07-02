use crate::error::{CswError, Result};
use crate::platform::PlatformProvider;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct MockPlatformProvider {
    pub desktop_default: PathBuf,
    pub cli_default: PathBuf,
    pub app_data: PathBuf,
    /// Test knob: report Claude Desktop as running.
    desktop_running: AtomicBool,
    /// Test knob: make `create_symlink` fail (to exercise rollback paths).
    fail_symlink: AtomicBool,
    /// Test knob: make `move_to_trash` fail (to exercise the no-fallback path).
    fail_trash: AtomicBool,
    /// Test knob: arg lines returned by `running_desktop_args` (running main procs).
    running_args: std::sync::Mutex<Vec<String>>,
}

impl MockPlatformProvider {
    pub fn new(desktop_default: PathBuf, cli_default: PathBuf, app_data: PathBuf) -> Self {
        Self {
            desktop_default,
            cli_default,
            app_data,
            desktop_running: AtomicBool::new(false),
            fail_symlink: AtomicBool::new(false),
            fail_trash: AtomicBool::new(false),
            running_args: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Builder: make `is_claude_desktop_running` report `running`.
    pub fn with_desktop_running(self, running: bool) -> Self {
        self.desktop_running.store(running, Ordering::SeqCst);
        self
    }

    /// Builder: set the command-line arg strings returned by `running_desktop_args`
    /// (the running Claude main processes), for per-environment "in use" tests.
    pub fn with_running_args(self, args: Vec<String>) -> Self {
        *self.running_args.lock().unwrap() = args;
        self
    }

    /// Builder: make `create_symlink` fail, to test rollback/atomicity paths.
    pub fn with_symlink_failure(self, fail: bool) -> Self {
        self.fail_symlink.store(fail, Ordering::SeqCst);
        self
    }

    /// Builder: make `move_to_trash` fail, to test the no-fallback error path.
    pub fn with_trash_failure(self, fail: bool) -> Self {
        self.fail_trash.store(fail, Ordering::SeqCst);
        self
    }

    /// Where this mock "trashes" paths to (a directory next to app data).
    pub fn mock_trash_dir(&self) -> PathBuf {
        self.app_data.join("mock-trash")
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
        if self.fail_symlink.load(Ordering::SeqCst) {
            return Err(CswError::Other("mock: create_symlink failed".to_string()));
        }
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
        Ok(self.desktop_running.load(Ordering::SeqCst))
    }

    fn claude_desktop_pids(&self) -> Result<Vec<u32>> {
        Ok(vec![])
    }

    fn running_desktop_args(&self) -> Result<Vec<String>> {
        Ok(self.running_args.lock().unwrap().clone())
    }

    fn move_to_trash(&self, path: &std::path::Path) -> Result<()> {
        if self.fail_trash.load(Ordering::SeqCst) {
            return Err(CswError::TrashMoveFailed("mock failure".to_string()));
        }
        let trash = self.mock_trash_dir();
        std::fs::create_dir_all(&trash)?;
        let name = path
            .file_name()
            .ok_or_else(|| CswError::Other("mock: path has no file name".to_string()))?;
        std::fs::rename(path, trash.join(name))?;
        Ok(())
    }
}

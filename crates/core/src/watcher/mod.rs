use std::path::{Path, PathBuf};
use std::sync::Arc;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use crate::error::Result;
use crate::profile::{ProfileManager, SharingMode};

pub struct FileWatcher {
    profile_manager: Arc<ProfileManager>,
    watcher: Option<RecommendedWatcher>,
}

impl FileWatcher {
    pub fn new(profile_manager: Arc<ProfileManager>) -> Self {
        Self {
            profile_manager,
            watcher: None,
        }
    }

    /// Start watching the settings files for the active profile.
    /// If copy-mode configurations are edited, sync them to other profiles sharing the same source.
    pub fn start(&mut self) -> Result<()> {
        let active_profile = self.profile_manager.active_profile()?;
        let active_name = active_profile.profile.name.clone();
        let profile_manager = self.profile_manager.clone();

        // Find files to watch (only need to watch if SharingMode is Copy)
        let mut paths_to_watch = Vec::new();

        let cli_dir = &active_profile.isolation.cli_config_dir;
        let desktop_dir = &active_profile.isolation.desktop_user_data_dir;

        if active_profile.sharing.cli_settings == SharingMode::Copy {
            paths_to_watch.push(cli_dir.join("settings.json"));
        }
        if active_profile.sharing.cli_claude_md == SharingMode::Copy {
            paths_to_watch.push(cli_dir.join("CLAUDE.md"));
        }
        if active_profile.sharing.desktop_config == SharingMode::Copy {
            paths_to_watch.push(desktop_dir.join("claude_desktop_config.json"));
        }
        if active_profile.sharing.desktop_worktrees == SharingMode::Copy {
            paths_to_watch.push(desktop_dir.join("git-worktrees.json"));
        }

        if paths_to_watch.is_empty() {
            // Nothing to watch/sync
            return Ok(());
        }

        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    for path in event.paths {
                        let _ = tx.send(path);
                    }
                }
            }
        })?;

        for path in &paths_to_watch {
            if path.exists() {
                watcher.watch(path, RecursiveMode::NonRecursive)?;
            }
        }

        self.watcher = Some(watcher);

        // Spawn a background thread to process file modifications
        std::thread::spawn(move || {
            while let Ok(modified_path) = rx.recv() {
                let _ = Self::sync_file(&modified_path, &active_name, &profile_manager);
            }
        });

        Ok(())
    }

    fn sync_file(modified_path: &Path, active_name: &str, profile_manager: &ProfileManager) -> Result<()> {
        let file_name = match modified_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return Ok(()),
        };

        let profiles = profile_manager.list_profiles()?;
        for p_name in profiles {
            if p_name == active_name || p_name == "default" {
                continue;
            }

            let other_profile = profile_manager.get_profile(&p_name)?;
            // Check if this other profile shares the same source and has this component as Copy
            let is_match = match file_name {
                "settings.json" => {
                    other_profile.sharing.cli_settings == SharingMode::Copy
                        && other_profile.sharing.source.profile == "default"
                }
                "CLAUDE.md" => {
                    other_profile.sharing.cli_claude_md == SharingMode::Copy
                        && other_profile.sharing.source.profile == "default"
                }
                "claude_desktop_config.json" => {
                    other_profile.sharing.desktop_config == SharingMode::Copy
                        && other_profile.sharing.source.profile == "default"
                }
                "git-worktrees.json" => {
                    other_profile.sharing.desktop_worktrees == SharingMode::Copy
                        && other_profile.sharing.source.profile == "default"
                }
                _ => false,
            };

            if is_match {
                let target_path = match file_name {
                    "settings.json" | "CLAUDE.md" => {
                        other_profile.isolation.cli_config_dir.join(file_name)
                    }
                    "claude_desktop_config.json" | "git-worktrees.json" => {
                        other_profile.isolation.desktop_user_data_dir.join(file_name)
                    }
                    _ => continue,
                };

                // Perform copy sync
                if modified_path.exists() {
                    let _ = std::fs::copy(modified_path, target_path);
                }
            }
        }

        Ok(())
    }
}

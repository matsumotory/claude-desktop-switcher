pub mod config;
pub mod linker;

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Sharing mode for a configuration component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SharingMode {
    /// Fully isolated: new profile gets its own independent copy.
    Isolate,
    /// Shared via symlink to the source profile's file/directory.
    Share,
    /// One-time copy from the source (diverges after creation).
    Copy,
}

impl Default for SharingMode {
    fn default() -> Self {
        Self::Isolate
    }
}

/// A profile representing a Claude account environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub profile: ProfileMeta,
    pub isolation: IsolationConfig,
    pub sharing: SharingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub color: String,
    /// Whether this is the default (existing environment) profile.
    #[serde(default)]
    pub is_default: bool,
}

/// Paths for isolated session data (always per-profile).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    /// Path for Desktop user-data-dir (session, auth, conversations).
    pub desktop_user_data_dir: PathBuf,
    /// Path for CLI config dir (session, auth, history).
    pub cli_config_dir: PathBuf,
}

/// Per-component sharing/isolation preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConfig {
    /// claude_desktop_config.json (MCP servers, preferences)
    #[serde(default)]
    pub desktop_config: SharingMode,

    /// settings.json (permissions, hooks, theme, plugins config)
    #[serde(default)]
    pub cli_settings: SharingMode,

    /// CLAUDE.md (global personal rules, 17KB+)
    #[serde(default)]
    pub cli_claude_md: SharingMode,

    /// projects/<path>/memory/ directory (MEMORY.md index + feedback_*.md etc.)
    /// Total: ~250 files across all projects.
    #[serde(default)]
    pub cli_project_memory: SharingMode,

    /// plugins/ directory (installed plugins)
    #[serde(default)]
    pub cli_plugins: SharingMode,

    /// git-worktrees.json (worktree name → repo/branch mapping)
    #[serde(default)]
    pub desktop_worktrees: SharingMode,

    /// ant-did (device identifier, machine-unique)
    #[serde(default = "default_share")]
    pub desktop_device_id: SharingMode,

    /// Source profile for shared components (default: "default")
    #[serde(default)]
    pub source: SharingSource,
}

fn default_share() -> SharingMode {
    SharingMode::Share
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingSource {
    #[serde(default = "default_source_profile")]
    pub profile: String,
}

fn default_source_profile() -> String {
    "default".to_string()
}

impl Default for SharingSource {
    fn default() -> Self {
        Self {
            profile: default_source_profile(),
        }
    }
}

impl Default for SharingConfig {
    fn default() -> Self {
        Self {
            desktop_config: SharingMode::Isolate,
            cli_settings: SharingMode::Isolate,
            cli_claude_md: SharingMode::Isolate,
            cli_project_memory: SharingMode::Isolate,
            cli_plugins: SharingMode::Isolate,
            desktop_worktrees: SharingMode::Isolate,
            desktop_device_id: SharingMode::Share,
            source: SharingSource::default(),
        }
    }
}

pub struct ProfileManager {
    provider: Arc<dyn crate::platform::PlatformProvider>,
    app_config_path: PathBuf,
    app_config: std::sync::Mutex<config::AppConfig>,
}

impl ProfileManager {
    pub fn new(provider: Arc<dyn crate::platform::PlatformProvider>) -> Result<Self> {
        let app_data_dir = provider.app_data_dir();
        let app_config_path = app_data_dir.join("config.toml");
        
        let app_config = if app_config_path.exists() {
            config::AppConfig::load(&app_config_path)?
        } else {
            let default_config = config::AppConfig::default_for(&app_data_dir);
            default_config.save(&app_config_path)?;
            default_config
        };

        // Ensure default profile directory structure is initialized (virtual config only, no files modified)
        let manager = Self {
            provider,
            app_config_path,
            app_config: std::sync::Mutex::new(app_config),
        };

        Ok(manager)
    }

    pub fn active_profile_name(&self) -> String {
        self.app_config.lock().unwrap().active_profile.clone()
    }

    pub fn active_profile(&self) -> Result<Profile> {
        let name = self.active_profile_name();
        self.get_profile(&name)
    }

    pub fn get_profile(&self, name: &str) -> Result<Profile> {
        if name == "default" {
            return Ok(Profile {
                profile: ProfileMeta {
                    name: "default".to_string(),
                    icon: "💻".to_string(),
                    color: "#9E9E9E".to_string(),
                    is_default: true,
                },
                isolation: IsolationConfig {
                    desktop_user_data_dir: self.provider.claude_desktop_default_dir(),
                    cli_config_dir: self.provider.claude_cli_default_dir(),
                },
                sharing: SharingConfig {
                    desktop_config: SharingMode::Share,
                    cli_settings: SharingMode::Share,
                    cli_claude_md: SharingMode::Share,
                    cli_project_memory: SharingMode::Share,
                    cli_plugins: SharingMode::Share,
                    desktop_worktrees: SharingMode::Share,
                    desktop_device_id: SharingMode::Share,
                    source: SharingSource {
                        profile: "default".to_string(),
                    },
                },
            });
        }

        let profiles_dir = self.app_config.lock().unwrap().profiles_dir.clone();
        let profile_toml = profiles_dir.join(name).join("profile.toml");
        if !profile_toml.exists() {
            return Err(CswError::ProfileNotFound(name.to_string()));
        }

        config::load_profile(&profile_toml)
    }

    pub fn create_profile(&self, name: &str, sharing: SharingConfig) -> Result<Profile> {
        if name == "default" {
            return Err(CswError::ProfileAlreadyExists("default".to_string()));
        }

        let profiles_dir = self.app_config.lock().unwrap().profiles_dir.clone();
        let profile_dir = profiles_dir.join(name);
        let profile_toml = profile_dir.join("profile.toml");

        if profile_toml.exists() {
            return Err(CswError::ProfileAlreadyExists(name.to_string()));
        }

        let profile = Profile {
            profile: ProfileMeta {
                name: name.to_string(),
                icon: "💼".to_string(),
                color: "#4A90D9".to_string(),
                is_default: false,
            },
            isolation: IsolationConfig {
                desktop_user_data_dir: profile_dir.join("desktop-data"),
                cli_config_dir: profile_dir.join("cli-data"),
            },
            sharing,
        };

        config::save_profile(&profile, &profile_toml)?;

        // Apply linking immediately
        let source_profile = self.get_profile(&profile.sharing.source.profile)?;
        let linker = linker::Linker::new(self.provider.as_ref());
        linker.link_profile(&profile, &source_profile)?;

        Ok(profile)
    }

    pub fn delete_profile(&self, name: &str) -> Result<()> {
        if name == "default" {
            return Err(CswError::DefaultProfileCannotBeDeleted);
        }

        if self.active_profile_name() == name {
            return Err(CswError::ActiveProfileCannotBeDeleted(name.to_string()));
        }

        let profile = self.get_profile(name)?;
        let linker = linker::Linker::new(self.provider.as_ref());
        
        // Clean up symlinks first
        linker.unlink_profile(&profile)?;

        // Remove profile directory
        let profiles_dir = self.app_config.lock().unwrap().profiles_dir.clone();
        let profile_dir = profiles_dir.join(name);
        if profile_dir.exists() {
            std::fs::remove_dir_all(profile_dir)?;
        }

        Ok(())
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let profiles_dir = self.app_config.lock().unwrap().profiles_dir.clone();
        let mut names = config::list_profile_names(&profiles_dir)?;
        if !names.contains(&"default".to_string()) {
            names.insert(0, "default".to_string());
        }
        Ok(names)
    }

    pub fn switch_to(&self, name: &str) -> Result<()> {
        // Ensure profile exists
        let _profile = self.get_profile(name)?;

        let mut app_config = self.app_config.lock().unwrap();
        app_config.active_profile = name.to_string();
        app_config.save(&self.app_config_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;


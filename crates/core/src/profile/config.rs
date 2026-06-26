use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::profile::Profile;

/// Load a profile from a TOML file.
pub fn load_profile(path: &Path) -> Result<Profile> {
    let content = std::fs::read_to_string(path)?;
    let profile: Profile = toml::from_str(&content)?;
    Ok(profile)
}

/// Save a profile to a TOML file.
pub fn save_profile(profile: &Profile, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(profile)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Application-level configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// Currently active profile name.
    pub active_profile: String,
    /// Path to profiles directory.
    pub profiles_dir: PathBuf,
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn default_for(app_data_dir: &Path) -> Self {
        Self {
            active_profile: "default".to_string(),
            profiles_dir: app_data_dir.join("profiles"),
        }
    }
}

/// List all profile names in the profiles directory.
pub fn list_profile_names(profiles_dir: &Path) -> Result<Vec<String>> {
    if !profiles_dir.exists() {
        return Ok(vec![]);
    }

    let mut names = Vec::new();
    for entry in std::fs::read_dir(profiles_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let profile_toml = entry.path().join("profile.toml");
            if profile_toml.exists() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::*;

    #[test]
    fn test_profile_roundtrip() {
        let profile = Profile {
            profile: ProfileMeta {
                name: "test-work".to_string(),
                icon: "\u{1f4bc}".to_string(),
                color: "#4A90D9".to_string(),
                is_default: false,
            },
            isolation: IsolationConfig {
                desktop_user_data_dir: PathBuf::from("/tmp/test/desktop-data"),
                cli_config_dir: PathBuf::from("/tmp/test/cli-data"),
            },
            sharing: SharingConfig {
                desktop_config: SharingMode::Share,
                cli_settings: SharingMode::Share,
                cli_claude_md: SharingMode::Share,
                cli_project_memory: SharingMode::Isolate,
                cli_plugins: SharingMode::Share,
                desktop_worktrees: SharingMode::Share,
                desktop_device_id: SharingMode::Share,
                source: SharingSource {
                    profile: "default".to_string(),
                },
            },
        };

        let tmp = std::env::temp_dir().join("csw-test-profile");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let path = tmp.join("profile.toml");
        save_profile(&profile, &path).unwrap();

        let loaded = load_profile(&path).unwrap();
        assert_eq!(loaded.profile.name, "test-work");
        assert_eq!(loaded.sharing.desktop_config, SharingMode::Share);
        assert_eq!(loaded.sharing.cli_project_memory, SharingMode::Isolate);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

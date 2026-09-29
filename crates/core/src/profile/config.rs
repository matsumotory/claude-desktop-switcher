use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::profile::{Profile, SharingMode};

/// Load a profile from a TOML file.
pub fn load_profile(path: &Path) -> Result<Profile> {
    let content = std::fs::read_to_string(path)?;
    let mut profile: Profile = toml::from_str(&content)?;
    // A profile.toml written before `cli_rules` existed (issue #190) has no key
    // for it. Its rules directory follows CLAUDE.md: the mode descriptions
    // always counted both as the user's global rules, so an environment that
    // shares CLAUDE.md was meant to share rules/ too (the missing link is then
    // reported by the isolation check and created by `csw doctor --fix`). A
    // CLAUDE.md set to Copy does not carry over: nothing was ever copied for
    // rules/ in such an environment, and declaring Copy would show it as
    // "copied" when it is empty, so it reads as Isolate, its honest state.
    // An explicit key always wins.
    if !declares_cli_rules(&content)? {
        profile.sharing.cli_rules = match profile.sharing.cli_claude_md {
            SharingMode::Share => SharingMode::Share,
            SharingMode::Copy | SharingMode::Isolate => SharingMode::Isolate,
        };
    }
    Ok(profile)
}

/// Whether the TOML text spells out `sharing.cli_rules` (as opposed to serde
/// having filled in the field's default).
fn declares_cli_rules(content: &str) -> Result<bool> {
    let table: toml::Table = toml::from_str(content)?;
    Ok(table
        .get("sharing")
        .and_then(toml::Value::as_table)
        .is_some_and(|sharing| sharing.contains_key("cli_rules")))
}

/// Save a profile to a TOML file. Written via a temp file + rename so a crash
/// mid-write can never leave a truncated safety declaration behind.
pub fn save_profile(profile: &Profile, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(profile)?;
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
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
            if profile_toml.exists()
                && let Some(name) = entry.file_name().to_str()
            {
                names.push(name.to_string());
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
                note: "仕事用。会社の Google でサインイン".to_string(),
                created_at: Some("2026-07-03T00:00:00Z".to_string()),
                cloned_from: None,
            },
            isolation: IsolationConfig {
                desktop_user_data_dir: PathBuf::from("/tmp/test/desktop-data"),
                cli_config_dir: PathBuf::from("/tmp/test/cli-data"),
            },
            sharing: SharingConfig {
                cli_settings: SharingMode::Share,
                cli_claude_md: SharingMode::Share,
                cli_rules: SharingMode::Copy,
                cli_project_memory: SharingMode::Isolate,
                cli_plugins: SharingMode::Share,
                cli_skills: SharingMode::Share,
                cli_history: SharingMode::Share,
                desktop_worktrees: SharingMode::Share,
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
        assert_eq!(loaded.sharing.cli_claude_md, SharingMode::Share);
        assert_eq!(loaded.sharing.cli_rules, SharingMode::Copy);
        assert_eq!(loaded.sharing.cli_project_memory, SharingMode::Isolate);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// A profile.toml written before `cli_rules` existed (issue #190) must
    /// keep meaning what its mode description promised: a shared CLAUDE.md
    /// means a shared rules/ too. Anything else reads as Isolate, because no
    /// rules/ was ever copied into such an environment.
    fn profile_toml_without_cli_rules(claude_md: &str) -> String {
        format!(
            r##"[profile]
name = "old"
icon = ""
color = "#4A90D9"
is_default = false
note = ""

[isolation]
desktop_user_data_dir = "/tmp/old/desktop-data"
cli_config_dir = "/tmp/old/cli-data"

[sharing]
cli_settings = "copy"
cli_claude_md = "{claude_md}"
cli_project_memory = "share"
cli_plugins = "share"
cli_skills = "share"
cli_history = "share"
desktop_worktrees = "copy"

[sharing.source]
profile = "default"
"##
        )
    }

    fn load_toml_string(name: &str, content: &str) -> Profile {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(format!("{name}.toml"));
        std::fs::write(&path, content).unwrap();
        load_profile(&path).unwrap()
    }

    #[test]
    fn load_profile_without_cli_rules_follows_claude_md() {
        let shared = load_toml_string("shared", &profile_toml_without_cli_rules("share"));
        assert_eq!(shared.sharing.cli_claude_md, SharingMode::Share);
        assert_eq!(shared.sharing.cli_rules, SharingMode::Share);

        let isolated = load_toml_string("isolated", &profile_toml_without_cli_rules("isolate"));
        assert_eq!(isolated.sharing.cli_rules, SharingMode::Isolate);

        // Copy is not carried over: the old build never copied rules/, so the
        // environment must not present an empty directory as a finished copy.
        let copied = load_toml_string("copied", &profile_toml_without_cli_rules("copy"));
        assert_eq!(copied.sharing.cli_rules, SharingMode::Isolate);
    }

    #[test]
    fn load_profile_keeps_an_explicit_cli_rules() {
        // An explicit value always wins over the CLAUDE.md fallback.
        let content = profile_toml_without_cli_rules("share").replace(
            "cli_claude_md = \"share\"\n",
            "cli_claude_md = \"share\"\ncli_rules = \"isolate\"\n",
        );
        assert!(content.contains("cli_rules = \"isolate\""));
        let profile = load_toml_string("explicit", &content);
        assert_eq!(profile.sharing.cli_claude_md, SharingMode::Share);
        assert_eq!(profile.sharing.cli_rules, SharingMode::Isolate);
    }
}

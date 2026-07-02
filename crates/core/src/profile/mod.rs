pub mod config;
pub mod data_map;
pub mod inspector;
pub mod linker;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{CswError, Result};
use serde::{Deserialize, Serialize};

/// Validate a user-supplied environment (profile) name.
///
/// The name becomes a directory under `profiles/`, so it must not enable path
/// traversal, and it is passed to a shell in `eval $(csw env <name>)`, so it must
/// carry no shell metacharacters or whitespace. Unicode letters and digits
/// (including Japanese) plus `-` and `_` are allowed; everything else (path
/// separators, dots, spaces, symbols, control characters) is rejected. The
/// reserved name `default` is handled by the callers, not here.
pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(CswError::InvalidProfileName(
            "name must not be empty".to_string(),
        ));
    }
    if name.chars().count() > 64 {
        return Err(CswError::InvalidProfileName(
            "name must be 64 characters or fewer".to_string(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CswError::InvalidProfileName(
            "use letters, digits, '-' and '_' only (no spaces, slashes, dots or symbols)"
                .to_string(),
        ));
    }
    Ok(())
}

/// Sharing mode for a configuration component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SharingMode {
    /// Fully isolated: new profile gets its own independent copy.
    #[default]
    Isolate,
    /// Shared via symlink to the source profile's file/directory.
    Share,
    /// One-time copy from the source (diverges after creation).
    Copy,
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

    /// skills/ directory (CLI custom agent skills)
    #[serde(default)]
    pub cli_skills: SharingMode,

    /// history.jsonl (CLI command history file)
    #[serde(default)]
    pub cli_history: SharingMode,

    /// git-worktrees.json (worktree name → repo/branch mapping)
    #[serde(default)]
    pub desktop_worktrees: SharingMode,

    /// Source profile for shared components (default: "default")
    #[serde(default)]
    pub source: SharingSource,
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
    /// Fully isolated: every component gets its own per-profile copy and nothing
    /// is carried over from the source. This backs the "すべて分ける" create mode,
    /// so even the machine device id is isolated (no cross-account linkage).
    fn default() -> Self {
        Self {
            cli_settings: SharingMode::Isolate,
            cli_claude_md: SharingMode::Isolate,
            cli_project_memory: SharingMode::Isolate,
            cli_plugins: SharingMode::Isolate,
            cli_skills: SharingMode::Isolate,
            cli_history: SharingMode::Isolate,
            desktop_worktrees: SharingMode::Isolate,
            source: SharingSource::default(),
        }
    }
}

impl SharingConfig {
    // Some components can never be shared or copied, so they are deliberately NOT
    // fields of SharingConfig at all: there is no SharingMode to set, which makes the
    // illegal "share an account-keyed file" state unrepresentable for every caller
    // (GUI, CLI, or a hand-built config). The linker isolates them unconditionally
    // (see Linker::link_profile), so a mode is only ever a choice over the components
    // below — never over these:
    //   - config.json holds OAuth token caches and per-account state; sharing it
    //     would mix two logins into one file.
    //   - claude_desktop_config.json holds account-keyed permission gates and is
    //     rewritten via temp+rename on launch (which breaks any symlink), so it can
    //     be neither shared nor safely copied.
    //   - the device id (ant-did) is isolated so two accounts are not linked as one.
    //   - sessions/ is per-environment runtime state (pid, cwd), not shareable content.
    //   - the account login lives in the per-profile data dir, untouched by the linker.

    /// Preset for "会話とメモリも分ける" — separate accounts and conversations, but
    /// reuse the common setup. The CLI global rules (`CLAUDE.md`), `plugins/` and
    /// `skills/` are shared by symlink (the app only reads them and the user is
    /// their single writer, so a bidirectional write never breaks the link). The
    /// permission/hook `settings.json` and the worktree list are copied once at
    /// creation. Conversation history, project memory and command history stay
    /// isolated. Use case: split by purpose while keeping one rule set.
    pub fn share_settings_preset() -> Self {
        Self {
            cli_claude_md: SharingMode::Share,
            cli_plugins: SharingMode::Share,
            cli_skills: SharingMode::Share,
            cli_settings: SharingMode::Copy,
            desktop_worktrees: SharingMode::Copy,
            ..Self::default()
        }
    }

    /// Preset for "アカウントだけ分ける" — separate only the account (billing and
    /// resource usage); carry the whole working context across. On top of
    /// [`Self::share_settings_preset`] it also shares the per-project conversation
    /// history and auto-memory (`projects/`) and the prompt history
    /// (`history.jsonl`) by symlink — these are directories the app appends into
    /// plus an append-only file, so the links stay intact. Because every account
    /// belongs to the same user, sharing their own conversations is a continuity
    /// choice, not a leak. `sessions/` is deliberately NOT shared: it holds runtime
    /// session state (pid, cwd, start time), which is per-environment bookkeeping,
    /// not conversation content. The login, OAuth tokens, Desktop config files and
    /// device id also stay isolated.
    /// Use case: run research and development on separate billing accounts while
    /// keeping one continuous workspace.
    pub fn share_workspace_preset() -> Self {
        Self {
            cli_project_memory: SharingMode::Share,
            cli_history: SharingMode::Share,
            ..Self::share_settings_preset()
        }
    }

    /// Whether this environment shares nothing with any other: every tunable
    /// component is [`SharingMode::Isolate`] (the "すべて分ける" preset). Only such
    /// environments are safe to launch in additional concurrent windows, because
    /// they hold no symlink into another profile that two live instances could
    /// race on, and each instance writes only inside its own `--user-data-dir`.
    /// A single Share or Copy component makes this false. The always-isolated,
    /// non-tunable files (config.json, claude_desktop_config.json, sessions/,
    /// device id) are not fields here, so they never affect the result.
    pub fn is_fully_isolated(&self) -> bool {
        self.cli_settings == SharingMode::Isolate
            && self.cli_claude_md == SharingMode::Isolate
            && self.cli_project_memory == SharingMode::Isolate
            && self.cli_plugins == SharingMode::Isolate
            && self.cli_skills == SharingMode::Isolate
            && self.cli_history == SharingMode::Isolate
            && self.desktop_worktrees == SharingMode::Isolate
    }
}

/// Whether the user's existing Claude data is present at the standard locations.
///
/// Sharing ("会話とメモリも分ける") symlinks from these default dirs, so a root that is
/// missing or empty has nothing to share. The create flow uses this to gate the
/// share mode: both missing → only "すべて分ける" makes sense; one missing → that
/// side simply won't carry over and the user should be told.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DefaultRootsStatus {
    pub desktop_present: bool,
    pub cli_present: bool,
}

/// True if `dir` exists and holds at least one entry (i.e. real data lives there).
fn dir_has_content(dir: &std::path::Path) -> bool {
    fs::read_dir(dir)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
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

    /// Report whether the standard existing-Claude data dirs (Desktop and CLI)
    /// exist and are non-empty. Used by the create flow to decide whether the
    /// "share" mode can carry anything over.
    pub fn default_roots_status(&self) -> DefaultRootsStatus {
        DefaultRootsStatus {
            desktop_present: dir_has_content(&self.provider.claude_desktop_default_dir()),
            cli_present: dir_has_content(&self.provider.claude_cli_default_dir()),
        }
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
                    // No hardcoded glyph: each surface picks its own (the GUI renders
                    // an SVG monitor icon); a decorative emoji here would surface raw
                    // in `csw profile show default` (AGENTS.md: no emoji for decoration).
                    icon: String::new(),
                    color: "#9E9E9E".to_string(),
                    is_default: true,
                },
                isolation: IsolationConfig {
                    desktop_user_data_dir: self.provider.claude_desktop_default_dir(),
                    cli_config_dir: self.provider.claude_cli_default_dir(),
                },
                sharing: SharingConfig {
                    cli_settings: SharingMode::Share,
                    cli_claude_md: SharingMode::Share,
                    cli_project_memory: SharingMode::Share,
                    cli_plugins: SharingMode::Share,
                    cli_skills: SharingMode::Share,
                    cli_history: SharingMode::Share,
                    desktop_worktrees: SharingMode::Share,
                    source: SharingSource {
                        profile: "default".to_string(),
                    },
                },
            });
        }

        // Names come from user input or directory listings; validating here keeps
        // path traversal structurally impossible for every caller (SPECIFICATION
        // §5.A: paths are resolved from the environment name, never accepted raw).
        validate_profile_name(name)?;

        let profiles_dir = self.app_config.lock().unwrap().profiles_dir.clone();
        let profile_toml = profiles_dir.join(name).join("profile.toml");
        if !profile_toml.exists() {
            return Err(CswError::ProfileNotFound(name.to_string()));
        }

        config::load_profile(&profile_toml)
    }

    pub fn create_profile(
        &self,
        name: &str,
        sharing: SharingConfig,
        icon: Option<String>,
    ) -> Result<Profile> {
        validate_profile_name(name)?;
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
                icon: icon.unwrap_or_default(),
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

    /// Read-only isolation check of one profile's link points (csw doctor).
    /// The default profile has no links, so inspecting it is rejected.
    pub fn inspect_profile_isolation(&self, name: &str) -> Result<inspector::ProfileReport> {
        if name == "default" {
            return Err(CswError::Other(
                "The default environment (existing Claude) has no links to inspect".to_string(),
            ));
        }
        let profile = self.get_profile(name)?;
        let source_profile = self.get_profile(&profile.sharing.source.profile)?;
        let inspector = inspector::Inspector::new(self.provider.as_ref());
        Ok(inspector.inspect_profile(&profile, &source_profile))
    }

    /// Re-point share links that no longer resolve to their existing expected
    /// source (csw doctor --fix). Only symlinks are swapped; never real data.
    pub fn doctor_fix_links(&self, name: &str) -> Result<Vec<&'static str>> {
        if name == "default" {
            return Err(CswError::Other(
                "The default environment (existing Claude) has no links to fix".to_string(),
            ));
        }
        let profile = self.get_profile(name)?;
        let source_profile = self.get_profile(&profile.sharing.source.profile)?;
        let inspector = inspector::Inspector::new(self.provider.as_ref());
        inspector.fix_relinkable(&profile, &source_profile)
    }

    /// Read-only data map of one profile (sizes and link targets for the
    /// detail screen). The default profile is the user's real Claude data, so
    /// it is never aggregated.
    pub fn profile_data_map(&self, name: &str) -> Result<data_map::DataMap> {
        if name == "default" {
            return Err(CswError::Other(
                "The default environment (existing Claude) is not aggregated".to_string(),
            ));
        }
        let profile = self.get_profile(name)?;
        Ok(data_map::build_data_map(&profile))
    }

    pub fn clone_profile(&self, source_name: &str, target_name: &str) -> Result<Profile> {
        validate_profile_name(target_name)?;
        if target_name == "default" {
            return Err(CswError::ProfileAlreadyExists("default".to_string()));
        }
        // The default profile's isolation dirs point at the user's real Claude
        // data directories; cloning it would bulk-read them, which docs/PRIVACY.md
        // promises never happens. Reject here so no UI path can reach it.
        if source_name == "default" {
            return Err(CswError::Other(
                "The default environment (existing Claude) cannot be duplicated".to_string(),
            ));
        }
        if source_name == target_name {
            return Err(CswError::Other(
                "Source and target profile names must be different".to_string(),
            ));
        }

        let source_profile = self.get_profile(source_name)?;

        let profiles_dir = self.app_config.lock().unwrap().profiles_dir.clone();
        let target_dir = profiles_dir.join(target_name);
        let target_toml = target_dir.join("profile.toml");

        if target_toml.exists() {
            return Err(CswError::ProfileAlreadyExists(target_name.to_string()));
        }

        let target_profile = Profile {
            profile: ProfileMeta {
                name: target_name.to_string(),
                icon: source_profile.profile.icon.clone(),
                color: source_profile.profile.color.clone(),
                is_default: false,
            },
            isolation: IsolationConfig {
                desktop_user_data_dir: target_dir.join("desktop-data"),
                cli_config_dir: target_dir.join("cli-data"),
            },
            sharing: source_profile.sharing.clone(),
        };

        // Create target directory structure
        fs::create_dir_all(&target_dir)?;
        fs::create_dir_all(&target_profile.isolation.desktop_user_data_dir)?;
        fs::create_dir_all(&target_profile.isolation.cli_config_dir)?;

        // Deep copy files/folders of copy/isolate modes from the source to target.
        // For shared modes, the linker will build proper symlinks.
        let source_desktop = &source_profile.isolation.desktop_user_data_dir;
        let source_cli = &source_profile.isolation.cli_config_dir;

        let linker = linker::Linker::new(self.provider.as_ref());

        // 1. Copy desktop-data and cli-data files that are not symlinks
        // If the source profile is "default" or other, we clone its current physical configurations.
        if source_desktop.exists() {
            for entry in fs::read_dir(source_desktop)? {
                let entry = entry?;
                let path = entry.path();
                let filename = entry.file_name();
                let target_path = target_profile
                    .isolation
                    .desktop_user_data_dir
                    .join(&filename);

                // Skip files managed by linker (will link/copy them via apply_link in linker)
                let name_str = filename.to_string_lossy();
                if name_str == "claude_desktop_config.json"
                    || name_str == "git-worktrees.json"
                    || name_str == "ant-did"
                    || name_str == "config.json"
                {
                    continue;
                }

                // Copy other caches, sessions, credentials backup
                if !self.provider.is_symlink(&path) {
                    if path.is_dir() {
                        self.copy_dir_recursive(&path, &target_path)?;
                    } else {
                        fs::copy(&path, &target_path)?;
                    }
                }
            }
        }

        if source_cli.exists() {
            for entry in fs::read_dir(source_cli)? {
                let entry = entry?;
                let path = entry.path();
                let filename = entry.file_name();
                let target_path = target_profile.isolation.cli_config_dir.join(&filename);

                let name_str = filename.to_string_lossy();
                if name_str == "settings.json"
                    || name_str == "CLAUDE.md"
                    || name_str == "projects"
                    || name_str == "plugins"
                    || name_str == "skills"
                    || name_str == "sessions"
                    || name_str == "history.jsonl"
                {
                    continue;
                }

                if !self.provider.is_symlink(&path) {
                    if path.is_dir() {
                        self.copy_dir_recursive(&path, &target_path)?;
                    } else {
                        fs::copy(&path, &target_path)?;
                    }
                }
            }
        }

        // Link/copy managed components according to target_profile's SharingConfig
        linker.link_profile(&target_profile, &source_profile)?;

        // Save profile metadata
        config::save_profile(&target_profile, &target_toml)?;

        Ok(target_profile)
    }

    fn copy_dir_recursive(&self, src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let target_path = dst.join(entry.file_name());
            if ty.is_dir() {
                self.copy_dir_recursive(&entry.path(), &target_path)?;
            } else {
                fs::copy(entry.path(), &target_path)?;
            }
        }
        Ok(())
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

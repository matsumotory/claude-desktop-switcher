//! Volatile per-environment state (`profiles/<name>/state.toml`).
//!
//! Kept separate from `profile.toml` on purpose: `profile.toml` is the safety
//! declaration (sharing modes, paths) and must never be rewritten by routine
//! launches. This file only carries informational stamps, so losing or
//! regenerating it is always harmless.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileState {
    /// RFC 3339 time of the last CSW-initiated launch of this environment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_launched_at: Option<String>,
}

/// Read the state file. A missing file is an empty state. An unparsable file
/// is also treated as empty on purpose: the stamps are informational only and
/// must never block profile operations; the file is rewritten on next launch.
pub fn load_state(profile_dir: &Path) -> Result<ProfileState> {
    let path = profile_dir.join("state.toml");
    if !path.exists() {
        return Ok(ProfileState::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&content).unwrap_or_default())
}

/// Stamp the last launch time (now), written atomically (temp file + rename).
pub fn record_last_launch(profile_dir: &Path) -> Result<()> {
    let mut state = load_state(profile_dir)?;
    state.last_launched_at = Some(now_rfc3339());
    let content = toml::to_string_pretty(&state)?;
    let tmp = profile_dir.join("state.toml.tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, profile_dir.join("state.toml"))?;
    Ok(())
}

/// Current time as RFC 3339 (UTC, second precision) for creation and launch
/// stamps.
pub(crate) fn now_rfc3339() -> String {
    humantime::format_rfc3339_seconds(std::time::SystemTime::now()).to_string()
}

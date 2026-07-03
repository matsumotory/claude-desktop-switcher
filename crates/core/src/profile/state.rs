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
    /// RFC 3339 time of the first CSW-initiated launch. Written once and never
    /// moved; the sign-in signpost card compares it with the last launch to
    /// tell "launched exactly once" apart from routine use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_launched_at: Option<String>,
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
/// The first launch is stamped once alongside it and never moves. A state file
/// that already has a last launch but no first launch comes from a build that
/// predates the first-launch stamp: such an environment has been launched (and
/// signed in) before, so the first launch is deliberately NOT backfilled;
/// otherwise the sign-in signpost would show for an already signed-in user.
pub fn record_last_launch(profile_dir: &Path) -> Result<()> {
    let mut state = load_state(profile_dir)?;
    let now = now_rfc3339();
    if state.first_launched_at.is_none() && state.last_launched_at.is_none() {
        state.first_launched_at = Some(now.clone());
    }
    state.last_launched_at = Some(now);
    let content = toml::to_string_pretty(&state)?;
    let tmp = profile_dir.join("state.toml.tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, profile_dir.join("state.toml"))?;
    Ok(())
}

/// Current time as RFC 3339 (UTC, millisecond precision) for creation and
/// launch stamps. Millisecond precision keeps two rapid launches from writing
/// an identical first == last pair, which would misread as "launched once".
pub(crate) fn now_rfc3339() -> String {
    humantime::format_rfc3339_millis(std::time::SystemTime::now()).to_string()
}

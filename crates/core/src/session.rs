//! App-level session state (`~/.context-switcher-claude/session.toml`).
//!
//! Holds the set of environments that were open together before the last
//! all-quit, so the settings window can offer to reopen them (for example after
//! a Claude Desktop update, which requires quitting every instance). Kept apart
//! from per-environment `state.toml` because it is app-wide, and written only on
//! the all-quit transition, so writes are rare.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

const FILE: &str = "session.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SessionState {
    /// Environment names open together before the last all-quit.
    #[serde(default)]
    reopen_set: Vec<String>,
}

/// Read the recorded reopen set. A missing file is empty; an unparsable file is
/// also treated as empty on purpose (the set is a convenience and must never
/// block anything). `app_data_dir` is CSW's own data root.
pub fn load_reopen_set(app_data_dir: &Path) -> Vec<String> {
    let path = app_data_dir.join(FILE);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    toml::from_str::<SessionState>(&content)
        .map(|s| s.reopen_set)
        .unwrap_or_default()
}

/// Save the reopen set, written atomically (temp file + rename).
pub fn save_reopen_set(app_data_dir: &Path, set: &[String]) -> Result<()> {
    std::fs::create_dir_all(app_data_dir)?;
    let state = SessionState {
        reopen_set: set.to_vec(),
    };
    let content = toml::to_string_pretty(&state)?;
    let tmp = app_data_dir.join("session.toml.tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, app_data_dir.join(FILE))?;
    Ok(())
}

/// Clear the recorded reopen set (the banner's dismiss, or after reopening).
pub fn clear_reopen_set(app_data_dir: &Path) -> Result<()> {
    let path = app_data_dir.join(FILE);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn reopen_set_round_trips() {
        let dir = tempdir().unwrap();
        assert!(load_reopen_set(dir.path()).is_empty());

        save_reopen_set(dir.path(), &["default".into(), "Work".into()]).unwrap();
        assert_eq!(load_reopen_set(dir.path()), vec!["default", "Work"]);

        clear_reopen_set(dir.path()).unwrap();
        assert!(load_reopen_set(dir.path()).is_empty());
        // Clearing an already-absent file is fine.
        clear_reopen_set(dir.path()).unwrap();
    }

    #[test]
    fn corrupt_session_file_reads_as_empty() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join(FILE), "this is not toml = = =").unwrap();
        assert!(load_reopen_set(dir.path()).is_empty());
    }
}

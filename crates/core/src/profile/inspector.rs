//! Read-only isolation inspector: compares a profile's declared sharing modes
//! (profile.toml) with what is actually on disk at the linker-managed link
//! points, and reports per-item health.
//!
//! Scan scope is deliberately narrow (docs/PRIVACY.md): only the fixed link
//! points from `LINK_ITEMS` are examined, via symlink metadata and existence
//! checks. File contents are never read, and environment data directories are
//! never listed. The existing Claude side is only ever `stat`ed to see whether
//! a shared source exists.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{CswError, Result};
use crate::platform::PlatformProvider;
use crate::profile::linker::{LINK_ITEMS, item_mode, item_path};
use crate::profile::{Profile, SharingMode};
use crate::switcher::desktop_dir_running;

/// Health of one linker-managed link point.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ItemHealth {
    /// Share: the symlink resolves to the expected source, which exists.
    SharedOk { target: String },
    /// Share: the symlink points at the expected source, but it no longer exists.
    SourceMissing { expected_source: String },
    /// Share: the symlink points somewhere other than the expected source.
    /// `fixable` is true when the expected source exists, so `--fix` can
    /// re-point the link (a symlink swap; no real data is touched).
    WrongTarget {
        expected_source: String,
        actual_target: String,
        fixable: bool,
    },
    /// Share: a real file/dir sits where a symlink should be. The link
    /// drifted into a real copy (e.g. an app rewrote it via temp+rename).
    /// Never auto-fixed: replacing the real file could lose data.
    Materialized,
    /// Share: nothing at the link point although the shared source exists.
    MissingLink { expected_source: String },
    /// Share declared, but the shared source does not exist in the source
    /// environment; the item stands alone locally. This matches how links are
    /// created when there is nothing to share yet, so it is not an issue.
    SourceAbsent,
    /// Copy/Isolate: no symlink (a real file/dir, or nothing) — as declared.
    IsolatedOk,
    /// Copy/Isolate: a symlink was found where the item must be independent.
    UnexpectedLink { actual_target: String },
}

impl ItemHealth {
    pub fn is_issue(&self) -> bool {
        !matches!(
            self,
            ItemHealth::SharedOk { .. } | ItemHealth::IsolatedOk | ItemHealth::SourceAbsent
        )
    }
}

/// One item's check result.
#[derive(Debug, Clone, Serialize)]
pub struct ItemReport {
    pub key: &'static str,
    pub mode: SharingMode,
    pub health: ItemHealth,
    pub is_issue: bool,
}

/// A profile's full isolation check result.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileReport {
    pub profile: String,
    pub items: Vec<ItemReport>,
    pub issue_count: usize,
    /// True when a Claude Desktop App instance is currently running on this
    /// environment's data dir. CLI sessions are not detected: that would need
    /// reading sessions/ contents, which is outside the scan scope. Checks race
    /// the app's own temp+rename writes, so callers should suggest quitting
    /// Claude and re-checking before acting on issues.
    pub running: bool,
}

/// Read-only comparison of declared sharing modes against the disk state.
pub struct Inspector<'a> {
    provider: &'a dyn PlatformProvider,
}

impl<'a> Inspector<'a> {
    pub fn new(provider: &'a dyn PlatformProvider) -> Self {
        Self { provider }
    }

    /// Inspect every linker-managed link point of `profile` against the
    /// sharing source `source_profile` (normally the existing Claude).
    pub fn inspect_profile(&self, profile: &Profile, source_profile: &Profile) -> ProfileReport {
        let items: Vec<ItemReport> = LINK_ITEMS
            .iter()
            .map(|item| {
                let mode = item_mode(profile, item);
                let health = self.check_item(
                    &item_path(profile, item),
                    &item_path(source_profile, item),
                    &mode,
                );
                ItemReport {
                    key: item.key,
                    is_issue: health.is_issue(),
                    mode,
                    health,
                }
            })
            .collect();

        let issue_count = items.iter().filter(|i| i.is_issue).count();
        let running_args = self.provider.running_desktop_args().unwrap_or_default();
        let running = desktop_dir_running(
            &profile.isolation.desktop_user_data_dir,
            &running_args,
            &self.provider.claude_desktop_default_dir(),
        );

        ProfileReport {
            profile: profile.profile.name.clone(),
            items,
            issue_count,
            running,
        }
    }

    /// Re-point share links that do not correctly resolve to an existing
    /// expected source (`WrongTarget` with `fixable: true`). Only symlinks are
    /// swapped; real files and directories are never touched, and operating
    /// inside the real default Claude dirs is refused. Returns the fixed keys.
    pub fn fix_relinkable(
        &self,
        profile: &Profile,
        source_profile: &Profile,
    ) -> Result<Vec<&'static str>> {
        let report = self.inspect_profile(profile, source_profile);
        let mut fixed = Vec::new();

        for item_report in &report.items {
            if !matches!(
                item_report.health,
                ItemHealth::WrongTarget { fixable: true, .. }
            ) {
                continue;
            }
            let item = LINK_ITEMS
                .iter()
                .find(|i| i.key == item_report.key)
                .expect("report keys come from LINK_ITEMS");
            let path = item_path(profile, item);
            self.assert_outside_default_roots(&path)?;
            self.provider.remove_symlink(&path)?;
            self.provider
                .create_symlink(&item_path(source_profile, item), &path)?;
            fixed.push(item_report.key);
        }

        Ok(fixed)
    }

    /// Classify one link point. Uses only symlink metadata and existence
    /// checks; never opens file contents or lists directories.
    fn check_item(&self, path: &Path, expected_source: &Path, mode: &SharingMode) -> ItemHealth {
        let is_link = self.provider.is_symlink(path);

        match mode {
            SharingMode::Share => {
                if is_link {
                    let actual: PathBuf = match fs::read_link(path) {
                        Ok(t) => t,
                        Err(_) => PathBuf::new(),
                    };
                    if actual == expected_source {
                        if expected_source.exists() {
                            ItemHealth::SharedOk {
                                target: display(&actual),
                            }
                        } else {
                            ItemHealth::SourceMissing {
                                expected_source: display(expected_source),
                            }
                        }
                    } else {
                        ItemHealth::WrongTarget {
                            expected_source: display(expected_source),
                            actual_target: display(&actual),
                            fixable: expected_source.exists(),
                        }
                    }
                } else if path.exists() {
                    if expected_source.exists() {
                        ItemHealth::Materialized
                    } else {
                        ItemHealth::SourceAbsent
                    }
                } else if expected_source.exists() {
                    ItemHealth::MissingLink {
                        expected_source: display(expected_source),
                    }
                } else {
                    ItemHealth::SourceAbsent
                }
            }
            SharingMode::Copy | SharingMode::Isolate => {
                if is_link {
                    let actual = fs::read_link(path).unwrap_or_default();
                    ItemHealth::UnexpectedLink {
                        actual_target: display(&actual),
                    }
                } else {
                    ItemHealth::IsolatedOk
                }
            }
        }
    }

    /// Defense in depth, mirroring the linker's guard: the fixer must never
    /// operate on paths inside the user's real default Claude data dirs.
    fn assert_outside_default_roots(&self, path: &Path) -> Result<()> {
        let defaults = [
            self.provider.claude_desktop_default_dir(),
            self.provider.claude_cli_default_dir(),
        ];
        if defaults
            .iter()
            .any(|root| path == root.as_path() || path.starts_with(root))
        {
            return Err(CswError::RefusedDefaultDataDeletion(
                path.display().to_string(),
            ));
        }
        Ok(())
    }
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

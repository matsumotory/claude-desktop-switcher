//! Read-only data map of an environment: per-item and per-folder approximate
//! sizes and last-modified times, for the detail screen's
//! 「この環境のデータの場所」 breakdown.
//!
//! Aggregation rules (docs/PRIVACY.md):
//! - Only file names and stat metadata (size, mtime) are read; file contents
//!   are never opened.
//! - Symlinks are never followed (`symlink_metadata` throughout), so shared
//!   originals on the existing Claude side are never walked and contribute
//!   nothing to the totals.
//! - The sign-in item (config.json) is reported with no size or mtime at all:
//!   minimal exposure.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::Serialize;

use crate::profile::linker::{LINK_ITEMS, item_mode, item_path};
use crate::profile::{Profile, SharingMode};

/// One link point's presence in the data map.
#[derive(Debug, Clone, Serialize)]
pub struct ItemData {
    pub key: &'static str,
    pub mode: SharingMode,
    /// Share items: where the link points (displayed, never walked).
    pub link_target: Option<String>,
    /// Non-share items: approximate bytes occupied (None for share items and
    /// for the sign-in item).
    pub size_bytes: Option<u64>,
    /// Unix seconds of the newest file under the item (None where size is None).
    pub modified_epoch: Option<u64>,
    pub exists: bool,
}

/// A profile's full data map.
#[derive(Debug, Clone, Serialize)]
pub struct DataMap {
    pub profile: String,
    pub desktop_dir: String,
    pub cli_dir: String,
    pub desktop_size_bytes: u64,
    pub cli_size_bytes: u64,
    /// Space this environment occupies by itself. Symlinks count as zero, so
    /// shared originals are excluded by construction.
    pub total_size_bytes: u64,
    pub items: Vec<ItemData>,
}

/// Build the data map for `profile`. Read-only; never follows symlinks.
pub fn build_data_map(profile: &Profile) -> DataMap {
    let items = LINK_ITEMS
        .iter()
        .map(|item| {
            let mode = item_mode(profile, item);
            let path = item_path(profile, item);
            let meta = fs::symlink_metadata(&path).ok();
            let exists = meta.is_some();
            let is_link = meta.as_ref().is_some_and(|m| m.file_type().is_symlink());

            // Minimal exposure for the sign-in item: presence only.
            if item.key == "desktop_app_config" {
                return ItemData {
                    key: item.key,
                    mode,
                    link_target: None,
                    size_bytes: None,
                    modified_epoch: None,
                    exists,
                };
            }

            if is_link {
                // A link (shared as declared, or drifted into one): show the
                // target, never walk it. A share item that materialized into a
                // real file falls through and gets sized like any local data,
                // so the per-item rows always add up with the folder totals.
                return ItemData {
                    key: item.key,
                    mode,
                    link_target: fs::read_link(&path).ok().map(|t| t.display().to_string()),
                    size_bytes: None,
                    modified_epoch: None,
                    exists,
                };
            }

            let (size, modified) = walk_size(&path);
            ItemData {
                key: item.key,
                mode,
                link_target: None,
                size_bytes: exists.then_some(size),
                modified_epoch: modified,
                exists,
            }
        })
        .collect();

    let (desktop_size_bytes, _) = walk_size(&profile.isolation.desktop_user_data_dir);
    let (cli_size_bytes, _) = walk_size(&profile.isolation.cli_config_dir);

    DataMap {
        profile: profile.profile.name.clone(),
        desktop_dir: profile
            .isolation
            .desktop_user_data_dir
            .display()
            .to_string(),
        cli_dir: profile.isolation.cli_config_dir.display().to_string(),
        desktop_size_bytes,
        cli_size_bytes,
        total_size_bytes: desktop_size_bytes + cli_size_bytes,
        items,
    }
}

/// Approximate bytes and newest mtime under `path`, never following symlinks:
/// a symlink (to a file or a directory) counts as zero bytes, and its target
/// is never visited. Only stat metadata is read; contents are never opened.
fn walk_size(path: &Path) -> (u64, Option<u64>) {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return (0, None);
    };
    if meta.file_type().is_symlink() {
        return (0, None);
    }
    if meta.is_file() {
        return (meta.len(), mtime_epoch(&meta));
    }
    let mut total = 0u64;
    let mut newest: Option<u64> = None;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let (size, modified) = walk_size(&entry.path());
            total += size;
            newest = match (newest, modified) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
        }
    }
    (total, newest)
}

fn mtime_epoch(meta: &fs::Metadata) -> Option<u64> {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

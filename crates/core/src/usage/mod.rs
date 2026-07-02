//! Per-environment usage display (statusline integration).
//!
//! Claude Code passes a JSON payload to the configured status-line command on
//! every assistant response; for Pro/Max subscribers it includes `rate_limits`
//! (the five-hour session window and the seven-day week window, each with
//! `used_percentage` and `resets_at`). CSW opts an environment in by pointing
//! that environment's `cli-data/settings.json` `statusLine` at a generated
//! shell script which saves the payload under CSW's own data dir and still
//! prints a status line (chaining any pre-existing command). The GUI then only
//! ever reads local files. See docs/SPECIFICATION.md §5.A and
//! https://code.claude.com/docs/en/statusline for the payload contract.
//!
//! Enabled/disabled is never tracked as a separate flag: it is derived from
//! whether settings.json's statusLine command points at the CSW script, so the
//! stored state cannot drift from reality.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CswError, Result};
use crate::profile::{Profile, SharingMode};
use serde::Serialize;
use serde_json::{Map, Value};

/// Locations of everything the usage feature touches. All generated artifacts
/// live under CSW's own data dir; the only file written outside it is the
/// opted-in environment's settings.json.
pub struct UsagePaths {
    app_data_dir: PathBuf,
    cli_default_dir: PathBuf,
}

impl UsagePaths {
    pub fn new(app_data_dir: PathBuf, cli_default_dir: PathBuf) -> Self {
        Self {
            app_data_dir,
            cli_default_dir,
        }
    }

    pub fn for_provider(provider: &dyn crate::platform::PlatformProvider) -> Self {
        Self::new(provider.app_data_dir(), provider.claude_cli_default_dir())
    }

    /// The generated status-line script for an environment.
    pub fn script_path(&self, name: &str) -> PathBuf {
        self.app_data_dir
            .join("statusline")
            .join(format!("{name}.sh"))
    }

    /// Where a pre-existing statusLine setting is parked while CSW's script is
    /// active, so disabling can restore it verbatim.
    pub fn backup_path(&self, name: &str) -> PathBuf {
        self.app_data_dir
            .join("statusline")
            .join(format!("{name}.original.json"))
    }

    /// The saved status-line payload for an environment.
    pub fn usage_file_path(&self, name: &str) -> PathBuf {
        self.app_data_dir.join("usage").join(format!("{name}.json"))
    }

    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
    }
}

/// One rate-limit window, as Claude Code reports it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UsageWindow {
    /// 0-100.
    pub used_percentage: f64,
    /// Unix epoch seconds when the window resets.
    pub resets_at: u64,
}

/// The usage values CSW last captured for an environment. Windows whose
/// `resets_at` has passed are dropped rather than shown stale.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UsageSnapshot {
    pub five_hour: Option<UsageWindow>,
    pub seven_day: Option<UsageWindow>,
    /// Unix epoch seconds of the capture (the saved file's mtime).
    pub captured_at: u64,
    /// Seconds elapsed between the capture and `now`.
    pub age_seconds: u64,
}

fn settings_path(profile: &Profile) -> PathBuf {
    profile.isolation.cli_config_dir.join("settings.json")
}

/// settings.json as a JSON object; a missing file is an empty object. Content
/// we cannot parse is a hard error so the file is never rewritten from a
/// misread state.
fn read_settings_object(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| {
        CswError::UsageSettingsUnreadable(path.display().to_string(), e.to_string())
    })?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(CswError::UsageSettingsUnreadable(
            path.display().to_string(),
            "top level is not a JSON object".to_string(),
        )),
    }
}

/// Write settings.json via temp + rename in the same directory, so a crash
/// never leaves a half-written settings file for Claude Code to read.
fn write_settings_object(path: &Path, map: &Map<String, Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".csw-tmp");
    let tmp = PathBuf::from(tmp);
    let mut body = serde_json::to_string_pretty(&Value::Object(map.clone()))?;
    body.push('\n');
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn statusline_command(map: &Map<String, Value>) -> Option<&str> {
    map.get("statusLine")?.get("command")?.as_str()
}

/// Single-quote a string for safe literal embedding in a POSIX shell script
/// (same escaping the `csw env` generator uses).
fn sh_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// The command string of a previously parked statusLine setting, if any.
fn read_parked_command(paths: &UsagePaths, name: &str) -> Result<Option<String>> {
    let backup = paths.backup_path(name);
    if !backup.exists() {
        return Ok(None);
    }
    let parked: Value = serde_json::from_str(&fs::read_to_string(&backup)?)?;
    Ok(parked
        .get("command")
        .and_then(Value::as_str)
        .map(str::to_string))
}

/// Whether usage display is on for this environment, derived from whether its
/// settings.json statusLine command points at the CSW-generated script.
pub fn is_enabled(paths: &UsagePaths, profile: &Profile) -> bool {
    let Ok(map) = read_settings_object(&settings_path(profile)) else {
        return false;
    };
    let script = paths.script_path(&profile.profile.name);
    statusline_command(&map) == Some(script.to_string_lossy().as_ref())
}

/// Whether the environment may be opted in at all. Environments that share
/// settings.json with the existing Claude (a symlink to the same file) are
/// refused: writing through the link would change the existing Claude too.
/// The default environment itself is allowed (it opts in its own real file).
pub fn can_enable(profile: &Profile) -> Result<()> {
    if !profile.profile.is_default && profile.sharing.cli_settings == SharingMode::Share {
        return Err(CswError::UsageSettingsShared(profile.profile.name.clone()));
    }
    Ok(())
}

/// Opt an environment in: generate the script, park any pre-existing
/// statusLine setting, and point settings.json at the script. Idempotent: a
/// second enable regenerates the script from what was parked the first time
/// and never parks CSW's own script as if it were the user's.
pub fn enable(paths: &UsagePaths, profile: &Profile) -> Result<()> {
    can_enable(profile)?;
    let name = &profile.profile.name;
    let spath = settings_path(profile);
    let mut map = read_settings_object(&spath)?;
    let script = paths.script_path(name);
    let script_str = script.to_string_lossy().to_string();

    let chained_command = match map.get("statusLine").cloned() {
        Some(v) if v.get("command").and_then(Value::as_str) == Some(script_str.as_str()) => {
            // Already ours: keep whatever was parked originally.
            read_parked_command(paths, name)?
        }
        Some(v) => {
            // Park the user's setting verbatim so disable can restore it.
            let backup = paths.backup_path(name);
            fs::create_dir_all(backup.parent().expect("backup path has a parent"))?;
            fs::write(&backup, serde_json::to_string_pretty(&v)?)?;
            v.get("command").and_then(Value::as_str).map(str::to_string)
        }
        None => None,
    };

    write_script(paths, name, chained_command.as_deref())?;

    let mut status_line = Map::new();
    status_line.insert("type".to_string(), Value::String("command".to_string()));
    status_line.insert("command".to_string(), Value::String(script_str));
    map.insert("statusLine".to_string(), Value::Object(status_line));
    write_settings_object(&spath, &map)
}

/// Opt an environment out: restore the parked statusLine (or drop the key if
/// none was parked) and remove the script, the parked copy and the saved
/// payload. If the user has meanwhile pointed statusLine somewhere else by
/// hand, that setting is theirs and settings.json is left exactly as it is.
pub fn disable(paths: &UsagePaths, profile: &Profile) -> Result<()> {
    let name = &profile.profile.name;
    let spath = settings_path(profile);
    if spath.exists() {
        let mut map = read_settings_object(&spath)?;
        let script_str = paths.script_path(name).to_string_lossy().to_string();
        if statusline_command(&map) == Some(script_str.as_str()) {
            let backup = paths.backup_path(name);
            if backup.exists() {
                let parked: Value = serde_json::from_str(&fs::read_to_string(&backup)?)?;
                map.insert("statusLine".to_string(), parked);
            } else {
                map.remove("statusLine");
            }
            write_settings_object(&spath, &map)?;
        }
    }
    remove_artifacts(paths, name)
}

/// Remove every artifact for an environment (also used when the environment is
/// deleted). Missing files are fine.
pub fn remove_artifacts(paths: &UsagePaths, name: &str) -> Result<()> {
    for path in [
        paths.script_path(name),
        paths.backup_path(name),
        paths.usage_file_path(name),
    ] {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// Read the last captured payload for an environment. Returns None when there
/// is no capture, it cannot be parsed, it has no rate_limits, or every window
/// has already reset (a stale value must not look fresh).
pub fn read_snapshot(paths: &UsagePaths, name: &str, now_epoch: u64) -> Option<UsageSnapshot> {
    let path = paths.usage_file_path(name);
    let captured_at = fs::metadata(&path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).ok()?).ok()?;
    let rate_limits = value.get("rate_limits")?;

    let window = |key: &str| -> Option<UsageWindow> {
        let w = rate_limits.get(key)?;
        let used_percentage = w.get("used_percentage")?.as_f64()?;
        let resets_at = w
            .get("resets_at")
            .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))?;
        (resets_at > now_epoch).then_some(UsageWindow {
            used_percentage,
            resets_at,
        })
    };

    let five_hour = window("five_hour");
    let seven_day = window("seven_day");
    if five_hour.is_none() && seven_day.is_none() {
        return None;
    }
    Some(UsageSnapshot {
        five_hour,
        seven_day,
        captured_at,
        age_seconds: now_epoch.saturating_sub(captured_at),
    })
}

/// Generate the status-line script. The save target is derived from
/// CLAUDE_CONFIG_DIR at run time (not baked in), so even if the settings.json
/// carrying this statusLine ever reaches another environment, each run still
/// saves under the environment it actually ran in, or not at all.
fn write_script(paths: &UsagePaths, name: &str, chained_command: Option<&str>) -> Result<()> {
    let script = paths.script_path(name);
    fs::create_dir_all(script.parent().expect("script path has a parent"))?;

    let mut body = String::new();
    body.push_str("#!/bin/sh\n");
    body.push_str(
        "# Generated by Claude Desktop Switcher (usage display). Managed automatically;\n\
         # edits are overwritten. Reads the status-line JSON from Claude Code on stdin,\n\
         # saves it so CSW can show usage bars, then prints a status line.\n",
    );
    body.push_str(&format!(
        "csw_data={}\n",
        sh_single_quote(&paths.app_data_dir.to_string_lossy())
    ));
    body.push_str(&format!(
        "cli_default={}\n",
        sh_single_quote(&paths.cli_default_dir.to_string_lossy())
    ));
    body.push_str(
        r#"json=$(cat)

# Save under the environment this run actually belongs to, derived from
# CLAUDE_CONFIG_DIR. Unknown locations are not CSW's to record.
cfg="${CLAUDE_CONFIG_DIR:-$cli_default}"
name=''
case "$cfg" in
  "$cli_default") name='default' ;;
  "$csw_data/profiles/"*"/cli-data")
    name="${cfg#"$csw_data/profiles/"}"
    name="${name%/cli-data}"
    ;;
esac
case "$name" in
  ''|*/*) : ;;
  *)
    mkdir -p "$csw_data/usage"
    tmp="$csw_data/usage/.$name.json.tmp"
    printf '%s' "$json" > "$tmp" && mv -f "$tmp" "$csw_data/usage/$name.json"
    ;;
esac
"#,
    );

    match chained_command {
        Some(cmd) => {
            // Keep the user's own status line: hand it the same stdin and let
            // its output through. Claude Code runs the command via the shell,
            // so embedding it as script lines preserves its semantics.
            body.push_str("\nprintf '%s' \"$json\" | {\n");
            body.push_str(cmd);
            body.push_str("\n}\n");
        }
        None => {
            body.push_str(
                r#"
# Default line: concise usage percentages (empty when rate_limits is absent,
# e.g. API-key sessions). plutil ships with macOS; no jq dependency. On a
# missing key some plutil versions print the error to stdout instead of
# stderr, so anything non-numeric is discarded rather than fed to printf
# (which would render a fake 0%).
five=$(printf '%s' "$json" | plutil -extract rate_limits.five_hour.used_percentage raw -o - - 2>/dev/null)
case "$five" in ''|*[!0-9.]*) five='' ;; esac
week=$(printf '%s' "$json" | plutil -extract rate_limits.seven_day.used_percentage raw -o - - 2>/dev/null)
case "$week" in ''|*[!0-9.]*) week='' ;; esac
line=''
[ -n "$five" ] && line="5h $(LC_ALL=C printf '%.0f' "$five")%"
if [ -n "$week" ]; then
  [ -n "$line" ] && line="$line / "
  line="${line}week $(LC_ALL=C printf '%.0f' "$week")%"
fi
printf '%s\n' "$line"
"#,
            );
        }
    }

    fs::write(&script, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;

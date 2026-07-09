pub mod desktop;
pub mod shell;

use std::path::Path;
use std::sync::Arc;

use crate::error::{CswError, Result};
use crate::platform::PlatformProvider;
use crate::profile::ProfileManager;

/// Coordinates switching the active profile.
///
/// Account/session isolation is achieved purely by per-profile directories:
/// the Desktop app is launched with `--user-data-dir` and the CLI runs with
/// `CLAUDE_CONFIG_DIR` pointing at the profile's own directories. Each profile
/// keeps its own login inside its own directory, so CSW never reads, writes, or
/// deletes Keychain credentials.
///
/// This was verified empirically: a fresh `CLAUDE_CONFIG_DIR` reports
/// `loggedIn: false` independently of the default profile, i.e. Claude Code
/// scopes its credentials per config directory. Likewise the Desktop app keeps
/// its session inside its `--user-data-dir`. Copying secrets out of the OS
/// Keychain into files would only weaken security without being necessary, so
/// that mechanism has been removed.
pub struct ContextSwitcher {
    provider: Arc<dyn PlatformProvider>,
    profile_manager: Arc<ProfileManager>,
}

impl ContextSwitcher {
    pub fn new(provider: Arc<dyn PlatformProvider>, profile_manager: Arc<ProfileManager>) -> Self {
        Self {
            provider,
            profile_manager,
        }
    }

    /// Switch the active profile.
    ///
    /// Isolation is handled at launch time via per-profile directories
    /// (`--user-data-dir` / `CLAUDE_CONFIG_DIR`), so this only validates the
    /// target exists and records it as the active profile. No Keychain or
    /// credential files are touched.
    ///
    /// Switching is refused while Claude Desktop is running: a live instance can
    /// write cached state back into the currently active profile's directory and
    /// race with shared (symlinked) files during the switch, so the user must
    /// quit Claude Desktop first.
    pub fn switch_to(&self, profile_name: &str) -> Result<()> {
        // Validate the profile exists before recording it as active.
        let _ = self.profile_manager.get_profile(profile_name)?;

        // Refuse to switch while Claude Desktop is running to avoid cache
        // write-back and symlink data races.
        if self.provider.is_claude_desktop_running()? {
            return Err(CswError::DesktopRunning);
        }

        self.profile_manager.switch_to(profile_name)?;
        Ok(())
    }
}

/// Whether a Claude Desktop instance for `target_dir` is currently running.
///
/// "In use" (利用中) is a live runtime state. Because fully-isolated environments
/// can now run side by side, it can no longer be derived from "the active
/// environment while Claude runs" (that would mislabel which environment is up).
/// Instead it is read from the live processes: `running_args` are the argument
/// strings of the running Claude *main* processes (helpers are filtered out by the
/// platform layer). An environment is in use when a main process was launched with
/// its `--user-data-dir`. A main process launched without the flag (a plain
/// Finder/Dock launch) uses the default data dir, so it counts as the existing
/// Claude (`default`).
pub fn desktop_dir_running(target_dir: &Path, running_args: &[String], default_dir: &Path) -> bool {
    let needle = format!("--user-data-dir={}", target_dir.display());
    if running_args.iter().any(|a| flag_value_present(a, &needle)) {
        return true;
    }
    // No explicit --user-data-dir means the default location.
    target_dir == default_dir && running_args.iter().any(|a| !a.contains("--user-data-dir="))
}

/// Resolve one Claude main-process args line to the environment it runs.
/// A `--user-data-dir` value matching an environment's data directory names
/// that environment; a line without the flag runs on the default data
/// directory (the existing Claude); anything else is no CSW environment.
/// `envs` pairs each environment name with its desktop data directory.
pub fn environment_for_args_line(
    line: &str,
    envs: &[(String, std::path::PathBuf)],
    default_dir: &Path,
) -> Option<String> {
    for (name, dir) in envs {
        let needle = format!("--user-data-dir={}", dir.display());
        if flag_value_present(line, &needle) {
            return Some(name.clone());
        }
    }
    if !line.contains("--user-data-dir=") {
        return envs
            .iter()
            .find(|(_, dir)| dir == default_dir)
            .map(|(name, _)| name.clone());
    }
    None
}

/// The pid of the running Claude main process that belongs to the `target`
/// environment, so its window can be brought to the front. Each `(pid, line)`
/// is resolved the same way as [`environment_for_args_line`]: a `--user-data-dir`
/// match names the environment, and a line without the flag is the existing
/// Claude (`default`). Returns the first matching pid, or `None` when that
/// environment has no running Claude.
pub fn pid_for_environment(
    processes: &[(u32, String)],
    envs: &[(String, std::path::PathBuf)],
    default_dir: &Path,
    target: &str,
) -> Option<u32> {
    processes.iter().find_map(|(pid, line)| {
        match environment_for_args_line(line, envs, default_dir) {
            Some(name) if name == target => Some(*pid),
            _ => None,
        }
    })
}

/// True when a live Claude main process was launched without `--user-data-dir`,
/// meaning it runs on the default data directory and was not started by CSW:
/// the Dock, or Squirrel's post-update relaunch, which drops the arguments.
/// The takeover notice uses this to tell "the update restarted Claude outside
/// the environment" apart from a CSW-launched existing Claude.
pub fn unmanaged_default_running(running_args: &[String]) -> bool {
    running_args.iter().any(|a| !a.contains("--user-data-dir="))
}

/// One environment's launch classification for [`plan_relaunch`]: its name, and
/// whether it can run concurrently. `is_default` is the existing Claude; a
/// non-fully-isolated environment shares files and must run alone.
#[derive(Debug, Clone)]
pub struct RelaunchEnv {
    pub name: String,
    pub is_default: bool,
    pub is_fully_isolated: bool,
}

/// What to launch to reopen a previously-open set of environments, and what
/// cannot be launched right now.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RelaunchPlan {
    /// The single non-concurrent environment (existing Claude, or a shared/copy
    /// environment) to switch to first. At most one, and only when nothing else
    /// is running.
    pub active: Option<String>,
    /// Fully-isolated environments to open alongside, in additional windows.
    pub additional: Vec<String>,
    /// Environments that cannot be reopened now because another Claude is
    /// running (they share files and run one at a time). The UI tells the user
    /// to quit the running Claude first.
    pub blocked: Vec<String>,
}

/// Plan how to reopen a recorded set of environments safely.
///
/// - Environments already running are excluded outright (default and
///   fully-isolated alike): `open -n` always spawns a new process, so
///   relaunching a running one would double it on the same data directory.
/// - Fully-isolated environments share nothing and run side by side, so they go
///   to `additional`.
/// - The existing Claude and shared/copy environments run one at a time. With
///   nothing running, one is chosen as `active`; the rest go to `blocked`. If
///   any Claude is already running, `switch_to` would be refused, so all
///   non-concurrent candidates are `blocked`.
///
/// `envs` describes every known environment; entries in `recorded` or `running`
/// with no matching env are ignored (e.g. an environment deleted since capture).
pub fn plan_relaunch(
    recorded: &[String],
    running: &[String],
    envs: &[RelaunchEnv],
) -> RelaunchPlan {
    let lookup = |name: &str| envs.iter().find(|e| e.name == name);
    // Any Claude running at all blocks a switch_to (its guard refuses while
    // Claude Desktop runs), so a non-concurrent environment can only seat when
    // nothing is running.
    let anything_running = running.iter().any(|n| lookup(n).is_some());

    let mut plan = RelaunchPlan::default();
    for name in recorded {
        if running.contains(name) {
            continue; // already up; never relaunch a running instance
        }
        let Some(env) = lookup(name) else {
            continue; // unknown/deleted environment
        };
        if !env.is_default && env.is_fully_isolated {
            plan.additional.push(name.clone());
        } else if !anything_running && plan.active.is_none() {
            plan.active = Some(name.clone());
        } else {
            plan.blocked.push(name.clone());
        }
    }
    plan
}

/// True if `args` contains `needle` as a whole token: followed by a space or the
/// end of the string. This keeps a value like `.../Claude` from matching a longer
/// sibling `.../Claude2`, while still allowing the value itself to contain spaces
/// (the default dir lives under "Application Support").
fn flag_value_present(args: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(rel) = args[from..].find(needle) {
        let end = from + rel + needle.len();
        if end == args.len() || args.as_bytes()[end] == b' ' {
            return true;
        }
        from = end;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::mock::MockPlatformProvider;
    use crate::profile::SharingConfig;
    use tempfile::tempdir;

    /// Switching while Claude Desktop is running must be refused so a live
    /// instance cannot write cached state back / race with shared files.
    #[test]
    fn switch_to_refuses_while_desktop_running() {
        let desktop_dir = tempdir().unwrap();
        let cli_dir = tempdir().unwrap();
        let app_dir = tempdir().unwrap();

        let provider = Arc::new(
            MockPlatformProvider::new(
                desktop_dir.path().to_path_buf(),
                cli_dir.path().to_path_buf(),
                app_dir.path().to_path_buf(),
            )
            .with_desktop_running(true),
        );
        let pm = Arc::new(ProfileManager::new(provider.clone()).unwrap());
        pm.create_profile("Work", SharingConfig::default(), None)
            .unwrap();

        let switcher = ContextSwitcher::new(provider, pm.clone());
        let err = switcher.switch_to("Work").unwrap_err();
        assert!(matches!(err, CswError::DesktopRunning));
        // Active profile must be unchanged after a refused switch.
        assert_eq!(pm.active_profile_name(), "default");
    }

    #[test]
    fn switch_to_succeeds_when_desktop_not_running() {
        let desktop_dir = tempdir().unwrap();
        let cli_dir = tempdir().unwrap();
        let app_dir = tempdir().unwrap();

        let provider = Arc::new(MockPlatformProvider::new(
            desktop_dir.path().to_path_buf(),
            cli_dir.path().to_path_buf(),
            app_dir.path().to_path_buf(),
        ));
        let pm = Arc::new(ProfileManager::new(provider.clone()).unwrap());
        pm.create_profile("Work", SharingConfig::default(), None)
            .unwrap();

        let switcher = ContextSwitcher::new(provider, pm.clone());
        switcher.switch_to("Work").unwrap();
        assert_eq!(pm.active_profile_name(), "Work");
    }

    use std::path::Path;

    /// An environment is running when a live Claude main process was launched with
    /// its `--user-data-dir`.
    #[test]
    fn desktop_dir_running_matches_user_data_dir_arg() {
        let default_dir = Path::new("/Users/x/Library/Application Support/Claude");
        let work = Path::new("/Users/x/.context-switcher-claude/profiles/Work/desktop-data");
        let args = vec![format!(
            "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={}",
            work.display()
        )];
        assert!(desktop_dir_running(work, &args, default_dir));
        assert!(!desktop_dir_running(default_dir, &args, default_dir));
    }

    /// A plain Finder/Dock launch carries no `--user-data-dir`, so it maps to the
    /// default data dir (the existing Claude), not to any created environment.
    #[test]
    fn desktop_dir_running_no_flag_counts_as_default() {
        let default_dir = Path::new("/Users/x/Library/Application Support/Claude");
        let work = Path::new("/Users/x/.context-switcher-claude/profiles/Work/desktop-data");
        let args = vec!["/Applications/Claude.app/Contents/MacOS/Claude".to_string()];
        assert!(desktop_dir_running(default_dir, &args, default_dir));
        assert!(!desktop_dir_running(work, &args, default_dir));
    }

    /// CSW launches even the default profile with an explicit `--user-data-dir`
    /// that contains spaces ("Application Support"); the match must still work and
    /// must not spuriously match a longer sibling path ("Claude2").
    #[test]
    fn desktop_dir_running_handles_spaces_and_prefix_siblings() {
        let default_dir = Path::new("/Users/x/Library/Application Support/Claude");
        let sibling = "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir=/Users/x/Library/Application Support/Claude2".to_string();
        assert!(!desktop_dir_running(default_dir, &[sibling], default_dir));

        let exact = format!(
            "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir={} --enable-logging",
            default_dir.display()
        );
        assert!(desktop_dir_running(default_dir, &[exact], default_dir));
    }

    /// Two fully-isolated environments can run at once; both count as running.
    #[test]
    fn desktop_dir_running_supports_multiple_concurrent() {
        let default_dir = Path::new("/d/Application Support/Claude");
        let a = Path::new("/p/A/desktop-data");
        let b = Path::new("/p/B/desktop-data");
        let args = vec![
            format!("...MacOS/Claude --user-data-dir={}", a.display()),
            format!("...MacOS/Claude --user-data-dir={}", b.display()),
        ];
        assert!(desktop_dir_running(a, &args, default_dir));
        assert!(desktop_dir_running(b, &args, default_dir));
        assert!(!desktop_dir_running(default_dir, &args, default_dir));
    }

    #[test]
    fn desktop_dir_running_false_when_nothing_running() {
        let default_dir = Path::new("/d");
        let work = Path::new("/w");
        assert!(!desktop_dir_running(work, &[], default_dir));
        assert!(!desktop_dir_running(default_dir, &[], default_dir));
    }

    // Spec: docs/proposals/frontmost-environment-check.md. Resolve one main
    // process line to the environment it runs, mirroring desktop_dir_running.
    #[test]
    fn environment_for_args_line_resolves_env_default_and_unknown() {
        let default_dir = Path::new("/Users/u/Library/Application Support/Claude");
        let envs = [
            (
                "Work".to_string(),
                std::path::PathBuf::from("/p/Work/desktop-data"),
            ),
            (
                "Work2".to_string(),
                std::path::PathBuf::from("/p/Work2/desktop-data"),
            ),
            ("default".to_string(), default_dir.to_path_buf()),
        ];

        // A --user-data-dir match names that environment; a sibling with the
        // matched dir as a prefix must not be confused with it.
        assert_eq!(
            environment_for_args_line(
                "...MacOS/Claude --user-data-dir=/p/Work/desktop-data",
                &envs,
                default_dir
            ),
            Some("Work".to_string())
        );
        assert_eq!(
            environment_for_args_line(
                "...MacOS/Claude --user-data-dir=/p/Work2/desktop-data",
                &envs,
                default_dir
            ),
            Some("Work2".to_string())
        );
        // No flag means the default data directory: the existing Claude.
        assert_eq!(
            environment_for_args_line("...MacOS/Claude", &envs, default_dir),
            Some("default".to_string())
        );
        // A dir CSW does not manage resolves to no environment.
        assert_eq!(
            environment_for_args_line(
                "...MacOS/Claude --user-data-dir=/elsewhere",
                &envs,
                default_dir
            ),
            None
        );
    }

    // Spec: docs/SPECIFICATION.md「利用中の環境を前面に表示する」. Resolve which
    // running main process (pid) belongs to a target environment, so its Claude
    // can be brought to the front. Mirrors environment_for_args_line per process.
    #[test]
    fn pid_for_environment_matches_env_default_and_missing() {
        let default_dir = Path::new("/Users/u/Library/Application Support/Claude");
        let envs = [
            (
                "Work".to_string(),
                std::path::PathBuf::from("/p/Work/desktop-data"),
            ),
            ("default".to_string(), default_dir.to_path_buf()),
        ];
        let processes = [
            (
                101u32,
                "...MacOS/Claude --user-data-dir=/p/Work/desktop-data".to_string(),
            ),
            (202u32, "...MacOS/Claude".to_string()),
        ];

        // A managed environment resolves to the pid of its main process.
        assert_eq!(
            pid_for_environment(&processes, &envs, default_dir, "Work"),
            Some(101)
        );
        // The existing Claude (no --user-data-dir) resolves to its pid.
        assert_eq!(
            pid_for_environment(&processes, &envs, default_dir, "default"),
            Some(202)
        );
        // An environment with no running Claude yields no pid.
        assert_eq!(pid_for_environment(&[], &envs, default_dir, "Work"), None);
        assert_eq!(
            pid_for_environment(&processes, &envs, default_dir, "Other"),
            None
        );
    }

    // Spec: docs/proposals/update-takeover-notice.md. A main process without
    // --user-data-dir was launched outside CSW (Dock, or Squirrel's post-update
    // relaunch) and runs on the default data directory.
    #[test]
    fn unmanaged_default_running_detects_argless_main_process() {
        assert!(unmanaged_default_running(&[
            "...MacOS/Claude".to_string(),
            "...MacOS/Claude --user-data-dir=/w".to_string(),
        ]));
        assert!(!unmanaged_default_running(&[
            "...MacOS/Claude --user-data-dir=/w".to_string(),
            "...MacOS/Claude --user-data-dir=/d".to_string(),
        ]));
        assert!(!unmanaged_default_running(&[]));
    }

    // Spec: docs/proposals/reopen-previous-set.md. Reopen a recorded set safely.
    fn envs() -> Vec<RelaunchEnv> {
        vec![
            RelaunchEnv {
                name: "default".into(),
                is_default: true,
                is_fully_isolated: false,
            },
            RelaunchEnv {
                name: "Iso1".into(),
                is_default: false,
                is_fully_isolated: true,
            },
            RelaunchEnv {
                name: "Iso2".into(),
                is_default: false,
                is_fully_isolated: true,
            },
            RelaunchEnv {
                name: "Shared".into(),
                is_default: false,
                is_fully_isolated: false,
            },
        ]
    }

    #[test]
    fn plan_relaunch_excludes_running_default_and_isolated() {
        // The existing Claude already relaunched (default running) and Iso1 is
        // up; only the still-closed isolated environment gets reopened.
        let plan = plan_relaunch(
            &["default".into(), "Iso1".into(), "Iso2".into()],
            &["default".into(), "Iso1".into()],
            &envs(),
        );
        assert_eq!(plan.active, None); // default already running, not relaunched
        assert_eq!(plan.additional, vec!["Iso2".to_string()]);
        assert!(plan.blocked.is_empty());
    }

    #[test]
    fn plan_relaunch_seats_one_primary_and_adds_isolated_when_idle() {
        // Nothing running: the existing Claude seats as active, the isolated
        // environments open alongside it.
        let plan = plan_relaunch(
            &["default".into(), "Iso1".into(), "Iso2".into()],
            &[],
            &envs(),
        );
        assert_eq!(plan.active.as_deref(), Some("default"));
        assert_eq!(
            plan.additional,
            vec!["Iso1".to_string(), "Iso2".to_string()]
        );
        assert!(plan.blocked.is_empty());
    }

    #[test]
    fn plan_relaunch_blocks_second_non_concurrent() {
        // Both the existing Claude and a shared environment were recorded, and
        // nothing runs: only one non-concurrent can seat; the other is blocked.
        let plan = plan_relaunch(
            &["default".into(), "Shared".into(), "Iso1".into()],
            &[],
            &envs(),
        );
        assert_eq!(plan.active.as_deref(), Some("default"));
        assert_eq!(plan.additional, vec!["Iso1".to_string()]);
        assert_eq!(plan.blocked, vec!["Shared".to_string()]);
    }

    #[test]
    fn plan_relaunch_blocks_shared_while_something_runs() {
        // The existing Claude is up (e.g. relaunched by the update). A recorded
        // shared environment cannot be switched to while Claude runs, but a
        // recorded isolated environment still opens alongside.
        let plan = plan_relaunch(
            &["Shared".into(), "Iso1".into()],
            &["default".into()],
            &envs(),
        );
        assert_eq!(plan.active, None);
        assert_eq!(plan.additional, vec!["Iso1".to_string()]);
        assert_eq!(plan.blocked, vec!["Shared".to_string()]);
    }

    #[test]
    fn plan_relaunch_ignores_deleted_environment() {
        let plan = plan_relaunch(&["Gone".into(), "Iso1".into()], &[], &envs());
        assert_eq!(plan.additional, vec!["Iso1".to_string()]);
        assert_eq!(plan.active, None);
        assert!(plan.blocked.is_empty());
    }
}

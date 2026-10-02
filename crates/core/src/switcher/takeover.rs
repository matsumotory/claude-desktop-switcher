//! The update-takeover watcher (SPECIFICATION.md §5.A 乗り移りの検知).
//!
//! Claude Desktop's updater relaunches the app without arguments, so a Claude
//! that ran in an environment comes back on the default data directory, the
//! existing Claude. The tray's 3-second poll feeds each process snapshot to
//! [`TakeoverWatch`], which reports the environment that was taken over so the
//! settings window can explain it.

use std::time::{Duration, Instant};

/// Pids of the Claude main processes launched without `--user-data-dir`: they
/// run on the default data directory and were not started by CSW (the Dock,
/// or the updater's relaunch). Same test as [`super::unmanaged_default_running`].
pub fn unmanaged_pids(processes: &[(u32, String)]) -> Vec<u32> {
    processes
        .iter()
        .filter(|(_, line)| !line.contains("--user-data-dir="))
        .map(|(pid, _)| *pid)
        .collect()
}

/// Watches process snapshots for the update-takeover transition: an
/// environment's Claude vanishes and, at the same time or within `window`, a
/// Claude launched outside CSW appears as a new process.
///
/// "Appears" means a pid the previous snapshot did not have. An existing
/// Claude that keeps running under the same pid never counts, so quitting an
/// environment's Claude that ran beside it is not a takeover. An update still
/// is: every Claude quits before the updater relaunches one without
/// arguments, so even an existing Claude that was running comes back under a
/// new pid.
pub struct TakeoverWatch {
    window: Duration,
    /// False until the first snapshot, which only sets the baseline: a Claude
    /// already running when the watch starts did not appear.
    started: bool,
    prev_envs: Vec<String>,
    prev_unmanaged: Vec<u32>,
    recently_gone: Vec<(String, Instant)>,
    /// When a new outside-CSW pid last appeared, while within the window.
    appeared_at: Option<Instant>,
}

impl TakeoverWatch {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            started: false,
            prev_envs: Vec::new(),
            prev_unmanaged: Vec::new(),
            recently_gone: Vec::new(),
            appeared_at: None,
        }
    }

    /// Record one snapshot: the environments whose Claude is running
    /// (`default` included when the existing Claude runs) and the pids of the
    /// Claude main processes launched outside CSW.
    pub fn observe(&mut self, now: Instant, running_envs: &[String], unmanaged_pids: &[u32]) {
        if self.started {
            for name in &self.prev_envs {
                if name != "default" && !running_envs.contains(name) {
                    self.recently_gone.push((name.clone(), now));
                }
            }
            if unmanaged_pids
                .iter()
                .any(|pid| !self.prev_unmanaged.contains(pid))
            {
                self.appeared_at = Some(now);
            }
        }
        self.started = true;
        let window = self.window;
        self.recently_gone
            .retain(|(_, at)| now.duration_since(*at) <= window);
        self.appeared_at = self
            .appeared_at
            .filter(|at| now.duration_since(*at) <= window);
        self.prev_envs = running_envs.to_vec();
        self.prev_unmanaged = unmanaged_pids.to_vec();
    }

    /// The environment taken over as of the latest snapshot, if any: the
    /// newest environment that vanished no later than the new outside-CSW
    /// Claude appeared and is still gone, while that Claude still runs. The
    /// entry is consumed, so a dismissed notice is not raised again by the
    /// same event.
    pub fn take_takeover(&mut self) -> Option<String> {
        let appeared = self.appeared_at?;
        if self.prev_unmanaged.is_empty() {
            return None;
        }
        let pos = self
            .recently_gone
            .iter()
            .rposition(|(n, gone)| *gone <= appeared && !self.prev_envs.contains(n))?;
        Some(self.recently_gone.remove(pos).0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: Duration = Duration::from_secs(12);

    fn envs(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// Feed snapshots taken 3 seconds apart, the tray's poll interval, and
    /// collect what `take_takeover` reports after each one.
    fn run(snapshots: &[(&[&str], &[u32])]) -> Vec<Option<String>> {
        let start = Instant::now();
        let mut watch = TakeoverWatch::new(WINDOW);
        snapshots
            .iter()
            .enumerate()
            .map(|(i, (running, pids))| {
                let now = start + Duration::from_secs(3 * i as u64);
                watch.observe(now, &envs(running), pids);
                watch.take_takeover()
            })
            .collect()
    }

    #[test]
    fn unmanaged_pids_are_the_main_processes_without_user_data_dir() {
        let processes = vec![
            (100, "/Applications/Claude.app/Contents/MacOS/Claude".to_string()),
            (
                200,
                "/Applications/Claude.app/Contents/MacOS/Claude --user-data-dir=/p/work/desktop-data"
                    .to_string(),
            ),
        ];
        assert_eq!(unmanaged_pids(&processes), vec![100]);
    }

    /// The user quits an environment's Claude that ran alongside the existing
    /// Claude. The existing Claude keeps its pid, so no Claude appeared: this
    /// is not a takeover (the notice must not claim an update reopened it).
    #[test]
    fn quitting_an_environment_beside_a_running_existing_claude_is_not_a_takeover() {
        let got = run(&[
            (&["default", "work"], &[100]),
            (&["default"], &[100]),
            (&["default"], &[100]),
            (&["default"], &[100]),
        ]);
        assert_eq!(got, vec![None, None, None, None]);
    }

    /// The updater quits Claude and relaunches it without arguments: the
    /// environment vanishes, then a new outside-CSW Claude appears.
    #[test]
    fn environment_vanishing_then_a_new_outside_claude_appearing_is_a_takeover() {
        let got = run(&[(&["work"], &[]), (&[], &[]), (&["default"], &[300])]);
        assert_eq!(got, vec![None, None, Some("work".to_string())]);
    }

    /// The update also restarts an existing Claude that ran alongside: its old
    /// pid goes away and the relaunched Claude comes back with a new pid, in
    /// a later snapshot or in the same one.
    #[test]
    fn update_restarting_the_existing_claude_too_is_a_takeover() {
        let later = run(&[
            (&["default", "work"], &[100]),
            (&[], &[]),
            (&["default"], &[300]),
        ]);
        assert_eq!(later, vec![None, None, Some("work".to_string())]);

        let same_snapshot = run(&[(&["default", "work"], &[100]), (&["default"], &[300])]);
        assert_eq!(same_snapshot, vec![None, Some("work".to_string())]);
    }

    /// An outside-CSW Claude that appears more than the window after the
    /// environment vanished is a separate launch, not the update's relaunch.
    #[test]
    fn outside_claude_appearing_after_the_window_is_not_a_takeover() {
        let got = run(&[
            (&["work"], &[]),
            (&[], &[]),
            (&[], &[]),
            (&[], &[]),
            (&[], &[]),
            (&[], &[]),
            (&["default"], &[300]),
        ]);
        assert!(got.iter().all(Option::is_none), "{got:?}");
    }

    /// An outside-CSW Claude opened first (from the Dock) and an environment
    /// quit afterwards is the user's own order of actions, not a relaunch.
    #[test]
    fn outside_claude_appearing_before_the_environment_vanished_is_not_a_takeover() {
        let got = run(&[
            (&["work"], &[]),
            (&["default", "work"], &[300]),
            (&["default"], &[300]),
        ]);
        assert_eq!(got, vec![None, None, None]);
    }

    /// The first snapshot is the baseline: an existing Claude already running
    /// when CSW starts did not appear.
    #[test]
    fn existing_claude_running_when_the_watch_starts_did_not_appear() {
        let got = run(&[(&["default", "work"], &[100]), (&["default"], &[100])]);
        assert_eq!(got, vec![None, None]);
    }

    /// A takeover is reported once, so a dismissed notice stays dismissed.
    #[test]
    fn a_takeover_is_reported_once() {
        let start = Instant::now();
        let mut watch = TakeoverWatch::new(WINDOW);
        watch.observe(start, &envs(&["work"]), &[]);
        watch.observe(start + Duration::from_secs(3), &envs(&["default"]), &[300]);
        assert_eq!(watch.take_takeover(), Some("work".to_string()));
        assert_eq!(watch.take_takeover(), None);
        watch.observe(start + Duration::from_secs(6), &envs(&["default"]), &[300]);
        assert_eq!(watch.take_takeover(), None);
    }
}

use crate::profile::Profile;

/// Generate the shell script that points `CLAUDE_CONFIG_DIR` at an
/// environment's cli-data: the single line `export CLAUDE_CONFIG_DIR='...'`.
///
/// It must stay exactly that one line. Users run it as the unquoted
/// `eval $(csw env <name>)` (what `csw switch`, the GUI and the docs show).
/// Unquoted command substitution is split into words and `eval` joins them
/// back into a single line, so a leading comment line would swallow the
/// `export` and set nothing. Anything for humans goes to stderr in the CLI.
pub fn generate_env_script(profile: &Profile) -> String {
    let path_str = profile.isolation.cli_config_dir.to_string_lossy();
    // Escape single quotes for safety in sh/zsh/bash
    let escaped_path = path_str.replace('\'', "'\\''");
    format!("export CLAUDE_CONFIG_DIR='{}'\n", escaped_path)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::profile::{IsolationConfig, ProfileMeta, SharingConfig};
    use std::path::PathBuf;
    use std::process::Command;

    fn profile_with_cli_dir(name: &str, cli_dir: &str) -> Profile {
        Profile {
            profile: ProfileMeta {
                name: name.to_string(),
                icon: String::new(),
                color: String::new(),
                is_default: false,
                note: String::new(),
                created_at: None,
                cloned_from: None,
            },
            isolation: IsolationConfig {
                desktop_user_data_dir: PathBuf::from("/unused/desktop-data"),
                cli_config_dir: PathBuf::from(cli_dir),
            },
            sharing: SharingConfig::default(),
        }
    }

    /// Evaluate `script` in `shell` the way the docs tell users to, and return
    /// the resulting `CLAUDE_CONFIG_DIR`. `S` stands in for `csw env <name>`,
    /// so `eval $(printf %s "$S")` is the documented `eval $(csw env <name>)`.
    /// `None` when that shell is not installed on this machine.
    fn eval_in(shell: &str, quoted: bool, script: &str) -> Option<String> {
        let eval = if quoted {
            r#"eval "$(printf %s "$S")""#
        } else {
            r#"eval $(printf %s "$S")"#
        };
        let mut cmd = Command::new(shell);
        if shell == "zsh" {
            // Skip the user's rc files so the check sees zsh defaults only.
            cmd.arg("-f");
        }
        let out = match cmd
            .arg("-c")
            .arg(format!(r#"{eval}; printf %s "$CLAUDE_CONFIG_DIR""#))
            .env("S", script)
            .env_remove("CLAUDE_CONFIG_DIR")
            .output()
        {
            Ok(out) => out,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(e) => panic!("failed to run {shell}: {e}"),
        };
        assert!(
            out.status.success(),
            "{shell} exited with {:?}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
        Some(String::from_utf8(out.stdout).expect("utf-8 output"))
    }

    /// The paths an environment's cli-data can take: a plain one, a Japanese
    /// environment name, and a home folder with a space and a single quote.
    const CASES: &[(&str, &str)] = &[
        (
            "Work",
            "/Users/me/.context-switcher-claude/profiles/Work/cli-data",
        ),
        (
            "仕事",
            "/Users/me/.context-switcher-claude/profiles/仕事/cli-data",
        ),
        (
            "Work",
            "/Users/Jo O'Neil/.context-switcher-claude/profiles/Work/cli-data",
        ),
    ];

    /// `eval $(csw env <name>)` is the unquoted form printed by `csw switch`,
    /// the GUI and every doc. Unquoted command substitution is split into words
    /// and `eval` joins them back into one line, so any leading `# ...` line
    /// would turn the `export` into part of a comment and set nothing.
    #[test]
    fn unquoted_eval_sets_claude_config_dir() {
        for (name, dir) in CASES {
            let script = generate_env_script(&profile_with_cli_dir(name, dir));
            let got = eval_in("sh", false, &script).expect("sh must be installed");
            assert_eq!(got, *dir, "sh: eval $(csw env {name}) with {script:?}");
            for shell in ["bash", "zsh"] {
                if let Some(got) = eval_in(shell, false, &script) {
                    assert_eq!(got, *dir, "{shell}: eval $(csw env {name}) with {script:?}");
                }
            }
        }
    }

    /// The quoted form `eval "$(csw env <name>)"` must keep working too.
    #[test]
    fn quoted_eval_sets_claude_config_dir() {
        for (name, dir) in CASES {
            let script = generate_env_script(&profile_with_cli_dir(name, dir));
            for shell in ["sh", "bash", "zsh"] {
                if let Some(got) = eval_in(shell, true, &script) {
                    assert_eq!(got, *dir, "{shell}: eval \"$(csw env {name})\"");
                }
            }
        }
    }

    /// Stdout of `csw env` is only the export statement (SPECIFICATION.md §5.B):
    /// no comment line that the unquoted eval could merge the export into.
    #[test]
    fn env_script_is_only_the_export_line() {
        for (name, dir) in CASES {
            let script = generate_env_script(&profile_with_cli_dir(name, dir));
            assert!(
                !script.lines().any(|l| l.trim_start().starts_with('#')),
                "no comment lines expected: {script:?}"
            );
            assert_eq!(script.lines().count(), 1, "one line expected: {script:?}");
            assert!(script.starts_with("export CLAUDE_CONFIG_DIR='"));
        }
    }
}

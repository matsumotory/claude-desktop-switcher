use crate::profile::Profile;

/// Generate shell script to configure the environment for the active profile.
/// Prints the export CLAUDE_CONFIG_DIR=... statement.
pub fn generate_env_script(profile: &Profile) -> String {
    let path_str = profile.isolation.cli_config_dir.to_string_lossy();
    // Escape single quotes for safety in sh/zsh/bash
    let escaped_path = path_str.replace('\'', "'\\''");
    format!(
        "# Environment settings for profile: {}\nexport CLAUDE_CONFIG_DIR='{}'\n",
        profile.profile.name, escaped_path
    )
}

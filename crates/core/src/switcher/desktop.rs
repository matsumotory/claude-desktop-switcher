use crate::error::Result;
use crate::platform::PlatformProvider;
use crate::profile::Profile;

/// Launch Claude Desktop for the given profile using --user-data-dir
pub fn launch_desktop(profile: &Profile, provider: &dyn PlatformProvider) -> Result<()> {
    provider.launch_claude_desktop(
        &profile.isolation.desktop_user_data_dir,
        Some(&profile.isolation.cli_config_dir),
    )
}

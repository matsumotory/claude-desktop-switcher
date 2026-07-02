use thiserror::Error;

#[derive(Error, Debug)]
pub enum CswError {
    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),

    #[error("Profile '{0}' already exists")]
    ProfileAlreadyExists(String),

    #[error("Invalid environment name: {0}")]
    InvalidProfileName(String),

    #[error("Default profile cannot be modified")]
    DefaultProfileImmutable,

    #[error("Default profile cannot be deleted")]
    DefaultProfileCannotBeDeleted,

    #[error("Active profile '{0}' cannot be deleted. Switch to another profile first.")]
    ActiveProfileCannotBeDeleted(String),

    #[error("Symlink creation failed: {source} -> {target}")]
    SymlinkFailed {
        source: String,
        target: String,
        #[source]
        cause: std::io::Error,
    },

    #[error("Existing file would be overwritten (non-destructive violation): {0}")]
    NonDestructiveViolation(String),

    #[error("Refused to delete '{0}': path lies inside the real Claude default data directory")]
    RefusedDefaultDataDeletion(String),

    #[error(
        "Claude Desktop is running. Quit it before switching profiles to avoid cache write-back and data races."
    )]
    DesktopRunning,

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Claude Desktop is not installed at expected path")]
    DesktopNotInstalled,

    #[error(
        "Environment '{0}' shares its settings.json with the existing Claude, so enabling usage display here would also change the existing Claude. Enable it on the existing Claude instead."
    )]
    UsageSettingsShared(String),

    #[error("Could not parse {0}. Fix or remove the file, then try again: {1}")]
    UsageSettingsUnreadable(String, String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CswError>;

use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
pub mod macos;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeychainCredential {
    pub service: String,
    pub account: String,
    pub password: String,
}

pub trait KeychainProvider {
    /// Retrieve a password from Keychain. Returns None if it doesn't exist.
    fn get_password(&self, service: &str, account: &str) -> crate::error::Result<Option<String>>;

    /// Save a password to Keychain. Overwrites if it already exists.
    fn set_password(&self, service: &str, account: &str, password: &str) -> crate::error::Result<()>;

    /// Delete a password from Keychain.
    fn delete_password(&self, service: &str, account: &str) -> crate::error::Result<()>;
}

/// Create the keychain provider for the current OS.
pub fn create_keychain_provider() -> Box<dyn KeychainProvider> {
    #[cfg(test)]
    {
        return Box::new(mock::MockKeychainProvider::new());
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsKeychainProvider::new())
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Fallback stub for other OSes
        struct StubKeychainProvider;
        impl KeychainProvider for StubKeychainProvider {
            fn get_password(&self, _service: &str, _account: &str) -> crate::error::Result<Option<String>> {
                Ok(None)
            }
            fn set_password(&self, _service: &str, _account: &str, _password: &str) -> crate::error::Result<()> {
                Ok(())
            }
            fn delete_password(&self, _service: &str, _account: &str) -> crate::error::Result<()> {
                Ok(())
            }
        }
        Box::new(StubKeychainProvider)
    }
}

#[cfg(test)]
pub mod mock;

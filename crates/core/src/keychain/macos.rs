use std::process::Command;
use crate::error::{CswError, Result};
use crate::keychain::KeychainProvider;

pub struct MacOsKeychainProvider;

impl MacOsKeychainProvider {
    pub fn new() -> Self {
        Self
    }
}

impl KeychainProvider for MacOsKeychainProvider {
    fn get_password(&self, service: &str, account: &str) -> Result<Option<String>> {
        let output = Command::new("security")
            .arg("find-generic-password")
            .arg("-s")
            .arg(service)
            .arg("-a")
            .arg(account)
            .arg("-w")
            .output()?;

        if output.status.success() {
            let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(password))
        } else {
            // Check stderr or exit code. If exit code is 45 (not found), return None.
            // Under normal circumstances, any failure here means it doesn't exist or is locked.
            Ok(None)
        }
    }

    fn set_password(&self, service: &str, account: &str, password: &str) -> Result<()> {
        // -U updates the item if it already exists
        let status = Command::new("security")
            .arg("add-generic-password")
            .arg("-s")
            .arg(service)
            .arg("-a")
            .arg(account)
            .arg("-w")
            .arg(password)
            .arg("-U")
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(CswError::Other(format!(
                "Failed to set keychain password for service '{}', account '{}'",
                service, account
            )))
        }
    }

    fn delete_password(&self, service: &str, account: &str) -> Result<()> {
        // Check if it exists first to avoid unnecessary error status
        if self.get_password(service, account)?.is_none() {
            return Ok(());
        }

        let status = Command::new("security")
            .arg("delete-generic-password")
            .arg("-s")
            .arg(service)
            .arg("-a")
            .arg(account)
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(CswError::Other(format!(
                "Failed to delete keychain password for service '{}', account '{}'",
                service, account
            )))
        }
    }
}

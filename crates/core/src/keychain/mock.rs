use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::error::Result;
use crate::keychain::KeychainProvider;

#[derive(Clone, Default)]
pub struct MockKeychainProvider {
    // service -> (account -> password)
    store: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
}

impl MockKeychainProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl KeychainProvider for MockKeychainProvider {
    fn get_password(&self, service: &str, account: &str) -> Result<Option<String>> {
        let store = self.store.lock().unwrap();
        if let Some(accounts) = store.get(service) {
            if let Some(pwd) = accounts.get(account) {
                return Ok(Some(pwd.clone()));
            }
        }
        Ok(None)
    }

    fn set_password(&self, service: &str, account: &str, password: &str) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store
            .entry(service.to_string())
            .or_insert_with(HashMap::new)
            .insert(account.to_string(), password.to_string());
        Ok(())
    }

    fn delete_password(&self, service: &str, account: &str) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(accounts) = store.get_mut(service) {
            accounts.remove(account);
        }
        Ok(())
    }
}

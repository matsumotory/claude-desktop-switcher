use std::collections::HashMap;
use std::cell::RefCell;
use crate::error::Result;
use crate::keychain::KeychainProvider;

thread_local! {
    static STORE: RefCell<HashMap<String, HashMap<String, String>>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Default)]
pub struct MockKeychainProvider {}

impl MockKeychainProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl KeychainProvider for MockKeychainProvider {
    fn get_password(&self, service: &str, account: &str) -> Result<Option<String>> {
        STORE.with(|store| {
            let store = store.borrow();
            if let Some(accounts) = store.get(service) {
                if let Some(pwd) = accounts.get(account) {
                    return Ok(Some(pwd.clone()));
                }
            }
            Ok(None)
        })
    }

    fn set_password(&self, service: &str, account: &str, password: &str) -> Result<()> {
        STORE.with(|store| {
            let mut store = store.borrow_mut();
            store
                .entry(service.to_string())
                .or_insert_with(HashMap::new)
                .insert(account.to_string(), password.to_string());
            Ok(())
        })
    }

    fn delete_password(&self, service: &str, account: &str) -> Result<()> {
        STORE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(accounts) = store.get_mut(service) {
                accounts.remove(account);
            }
            Ok(())
        })
    }
}

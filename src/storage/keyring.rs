use anyhow::{Context, Result};
use keyring::v1::Entry;

const SERVICE_NAME: &str = "twofauth-client";
const TOKEN_NAME: &str = "api-token";

fn credential_entry(service: &str, username: &str) -> Result<Entry> {
    Entry::new(service, username).context("Failed to open the system keyring")
}

fn token_entry() -> Result<Entry> {
    credential_entry(SERVICE_NAME, TOKEN_NAME)
}

pub fn save_token(token: &str) -> Result<()> {
    let entry = token_entry()?;

    entry
        .set_password(token)
        .context("Failed to save the 2FAuth token in the system keyring")
}

pub fn load_token() -> Result<String> {
    let entry = token_entry()?;

    entry
        .get_password()
        .context("Failed to load the 2FAuth token from the system keyring")
}

pub fn delete_token() -> Result<()> {
    let entry = token_entry()?;

    entry
        .delete_credential()
        .context("Failed to delete the 2FAuth token from the system keyring")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SERVICE_NAME: &str = "twofauth-client-tests";
    const TEST_TOKEN_NAME: &str = "api-token-test";
    const TEST_TOKEN: &str = "twofauth-client-keyring-test";

    #[test]
    fn keyring_roundtrip() {
        let entry = credential_entry(TEST_SERVICE_NAME, TEST_TOKEN_NAME)
            .expect("Could not open test keyring entry");

        entry
            .set_password(TEST_TOKEN)
            .expect("Could not save test token");

        let loaded = entry.get_password().expect("Could not load test token");

        assert_eq!(loaded, TEST_TOKEN);

        entry
            .delete_credential()
            .expect("Could not delete test token");
    }
}

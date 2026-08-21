use anyhow::{Context, Result};
use keyring::v1::Entry;

const SERVICE_NAME: &str = "twofauth-client";
const TOKEN_NAME: &str = "api-token";

fn token_entry() -> Result<Entry> {
    Entry::new(SERVICE_NAME, TOKEN_NAME).context("Failed to open the system keyring")
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

    #[test]
    fn keyring_roundtrip() {
        const TEST_TOKEN: &str = "twofauth-client-keyring-test";

        save_token(TEST_TOKEN).expect("Could not save test token");

        let loaded = load_token().expect("Could not load test token");

        assert_eq!(loaded, TEST_TOKEN);

        delete_token().expect("Could not delete test token");
    }
}

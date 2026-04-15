use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "smlcli";
const MASTER_KEY_USER: &str = "master-key";

pub fn get_or_create_master_key() -> Result<Vec<u8>> {
    let entry = Entry::new(SERVICE_NAME, MASTER_KEY_USER)
        .context("Failed to connect to credential manager")?;

    match entry.get_password() {
        Ok(encoded) => Ok(hex::decode(encoded)?),
        Err(_) => {
            let mut key = vec![0u8; 32];
            getrandom::fill(&mut key).map_err(|e| anyhow::anyhow!("RNG failed: {}", e))?;
            let encoded = hex::encode(&key);
            entry.set_password(&encoded)?;
            Ok(key)
        }
    }
}

pub fn save_api_key(alias: &str, secret: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, alias)?;
    entry.set_password(secret)?;
    Ok(())
}

pub fn get_api_key(alias: &str) -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, alias)?;
    Ok(entry.get_password()?)
}

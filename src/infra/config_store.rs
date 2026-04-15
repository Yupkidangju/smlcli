use crate::domain::settings::PersistedSettings;
use anyhow::Result;
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use std::path::PathBuf;

pub fn save_config(key: &[u8], settings: &PersistedSettings) -> Result<()> {
    let toml_str = toml::to_string(settings)?;

    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("Invalid key length: {}", e))?;

    let mut nonce_bytes = [0u8; 24];
    getrandom::fill(&mut nonce_bytes).map_err(|e| anyhow::anyhow!("RNG failed: {}", e))?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, toml_str.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = config_dir.join("smlcli");
    std::fs::create_dir_all(&app_dir)?;

    let mut file_content = Vec::new();
    file_content.extend_from_slice(&nonce_bytes);
    file_content.extend_from_slice(&ciphertext);

    std::fs::write(app_dir.join("settings.enc"), file_content)?;
    Ok(())
}

pub fn load_config(key: &[u8]) -> Result<Option<PersistedSettings>> {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let path = config_dir.join("smlcli/settings.enc");

    if !path.exists() {
        return Ok(None);
    }

    let data = std::fs::read(path)?;
    if data.len() < 24 {
        return Err(anyhow::anyhow!("Config file is too short/corrupted"));
    }

    let nonce = XNonce::from_slice(&data[0..24]);
    let ciphertext = &data[24..];

    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("Invalid key length: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed (Corrupt or wrong key): {}", e))?;

    let settings: PersistedSettings = toml::from_str(std::str::from_utf8(&plaintext)?)?;
    Ok(Some(settings))
}

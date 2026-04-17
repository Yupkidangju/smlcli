// [v0.1.0-beta.14] 아키텍처 변경: keyring 의존성 완전 제거.
// 마스터 키를 ~/.smlcli/.master_key 파일에 저장 (hex 인코딩, chmod 600).
// API 키는 ChaCha20Poly1305로 암호화하여 config.toml의 encrypted_keys 맵에 보관.
// 이 모듈은 마스터 키 관리 + 값 암호화/복호화만 담당.
// config.toml 읽기/쓰기는 config_store.rs가 담당.

use anyhow::{Context, Result};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use std::path::PathBuf;

/// 설정 디렉토리 경로 반환: ~/.smlcli/
fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".smlcli")
}

/// 마스터 키 파일 경로: ~/.smlcli/.master_key
fn master_key_path() -> PathBuf {
    get_config_dir().join(".master_key")
}

/// 마스터 키를 파일에서 읽거나, 없으면 새로 생성하여 저장.
/// [v0.1.0-beta.19] secrecy::SecretBox<Vec<u8>>을 사용하여 메모리 상의 키 노출을 방지.
pub fn get_or_create_master_key() -> Result<SecretBox<Vec<u8>>> {
    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir).context("~/.smlcli 디렉토리 생성 실패")?;

    let path = master_key_path();

    if path.exists() {
        let encoded = std::fs::read_to_string(&path).context("마스터 키 파일 읽기 실패")?;
        let key =
            hex::decode(encoded.trim()).context("마스터 키 hex 디코딩 실패 (파일 손상 가능)")?;
        if key.len() != 32 {
            return Err(anyhow::anyhow!(
                "마스터 키 길이 불일치: {}바이트 (32바이트 필요)",
                key.len()
            ));
        }
        Ok(SecretBox::new(key.into()))
    } else {
        let mut key = vec![0u8; 32];
        getrandom::fill(&mut key).map_err(|e| anyhow::anyhow!("난수 생성 실패: {}", e))?;
        let encoded = hex::encode(&key);
        std::fs::write(&path, &encoded).context("마스터 키 파일 저장 실패")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .context("마스터 키 파일 권한(600) 설정 실패")?;
        }

        Ok(SecretBox::new(key.into()))
    }
}

/// 평문 값을 암호화하여 "hex_nonce:hex_ciphertext" 형식 반환.
pub fn encrypt_value(master_key: &SecretBox<Vec<u8>>, plaintext: &SecretString) -> Result<String> {
    let cipher = XChaCha20Poly1305::new_from_slice(master_key.expose_secret())
        .map_err(|e| anyhow::anyhow!("암호화 키 길이 오류: {}", e))?;

    let mut nonce_bytes = [0u8; 24];
    getrandom::fill(&mut nonce_bytes).map_err(|e| anyhow::anyhow!("논스 생성 실패: {}", e))?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.expose_secret().as_bytes())
        .map_err(|e| anyhow::anyhow!("암호화 실패: {}", e))?;

    Ok(format!(
        "{}:{}",
        hex::encode(nonce_bytes),
        hex::encode(ciphertext)
    ))
}

/// 암호화된 값을 복호화하여 SecretString으로 반환.
pub fn decrypt_value(master_key: &SecretBox<Vec<u8>>, encrypted: &str) -> Result<SecretString> {
    let parts: Vec<&str> = encrypted.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("암호화된 값 형식 오류"));
    }

    let nonce_bytes = hex::decode(parts[0]).context("논스 hex 디코딩 실패")?;
    let ciphertext = hex::decode(parts[1]).context("암호문 hex 디코딩 실패")?;

    let cipher = XChaCha20Poly1305::new_from_slice(master_key.expose_secret())
        .map_err(|e| anyhow::anyhow!("복호화 키 길이 오류: {}", e))?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_e| anyhow::anyhow!("복호화 실패 (키 불일치 또는 데이터 손상)"))?;

    let secret_str =
        String::from_utf8(plaintext).context("복호화된 값이 유효한 UTF-8이 아닙니다")?;
    Ok(SecretString::new(secret_str.into()))
}

/// API 키를 암호화하여 settings에 저장.
pub fn save_api_key(
    settings: &mut crate::domain::settings::PersistedSettings,
    alias: &str,
    secret: &SecretString,
) -> Result<()> {
    let mk = get_or_create_master_key()?;
    let encrypted = encrypt_value(&mk, secret)?;
    settings.encrypted_keys.insert(alias.to_string(), encrypted);
    Ok(())
}

/// API 키를 복호화하여 반환.
pub fn get_api_key(
    settings: &crate::domain::settings::PersistedSettings,
    alias: &str,
) -> Result<SecretString> {
    let encrypted = settings
        .encrypted_keys
        .get(alias)
        .ok_or_else(|| anyhow::anyhow!("{} API 키가 설정되지 않았습니다.", alias))?;

    let mk = get_or_create_master_key()?;
    decrypt_value(&mk, encrypted)
}

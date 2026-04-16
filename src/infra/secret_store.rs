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
use std::path::PathBuf;

/// 설정 디렉토리 경로 반환: ~/.smlcli/
/// 크로스플랫폼 호환: dirs::home_dir() 사용.
fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".smlcli")
}

/// 마스터 키 파일 경로: ~/.smlcli/.master_key
/// 이 파일은 32바이트 랜덤 키를 hex 인코딩하여 저장.
/// Unix 환경에서는 chmod 600으로 소유자만 읽기 가능하도록 보호.
fn master_key_path() -> PathBuf {
    get_config_dir().join(".master_key")
}

/// 마스터 키를 파일에서 읽거나, 없으면 새로 생성하여 저장.
/// 이 키는 API 키 암호화/복호화에 사용되는 대칭 키 (32바이트, ChaCha20Poly1305).
///
/// 기존 keyring 기반 방식 대비 변경점:
/// - keyring::Entry 대신 파일 시스템 사용 → OS 백엔드 의존성 제거
/// - Linux, Windows, macOS 모두 동일한 동작 보장
pub fn get_or_create_master_key() -> Result<Vec<u8>> {
    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir).context("~/.smlcli 디렉토리 생성 실패")?;

    let path = master_key_path();

    if path.exists() {
        // 기존 마스터 키 로드
        let encoded = std::fs::read_to_string(&path).context("마스터 키 파일 읽기 실패")?;
        let key =
            hex::decode(encoded.trim()).context("마스터 키 hex 디코딩 실패 (파일 손상 가능)")?;
        if key.len() != 32 {
            return Err(anyhow::anyhow!(
                "마스터 키 길이 불일치: {}바이트 (32바이트 필요). 파일이 손상되었을 수 있습니다.",
                key.len()
            ));
        }
        Ok(key)
    } else {
        // 신규 마스터 키 생성
        let mut key = vec![0u8; 32];
        getrandom::fill(&mut key).map_err(|e| anyhow::anyhow!("난수 생성 실패: {}", e))?;
        let encoded = hex::encode(&key);
        std::fs::write(&path, &encoded).context("마스터 키 파일 저장 실패")?;

        // Unix: 소유자만 읽기/쓰기 가능하도록 권한 설정
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .context("마스터 키 파일 권한(600) 설정 실패")?;
        }

        Ok(key)
    }
}

/// 평문 값을 ChaCha20Poly1305로 암호화하여 "hex_nonce:hex_ciphertext" 형식 문자열 반환.
/// master_key: 32바이트 대칭 키 (get_or_create_master_key()에서 획득).
/// plaintext: 암호화할 평문 (API 키 등).
pub fn encrypt_value(master_key: &[u8], plaintext: &str) -> Result<String> {
    let cipher = XChaCha20Poly1305::new_from_slice(master_key)
        .map_err(|e| anyhow::anyhow!("암호화 키 길이 오류: {}", e))?;

    // 24바이트 논스 생성 (매 암호화마다 고유)
    let mut nonce_bytes = [0u8; 24];
    getrandom::fill(&mut nonce_bytes).map_err(|e| anyhow::anyhow!("논스 생성 실패: {}", e))?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("암호화 실패: {}", e))?;

    // "nonce_hex:ciphertext_hex" 형식으로 반환
    Ok(format!(
        "{}:{}",
        hex::encode(nonce_bytes),
        hex::encode(ciphertext)
    ))
}

/// "hex_nonce:hex_ciphertext" 형식 문자열을 복호화하여 평문 반환.
/// master_key: 32바이트 대칭 키.
/// encrypted: encrypt_value()가 생성한 암호화 문자열.
pub fn decrypt_value(master_key: &[u8], encrypted: &str) -> Result<String> {
    let parts: Vec<&str> = encrypted.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "암호화된 값 형식 오류: ':' 구분자가 없습니다. 값이 손상되었을 수 있습니다."
        ));
    }

    let nonce_bytes = hex::decode(parts[0]).context("논스 hex 디코딩 실패")?;
    let ciphertext = hex::decode(parts[1]).context("암호문 hex 디코딩 실패")?;

    if nonce_bytes.len() != 24 {
        return Err(anyhow::anyhow!(
            "논스 길이 오류: {}바이트 (24바이트 필요)",
            nonce_bytes.len()
        ));
    }

    let cipher = XChaCha20Poly1305::new_from_slice(master_key)
        .map_err(|e| anyhow::anyhow!("복호화 키 길이 오류: {}", e))?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("복호화 실패 (키 불일치 또는 데이터 손상): {}", e))?;

    String::from_utf8(plaintext).context("복호화된 값이 유효한 UTF-8이 아닙니다")
}

/// API 키를 암호화하여 settings의 encrypted_keys 맵에 저장.
/// 이 함수는 settings 객체를 수정만 하고, 디스크 저장은 호출자가 config_store::save_config()로 수행.
pub fn save_api_key(
    settings: &mut crate::domain::settings::PersistedSettings,
    alias: &str,
    secret: &str,
) -> Result<()> {
    let mk = get_or_create_master_key()?;
    let encrypted = encrypt_value(&mk, secret)?;
    settings.encrypted_keys.insert(alias.to_string(), encrypted);
    Ok(())
}

/// settings의 encrypted_keys 맵에서 API 키를 읽어 복호화하여 반환.
pub fn get_api_key(
    settings: &crate::domain::settings::PersistedSettings,
    alias: &str,
) -> Result<String> {
    let encrypted = settings.encrypted_keys.get(alias).ok_or_else(|| {
        anyhow::anyhow!(
            "{} API 키가 설정되지 않았습니다. /setting으로 설정하세요.",
            alias
        )
    })?;

    let mk = get_or_create_master_key()?;
    decrypt_value(&mk, encrypted)
}

# Build Guide

이 문서는 `smlcli` 프로젝트에 대한 빌드 및 개발 환경 설정 지침을 제공합니다.

## 시스템 요구사항

- **Rust**: 버킷의 안정적인 `stable` 채널 최신 버전 (Edition 2024 지원 필수)
- **OS**: Linux (Ubuntu, Debian 계열 권장) 및 Windows 10/11 (PowerShell 환경 또는 WSL2 동시 검증용)
- **Linux Sandbox Runtime**: Linux에서 `ExecShell`의 실제 격리를 사용하려면 `bubblewrap`(`bwrap`)가 설치되어 있어야 합니다.

## 프로젝트 빌드 과정

### 1. 의존성 설치 점검 (Linux 기준)
OpenSSL이 빌드 과정 중 C-바인딩을 요구할 수 있습니다. (keyring 의존성은 v0.1.0-beta.14에서 제거됨)
```bash
sudo apt-get update
sudo apt-get install pkg-config libssl-dev libc++-dev bubblewrap
```

### 2. 저장소 준비 및 실행
```bash
# Debug Mode
cargo build
cargo run

# Release Mode
cargo build --release
cargo run --release
```

### 3. CI 및 테스팅 환경 검증
본 프로젝트는 항상 메인 브랜치 병합 전 아래 명령을 성공적으로 통과해야 합니다.
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo audit
```

## 개발 시 주의 사항
- TUI(Text User Interface) 애플리케이션 특성상, 개발 모드에서 오류 패닉이 발생하면 터미널 설정이 깨질 수 있으므로 패닉 핸들러 복원 로직을 중점 확인해야 합니다.
- 크로스 플랫폼 호환성 테스트는 필수이며, 터미널별로 다른 인코딩 및 줄바꿈 문자를 일관성있게 처리해야 합니다.

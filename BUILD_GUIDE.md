# Build Guide

이 문서는 `smlcli` 프로젝트에 대한 빌드 및 개발 환경 설정 지침을 제공합니다.

## 시스템 요구사항

- **Rust**: `1.94.1` (CI/release pinned toolchain, Edition 2024)
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
cargo build --locked
cargo run --locked

# Release Mode
cargo build --release --locked
cargo run --release --locked
```

### 3. CI 및 테스팅 환경 검증
본 프로젝트는 항상 메인 브랜치 병합 전 아래 명령을 성공적으로 통과해야 합니다.
```bash
cargo fmt --check
cargo check --all-targets --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked --no-fail-fast
cargo audit
cargo deny check
bash scripts/check-version-sync.sh
```

### 4. Canonical release targets

- Linux: `x86_64-unknown-linux-musl` on `ubuntu-24.04` with `musl-tools`
- Windows: `x86_64-pc-windows-msvc` on `windows-2025`
- `x86_64-unknown-linux-gnu`와 `x86_64-pc-windows-gnu`는 개발용 비공식 경로이며 release artifact가 아니다.

`main` push와 수동 dispatch는 `.github/workflows/release.yml`에서 두 canonical target의 build/smoke, SHA-256, SPDX 2.3 SBOM 검증, GitHub Sigstore provenance 및 binary↔SBOM attestation을 실행한다. `docs/audit/**`만 바뀐 push는 이 무거운 hosted verification을 건너뛴다. Git tag `v*`는 같은 검증을 통과한 artifact만 GitHub Release에 게시하며, branch/manual 실행은 Release를 만들지 않는다.

각 hosted build job은 생성 직후 provenance와 SPDX SBOM attestation을 GitHub API에서 다시 조회해 workflow identity와 commit SHA까지 검증한다.

검증 예시:

```bash
gh attestation verify ./smlcli-linux-x86_64 -R OWNER/REPOSITORY
python scripts/verify-release-artifacts.py ./downloaded-release
```

### 5. Release rollback

1. 문제가 있는 workflow를 취소하고 해당 GitHub Release를 내려 추가 다운로드를 중단한다.
2. 이미 공개된 tag는 재지정하거나 force-push하지 않는다.
3. 원인을 수정하고 전체 quality/security/cross-target gate를 다시 통과시킨다.
4. 새 PATCH version/tag로 교체 release를 발행하고 이전 release의 영향과 교체 버전을 CHANGELOG에 기록한다.
5. `gh attestation verify`와 checksum 검증이 끝나기 전에는 rollback 완료로 판정하지 않는다.

## 개발 시 주의 사항
- TUI(Text User Interface) 애플리케이션 특성상, 개발 모드에서 오류 패닉이 발생하면 터미널 설정이 깨질 수 있으므로 패닉 핸들러 복원 로직을 중점 확인해야 합니다.
- 크로스 플랫폼 호환성 테스트는 필수이며, 터미널별로 다른 인코딩 및 줄바꿈 문자를 일관성있게 처리해야 합니다.
- local 환경에 canonical target/linker가 없으면 해당 target을 PASS로 기록하지 않고 GitHub hosted runner 결과를 기다립니다.

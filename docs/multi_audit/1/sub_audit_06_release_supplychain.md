# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: A06 — 빌드·패키징·CI/CD·의존성·플랫폼·저장소 위생
- User Goal: 프로젝트의 모든 문서 및 구현내용을 파악한 뒤 전체 문제를 감사하여 안정화와 완성도를 높인다.
- Audit Basis: Standard-backed
- Standard Path: `/mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md`
- Report Contract: `/home/eunho1/.codex/skills/multi-audit/references/report-contract.md`
- Audited Tree: `/mnt/Projects_SSD/rust/smlcli`, branch `main`, HEAD `55d1c33`
- Worktree Note: 감사 시작 시 `AGENTS.md`, `AI_AUDIT_DOC_STANDARD.md`가 수정 상태이고 `AI_CODING_STANDARD.md` 및 `docs/multi_audit/1/audit_run.json`이 미추적 상태였다. 기존 변경은 보존했고, 이 보고서만 새로 작성했다.
- Status Convention: report-contract의 `Confirmed/Probable/Needs Clarification`과 표준의 gate 상태를 함께 표기한다(예: `Confirmed (Needs Fix)`).

## 2. Assigned Scope

다음 배포·빌드·공급망 표면을 현재 트리의 파일, lockfile, 문서, 명령 결과로 대조했다.

- `Cargo.toml`, `Cargo.lock`, `build.rs`, `.cargo/config.toml`
- `build.sh`, `BUILD_GUIDE.md`, `README.md`, `scripts/check-version-sync.sh`
- `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- Rust edition/toolchain, host build, Linux glibc/musl 및 Windows GNU/MSVC target 선언과 설치 상태
- native/system dependency 경로(`arboard`, TLS, `shadow-rs`/`git2`, C build dependencies)
- lockfile 중복 및 로컬 advisory DB 기반 취약성 점검
- 버전·프로젝트명·Phase·tag 동기화
- `scratch.py`, `scratch2.py`, 빈 `.codex`, 추적된 `.agents`/`__pycache__`, `.gemini` 설정, `stitch_modern_tui_redesign` reference HTML/PNG, 외부 대상 symlink의 저장소 위생 및 shipped-scope 위험

## 3. Excluded and Uninspected Scope

- `.git` 내부 객체와 GitHub 원격 API/실제 Actions 실행은 조사하지 않았다. `status`, branch, local tag 목록 등 읽기 전용 메타데이터만 확인했다.
- `docs/multi_audit/1`의 동료 보고서와 `audit_report_*.md` 과거 보고서는 읽지 않았다.
- 프로젝트 `target/`은 사용하지 않았다. host release build는 지정된 `/tmp/smlcli-multi-audit-1-target`로 실행 후 정리했다.
- `build.sh`를 실행하지 않았고, `rustup target add`, apt 설치, MinGW/MSVC/musl 설치, 외부 네트워크 및 유료 검사를 수행하지 않았다.
- `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc` 실제 컴파일과 Windows runtime/수동 QA는 target 및 패키지 캐시 부족으로 미검증이다.
- `cargo deny`는 네트워크 없이 실행했으나 로컬에 필요한 crate/config가 없어 정책 결과를 산출하지 못했다.
- `cargo package`/실제 release upload는 실행하지 않았다. package inclusion은 manifest와 ignore 설정을 정적으로 판정했다.

## 4. Evidence Examined

### 4.1 문서·설정·구현

- `Cargo.toml:1-45`: package `smlcli` `3.9.0`, edition `2024`, direct/build dependencies. `rust-version`, `license`, `description`, `repository`, `include`/`exclude`가 없다.
- `Cargo.lock:1-3,2863-2910`: lockfile v4 및 root package `smlcli 3.9.0`; 모든 registry package checksum은 존재한다.
- `build.rs:1-3`, `src/main.rs:16-18`, `src/infra/doctor.rs:172-179`: `shadow-rs`가 Git short commit과 build time을 바이너리에 주입한다.
- `.cargo/config.toml:1-3`: Windows GNU target에만 `x86_64-w64-mingw32-gcc` linker가 선언되어 있다.
- `build.sh:11-18,22-30,34-59`: Linux GNU와 Windows GNU만 선택하며, Windows 경로에서 `rustup target add`와 `.cargo/config.toml` append를 수행한다.
- `BUILD_GUIDE.md:5-18,31-38`: stable 최신/Edition 2024, Linux·Windows, OpenSSL/libssl-dev/bubblewrap 설치와 fmt/clippy/test/audit 명령을 요구한다.
- `spec.md:1-35,1244-1275,1263-1274,1303-1314`: 현재 version `v3.9.0`, Linux+Windows 목표, QA matrix와 `cargo audit`/`cargo deny` release gate를 명시한다.
- `spec.md:2565-2614`: Phase 45의 Linux musl + Windows MSVC release 성공 기준과 예시 workflow.
- `spec.md:2911-3018`: Phase 53 `v3.9.1`, Phase 54 `v3.9.2` scope와 Windows sandbox parity 비목표.
- `audit_roadmap.md:445-459,636-681`: Phase 45 완료 주장, artifact 이름 기대치, Phase 53/54 감사 기준.
- `IMPLEMENTATION_SUMMARY.md:83-104,1045-1055`: Phase 54 및 Phase 45 완료 주장.
- `README.md:1-46,61-99`: Linux/Windows 지원, `./build.sh` quick start, placeholder clone URL.
- `scripts/check-version-sync.sh:1-53`: Cargo.toml ↔ CHANGELOG 및 `GITHUB_REF_NAME`이 `v*`일 때만 tag 비교.
- `.github/workflows/ci.yml:5-60`, `.github/workflows/release.yml:5-88`: trigger, action refs, permissions, quality gates, target matrix, artifact upload.

### 4.2 저장소 위생·범위 증거

- `.gitignore:1`: `/target` 한 줄만 존재한다.
- 추적 파일 수: `.agents` 56개(832,186 bytes), 그 중 `scripts/__pycache__` `.pyc` 3개; `stitch_modern_tui_redesign` 17개(1,672,198 bytes, HTML/PNG 포함).
- `.codex`: 추적된 0-byte 파일; `.gemini/settings.json:1-9`: `browser_agent` override만 포함.
- `scratch.py:3-42`, `scratch2.py:3-12`: `src/tests/audit_regression.rs`를 정규식/문자열 치환으로 직접 읽고 같은 경로에 덮어쓴다.
- 추적 symlink `.antigravitycli/a64b3a3a-7f50-4544-9fd4-c30d4cfb405f.json`는 `/home/eunho1/.gemini/config/projects/...` 외부 경로를 가리킨다.
- `redesign_plan.md:15`: `stitch_modern_tui_redesign`를 design reference로 언급하지만 manifest의 package scope 선언은 없다.

### 4.3 실행 명령 및 결과

| 명령 | 결과 | 핵심 증거 |
| --- | --- | --- |
| `cargo metadata --format-version 1 --locked --no-deps` | PASS (exit 0) | package `smlcli 3.9.0`, edition `2024`; metadata `license/description/repository`는 `null` |
| `cargo tree --locked --depth 2` / `cargo tree --locked --duplicates` | PASS (exit 0) | `getrandom 0.2.17/0.4.2`, `hashbrown 0.16.1/0.17.0` 등 transitive duplicate 확인; 동일 major duplicate 정책은 없음 |
| `cargo tree --locked --target x86_64-unknown-linux-musl --depth 3` | PASS (exit 0) | `arboard 3.6.1 → x11rb 0.13.2`; `reqwest 0.13.2 → hyper-rustls/rustls` 경로 확인 |
| `cargo fmt --check` | PASS (exit 0) | formatting 오류 없음 |
| `./scripts/check-version-sync.sh` | PASS (exit 0) | Cargo.toml/CHANGELOG 모두 `3.9.0` |
| `GITHUB_REF_NAME=v3.9.0 ./scripts/check-version-sync.sh` | PASS (exit 0) | tag `3.9.0` 비교 통과 |
| `GITHUB_REF_NAME=v3.9.2 ./scripts/check-version-sync.sh` | FAIL (exit 1) | Cargo `3.9.0` ≠ tag `3.9.2`를 감지 |
| `GITHUB_REF_NAME=release-3.9.0 ./scripts/check-version-sync.sh` | PASS (exit 0) | `v*`가 아닌 ref는 tag 비교를 건너뜀 |
| `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo build --release --locked` | PASS (exit 0) | host `x86_64-unknown-linux-gnu`, release 1.94.1에서 `Finished release` 2m24s |
| host artifact `file/readelf/ldd` | PASS (관찰) | ELF PIE, interpreter `/lib64/ld-linux-x86-64.so.2`, glibc 동적 링크; musl 산출물 아님 |
| `cargo audit --no-fetch` | FAIL (exit 1) | local `/home/eunho1/.cargo/advisory-db` 1,166 advisories; 462 dependencies scan; 2 vulnerabilities + 3 allowed warnings |
| `CARGO_NET_OFFLINE=true cargo deny check licenses bans sources` | FAIL (exit 1) | config path 없음(default fallback), `wasm-bindgen`/`bumpalo` offline fetch 불가 |
| `rustup target list --installed` | PASS (관찰) | `wasm32-unknown-unknown`, `x86_64-pc-windows-gnu`, `x86_64-unknown-linux-gnu`만 설치; musl/MSVC 없음 |
| `cargo tree --locked --target x86_64-pc-windows-gnu/msvc` | NOT COVERED (exit 101) | Windows target graph 조회가 `clipboard-win 5.4.1` cache unpack/read-only filesystem에서 중단 |
| `bash -n build.sh scripts/check-version-sync.sh` | PASS (exit 0) | shell syntax 오류 없음 |

`cargo audit --no-fetch`는 네트워크 advisory fetch 없이 로컬 DB만 사용했다. 출력된 advisory 중 직접/빌드 경로는 다음과 같다.

- `RUSTSEC-2026-0204`: `crossbeam-epoch 0.9.18` → `ignore 0.4.25` → `smlcli`, solution `>=0.9.20`.
- `RUSTSEC-2026-0185`: `quinn-proto 0.11.14`의 high(7.5) remote memory exhaustion, `reqwest 0.13.2` lock graph에 포함되며 solution `>=0.11.15`. 현재 default host feature graph에서 runtime reachability는 별도 확인이 필요하다.
- 경고 3건: `anyhow 1.0.102` (`RUSTSEC-2026-0190`), `git2 0.20.4` (`RUSTSEC-2026-0184`, `RUSTSEC-2026-0183`); `git2`는 `shadow-rs 1.7.1` build/runtime 경로다.

## 5. Findings

### Pass 1: Implementation Compliance

### [A06-F005] 프로젝트 identity/version/Phase authority가 분리되어 release tag 검증에 닫히지 않음

- Pass: Implementation
- Pattern: `IMP-003`, `IMP-004`
- Area: 버전·프로젝트명·Phase·artifact naming 정합성
- Severity: Major
- Status: Confirmed (Needs Fix)
- Confidence: High
- Summary: Cargo package와 CHANGELOG는 `smlcli 3.9.0`으로 동기화되어 있지만, 현재 통제 문서와 운영 파일에는 다른 프로젝트 identity/version 및 후속 Phase version이 동시에 존재한다. release workflow는 tag가 package version과 일치하는지 검사하지 않는다.
- Evidence:
  - `Cargo.toml:1-4`, `Cargo.lock:2863-2866`, `spec.md:1,20-35`, `CHANGELOG.md:15`는 `smlcli 3.9.0`을 가리킨다.
  - `AGENTS.md:5-6,25-26`은 현재 저장소를 `doomlike 0.5.1`로 표기한다. 해당 변경은 감사 시작 시 미커밋 상태였으므로, 프로젝트 authority인지 템플릿인지도 닫혀 있지 않다.
  - `spec.md:2911`, `audit_roadmap.md:636`은 Phase 53을 `v3.9.1`, `spec.md:3001`, `audit_roadmap.md:681`은 Phase 54를 `v3.9.2`로 표기하지만 package/README/summary의 current release는 `3.9.0`이다.
  - `audit_roadmap.md:457-459`의 Phase 45 기대 artifact 이름(`smlcli-x86_64-unknown-linux-musl`, `smlcli-x86_64-pc-windows-msvc.exe`)은 실제 workflow의 `smlcli-linux-x86_64`, `smlcli-windows-x86_64.exe`(`release.yml:39-45`)와 다르다.
  - `scripts/check-version-sync.sh:8-41`은 Cargo/CHANGELOG 및 `v*` 환경일 때만 tag를 검사한다. 실제 실행은 `v3.9.0`에서 PASS, `v3.9.2`에서 FAIL했지만, `.github/workflows/release.yml:7-10,18-30`에는 이 script 호출이 없다. local tag 목록은 비어 있어 실제 release tag는 검증하지 못했다.
- Expected Basis: `spec.md:1263-1275,1303-1314`의 release/보안 gate, 표준 `IMP-003/IMP-004`, 사용자 요구의 tag-package-version 충돌 탐지 조건.
- Actual: package version만 3.9.0으로 닫혀 있고 project name, current Phase/version, artifact naming, release tag는 같은 gate에서 검증되지 않는다.
- Impact: 잘못된 tag 또는 stale phase 문서로 다른 버전의 바이너리/릴리스 노트를 publish할 수 있고, 지원·rollback·재감사 대상 식별이 불명확해진다.
- Suggested Action: canonical project name/version/current Phase authority를 하나로 결정하고, 템플릿/과거 Phase 표기는 명시적으로 분리한다. release job에서 `scripts/check-version-sync.sh`를 `GITHUB_REF_NAME`과 함께 실행하고 Cargo metadata, spec heading, current release 문서, artifact naming을 기계적으로 비교한다.
- Re-audit Method: canonical authority 문서와 tag policy를 재확인한 뒤 `GITHUB_REF_NAME=v<package-version>` 및 불일치 tag를 CI에서 실행하고, release matrix artifact 이름과 roadmap 기대치를 다시 대조한다.
- Owner: Architect / Release owner
- Notes: `scripts/check-version-sync.sh` 자체의 불일치 감지는 동작하지만, release workflow 연결이 없어 gate가 아니다.

### Pass 2: Debug / Engineering Quality

### [A06-F002] CI와 release quality gate가 문서의 locked/security/release 검증을 실제로 집행하지 않음

- Pass: Debug
- Pattern: `BUILD-001`, `IMP-003`
- Area: CI trigger, build reproducibility, release gate completeness
- Severity: Major
- Status: Confirmed (Needs Fix)
- Confidence: High
- Summary: CI는 fmt/clippy/test만 실행하고 release workflow도 동일한 부분 gate 뒤에 `cargo build --release`를 실행한다. `--locked`, version-sync, `cargo audit`, `cargo deny`, release-profile test/build 검증이 tag publish 경로에 연결되어 있지 않다.
- Evidence:
  - `BUILD_GUIDE.md:31-38`과 `spec.md:1263-1274,1303-1314`는 fmt, clippy, test, audit, deny, Linux/Windows QA 및 provider/security smoke를 release 전 요구한다.
  - `ci.yml:41-48`은 `cargo fmt --check`, `cargo clippy ...`, `cargo test --all-targets`만 실행하며 `--locked`/audit/deny가 없다. `ci.yml:50-60`의 version-sync job은 branch push/PR trigger(`ci.yml:7-11`)에만 속한다.
  - `release.yml:18-29` quality gate도 fmt/clippy/test만 실행하고, `release.yml:72-73` build는 `cargo build --release --target ...`로 `--locked`가 없다.
  - 정적/host 명령 결과는 fmt와 locked host release build가 PASS였지만, 이는 tag workflow의 누락 gate를 보완하지 않는다. `cargo audit --no-fetch`는 실제로 exit 1이다(A06-F001).
- Expected Basis: `BUILD-001`, `IMP-003`, `spec.md`의 Release Gate와 사용자 요구의 CI trigger/gates/lock/audit 검증.
- Actual: committed lockfile과 다른 dependency resolution이 CI에서 허용될 수 있고, advisory/license/policy 검사 없이 tag upload 단계로 진행할 수 있다. release profile 자체는 quality gate에서 사전 검증되지 않는다.
- Impact: 로컬 PASS와 GitHub release 결과가 달라질 수 있으며, 취약 dependency 또는 미검증 release build가 자동 게시될 수 있다. Phase 45 “완료” 주장을 현재 release readiness로 해석할 수 없다.
- Suggested Action: 모든 cargo gate에 `--locked`를 적용하고 release workflow에 version-sync, `cargo audit --no-fetch` 또는 CI advisory policy, `cargo deny check`, release-profile build/test를 명시한다. Windows/Linux manual QA와 provider/security smoke의 자동화 범위를 문서에 맞춰 닫거나 미검증으로 표시한다.
- Re-audit Method: workflow YAML과 branch/tag trigger를 다시 정적 검증하고, isolated target directory에서 `fmt`, `clippy`, `test`, `build --release --locked`, audit/deny gate가 동일 lockfile을 사용하는지 확인한다.
- Owner: Release owner / Coder
- Notes: `cargo deny`의 현재 로컬 실행은 config 및 offline package 부족으로 별도 미검증이며, 이것도 gate coverage gap이다.

### [A06-F004] 문서·스크립트·release matrix의 Windows/Linux target과 native dependency 계약이 서로 다름

- Pass: Debug
- Pattern: `BUILD-001`, `DEP-001`
- Area: cross compile, glibc/musl, Windows GNU/MSVC, native/system dependencies
- Severity: Major
- Status: Confirmed (Needs Fix)
- Confidence: High
- Summary: 공식 release는 Linux musl + Windows MSVC를 선언하지만 로컬 build script는 Linux GNU + Windows GNU를 제공한다. 현재 host에서 재현한 산출물은 glibc 동적 ELF이며 musl/MSVC target은 설치·실행 검증이 되지 않았다. 의존성 문서도 실제 lock graph와 어긋난다.
- Evidence:
  - `spec.md:34-35,90,1244-1250`은 Linux+Windows 동등 지원과 Windows QA matrix를 요구한다.
  - `release.yml:37-45`는 `x86_64-unknown-linux-musl`와 `x86_64-pc-windows-msvc`를 사용한다. 반면 `build.sh:15-17,45-59`는 Windows `x86_64-pc-windows-gnu`, MinGW linker, `rustup target add`를 사용한다. `.cargo/config.toml:2-3`도 GNU target만 설정한다.
  - `rustup target list --installed` 결과는 native, Windows GNU, wasm만 포함하고 musl/MSVC는 없다. `cargo tree --target x86_64-pc-windows-gnu/msvc`는 `clipboard-win 5.4.1`을 read-only registry cache에 unpack하지 못해 exit 101이었다. target 설치/네트워크는 의도적으로 수행하지 않았다.
  - 허용된 host build는 `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo build --release --locked`로 PASS했지만, `file/readelf/ldd` 결과는 `/lib64/ld-linux-x86-64.so.2` interpreter와 glibc dynamic linkage를 보여준다. musl 정적 artifact의 증거가 아니다.
  - `cargo tree --target x86_64-unknown-linux-musl`는 `arboard 3.6.1 → x11rb 0.13.2`, `reqwest 0.13.2 → hyper-rustls/rustls`를 보였다. `Cargo.lock`에는 `aws-lc-sys:164-176`, `libgit2-sys:1609-1617`, `libz-sys:1630-1638`, `clipboard-win:375-383` 등 target/build native 경로가 있다.
  - `BUILD_GUIDE.md:13-18`은 OpenSSL/libssl-dev를 요구하지만 lock graph에는 `openssl-sys`가 없고 TLS runtime은 rustls 계열이다. 실제 C build/native prerequisites와 Windows clipboard dependency가 문서에 완전하게 매핑되지 않는다.
  - `rust-toolchain`/`rust-toolchain.toml`은 없고, CI는 `dtolnay/rust-toolchain@stable`(`ci.yml:25-28`, `release.yml:51-54`)을 사용한다. Cargo.toml에도 `rust-version`이 없다.
- Expected Basis: `BUILD-001`, `DEP-001`, `spec.md:45-66,1244-1274,2565-2614`, 사용자 요구의 glibc/musl 및 GNU/MSVC 재현성 검증.
- Actual: host glibc build만 재현되며 release target과 build.sh target이 다르고, target-specific package/native prerequisites 및 toolchain pin이 닫혀 있지 않다.
- Impact: “Linux/Windows 지원”과 “musl/MSVC release artifact”를 현재 증거로 보장할 수 없다. 잘못된 linker/target을 선택하거나 CI에서만 다운로드 가능한 native package에 의존할 수 있다.
- Suggested Action: 공식 Windows target(MSVC 또는 GNU)을 하나로 결정하고 build.sh/release/spec/README를 같은 matrix로 통일한다. Rust toolchain과 minimum `rust-version`을 pin하고, target별 prerequisite(`musl-tools`, MinGW/MSVC, clipboard/TLS/native C toolchain)를 문서 및 CI에 명시한다. release build는 `--locked`로 실행한다.
- Re-audit Method: 승인된 환경에서 musl/MSVC target을 실제 설치·컴파일하고 `file/ldd` 또는 Windows binary inspection, artifact naming, smoke test를 수행한다. 문서된 native dependencies와 `cargo tree --target all`를 다시 대조한다.
- Owner: Architect / Release owner
- Notes: 현재 미검증은 환경·권한 경계로 인한 것이며 source compile failure로 단정하지 않는다. 다만 release readiness는 `HOLD`다.

### Pass 3: Security / Supply Chain

### [A06-F001] lockfile advisory scan이 현재 release dependency graph의 취약성을 보고함

- Pass: Security
- Pattern: `SEC-006`, `DEP-001`
- Area: dependency advisory, lockfile vulnerability, scanner provenance
- Severity: Major
- Status: Confirmed (Needs Fix)
- Confidence: High for lockfile findings; Medium for default-runtime reachability of the optional/target `quinn` path
- Summary: 네트워크 없이 로컬 advisory DB로 `Cargo.lock` 462개 dependency를 검사한 결과 2개 vulnerability와 3개 unsoundness warning이 보고되었고 exit code는 1이다. 프로젝트에는 `deny.toml` 또는 audit policy/exception 문서가 없다.
- Evidence:
  - `cargo audit --no-fetch`는 `/home/eunho1/.cargo/advisory-db`에서 1,166 advisories를 로드했고 `RUSTSEC-2026-0204` (`crossbeam-epoch 0.9.18`, `ignore 0.4.25` 경유)와 `RUSTSEC-2026-0185` (`quinn-proto 0.11.14`, high 7.5, `reqwest` lock graph 경유)를 보고하여 `error: 2 vulnerabilities found`, exit 1을 반환했다.
  - 같은 scan은 `anyhow 1.0.102` (`RUSTSEC-2026-0190`)와 `git2 0.20.4` (`RUSTSEC-2026-0184`, `RUSTSEC-2026-0183`)를 allowed warning으로 보고했다. `cargo tree --locked -i git2`는 `git2 → shadow-rs → smlcli` build/runtime 경로를 확인했다.
  - `cargo tree --locked --duplicates`는 `getrandom 0.2.17/0.4.2`, `hashbrown 0.16.1/0.17.0` 등 여러 transitive version duplicate를 보여주지만, duplicate 자체는 자동 취약점 판정이 아니다.
  - `CARGO_NET_OFFLINE=true cargo deny check licenses bans sources`는 `deny.toml` 부재로 default config에 fallback한 뒤 `wasm-bindgen`/`bumpalo` offline fetch 오류로 exit 1이었다. 따라서 license/ban/source 정책 결과는 Not Covered다.
- Expected Basis: `spec.md:1263-1274,1303-1314`의 audit/deny release gate, 표준 `SEC-006`의 shipped scope와 scanner provenance 조건, 사용자 요구의 local-only advisory 검증.
- Actual: 현재 lockfile에는 unresolved vulnerability가 있고, release workflow에는 audit/deny gate가 없으며, optional/target dependency advisory를 어떻게 triage할지 정책도 없다.
- Impact: 취약 lockfile을 가진 build가 자동 release에 도달할 수 있다. 특히 `RUSTSEC-2026-0185`는 high remote-memory-exhaustion advisory이나 현재 default host feature에서의 실제 reachability는 별도 triage 전까지 확정하지 않는다.
- Suggested Action: advisory별 direct/transitive·target/feature reachability를 분류하고 해결 가능한 dependency를 update/patch한다. 해결 불가 항목은 owner, rationale, expiry와 release block 조건을 policy 문서에 기록한다. `cargo audit --no-fetch`와 `cargo deny`를 locked CI/release gate로 연결하고 license/source allowlist 및 shipped scope를 선언한다.
- Re-audit Method: 동일 lockfile에서 `cargo audit --no-fetch`, `cargo tree -i`/`--target all`, configured `cargo deny check advisories bans licenses sources`를 재실행하고 exit 0 또는 명시적 만료 exception을 확인한다.
- Owner: Dependency owner / Release owner
- Notes: advisory DB는 로컬에서만 읽었다. 외부 advisory network refresh는 이 감사 범위에서 제외했다.

### [A06-F003] release artifact provenance, integrity metadata, action pinning, least privilege가 없음

- Pass: Security
- Pattern: `SEC-006`, `BUILD-001`
- Area: release artifact, checksum/SBOM/signing, action/toolchain provenance, permissions, rollback
- Severity: Major
- Status: Confirmed (Needs Fix)
- Confidence: High
- Summary: release workflow는 bare Linux/Windows binary만 업로드하며 checksum, SBOM, signature/attestation, license manifest, provenance, rollback 절차를 만들지 않는다. Actions/toolchain은 mutable tag이고 workflow-level `contents: write`가 quality/build job까지 적용된다.
- Evidence:
  - `release.yml:14-15`가 전체 workflow에 `contents: write`를 부여한다. `quality-gate`와 matrix build에는 upload가 필요하지 않지만 별도 job-level read 권한이 없다.
  - `ci.yml:23,26,31`, `release.yml:23-24,49,52,62,85`는 `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `actions/cache@v4`, `softprops/action-gh-release@v2` mutable ref를 사용하며 commit SHA pin이 없다.
  - `release.yml:72-88`은 target binary를 rename하여 바로 upload할 뿐 SHA-256 checksum, SBOM/SPDX/CycloneDX, cosign/signature, SLSA provenance, license report, retention/rollback artifact를 생성하지 않는다. 저장소 검색에서도 `sha256sum`, `SBOM`, `cosign`, `attest`, `cargo-deny`, `cargo-auditable` release step이 없다.
  - `Cargo.toml:1-45` 및 `cargo metadata --locked --no-deps`에는 `license`, `description`, `repository`, `authors`, `categories`, `keywords`가 null/미선언이다. `LICENSE`는 Apache 2.0이지만 package metadata로 연결되어 있지 않다.
  - `build.rs:1-3`와 `src/infra/doctor.rs:175-178`는 `BUILD_TIME`을 바이너리에 출력한다. `SOURCE_DATE_EPOCH` 또는 reproducible-build 정책이 없어 동일 source/lockfile의 byte-identical artifact를 보장하지 않는다.
- Expected Basis: 사용자 요구의 checksum/SBOM/signing/license/package metadata/rollback/least privilege 평가, 표준 `SEC-006`, 일반적인 release supply-chain provenance 불변조건.
- Actual: 사용자는 다운로드한 artifact의 출처·무결성·구성요소·서명·rollback 가능성을 독립적으로 검증할 수 없고, mutable action/toolchain 변경과 write token 범위도 제한되지 않는다.
- Impact: CI action 또는 dependency 공급망 변경이 감지·검증되지 않은 채 release에 반영될 수 있다. release 장애 시 이전 artifact로 안전하게 복귀할 계약도 없다.
- Suggested Action: Actions와 toolchain을 승인된 immutable SHA/version으로 pin하고 job별 최소 권한을 적용한다. 각 target에 checksum, SBOM/license inventory, 서명/attestation과 source revision/package version을 함께 생성·게시하고 release immutability/rollback/retention 절차를 문서화한다. Cargo manifest metadata를 채우고 build timestamp의 reproducibility 정책을 정한다.
- Re-audit Method: workflow diff에서 SHA pin/permissions를 확인하고, clean locked build에서 binary, checksum, SBOM, signature/attestation, version/source hash를 검증한 뒤 staged rollback rehearsal 결과를 기록한다.
- Owner: Release owner / Security owner
- Notes: 현재 workflow가 binary를 실제로 GitHub에 upload하는지 live run은 실행하지 않았지만, 정적 workflow상 해당 controls가 부재한 것은 Confirmed다.

### [A06-F006] generated/local/reference 파일이 추적되고 package/shipped scope가 닫혀 있지 않음

- Pass: Security
- Pattern: `SEC-006`, `IMP-004`
- Area: repository hygiene, shipped scope, local configuration/privacy
- Severity: Minor
- Status: Confirmed (Needs Fix)
- Confidence: High
- Summary: `.gitignore`가 `/target`만 제외하므로 local tool configuration, bytecode cache, one-off scripts, design reference corpus, 외부 경로 symlink가 repository에 포함되어 있다. 현재 값에서 API token 등 평문 secret은 확인되지 않았지만, 추적 범위와 package inclusion 정책이 명시되지 않았다.
- Evidence:
  - `.gitignore:1`은 `/target`만 제외한다. `Cargo.toml`에도 `include`/`exclude`가 없다.
  - `.agents` 56개(832,186 bytes)가 추적되고 `.agents/skills/ui-ux-pro-max/scripts/__pycache__/core.cpython-314.pyc`, `design_system.cpython-314.pyc`, `search.cpython-314.pyc` 3개가 포함된다.
  - `.codex`는 0-byte 추적 파일이고 `.gemini/settings.json:1-9`는 local `browser_agent` override를 저장한다.
  - `stitch_modern_tui_redesign`에는 17개, 1,672,198 bytes의 reference HTML/PNG가 추적된다. `redesign_plan.md:15`가 design reference임을 말하지만 release/package에서 제외할지 선언하지 않는다.
  - `.antigravitycli/...json` 추적 symlink는 `/home/eunho1/.gemini/config/projects/...` 외부 경로를 가리킨다. symlink target 내용이나 secret은 읽지 않았지만 clone/package portability와 local path privacy 위험이 있다.
- Expected Basis: 표준 `SEC-006`의 shipped/non-shipped scope 분리 및 사용자 요구의 `.codex`/`.agents`/`.gemini`/reference artifact hygiene 분류.
- Actual: release workflow는 binary만 upload하지만 source archive/package와 repository checkout의 scope는 `include`/`exclude` 또는 ignore policy로 제한되지 않는다. 무엇이 의도적 제품 자산인지 generated/local residue인지 authority가 없다.
- Impact: clone/package/source archive가 불필요한 도구 데이터와 사용자 환경 흔적을 배포할 수 있고, generated bytecode/reference 변경이 품질 검토를 오염시킨다. 현재 평문 secret 노출은 증명되지 않았다.
- Suggested Action: local/generated data와 reference-only corpus를 non-shipped로 분류하고 ignore/exclude한다. `.pyc`, empty `.codex`, `.gemini` local override, 외부 symlink와 scratch residue를 repository policy에 맞게 정리하거나 명시적 이유와 ownership을 기록한다. `Cargo.toml` package scope를 explicit `include`/`exclude`로 닫는다.
- Re-audit Method: `git ls-files`와 `cargo package --list`를 isolated temp target에서 재확인하고, shipped/non-shipped inventory, secret scan, symlink portability 검사를 통과시킨다.
- Owner: Repository owner / Release owner
- Notes: reference HTML/PNG 자체의 UI 내용은 A06 범위가 아니며, 여기서는 추적·배포 범위만 판정했다.

### [A06-F007] scratch.py/scratch2.py가 source를 직접 덮어쓰는 비가역 일회성 도구로 남아 있음

- Pass: Debug
- Pattern: `BUILD-001`, `IMP-004`
- Area: maintenance tooling, accidental source rewrite, repository hygiene
- Severity: Minor
- Status: Confirmed (Needs Fix)
- Confidence: High
- Summary: 루트의 추적된 scratch scripts는 audit regression test source를 backup/dry-run/범위 검증 없이 in-place rewrite한다. CI나 문서에서 호출되지는 않지만 repository 사용자가 실수로 실행할 수 있는 maintenance hazard다.
- Evidence:
  - `scratch.py:3-4`가 `src/tests/audit_regression.rs`를 읽고 `scratch.py:36-38`의 `re.sub(..., flags=re.DOTALL)`로 넓은 패턴을 치환한 뒤 `scratch.py:40-42`에서 같은 파일에 덮어쓴다.
  - `scratch2.py:3-4,7-12`도 같은 source를 읽어 문자열 치환 후 backup 없이 덮어쓴다.
  - repository-wide search에서 두 파일을 호출하는 CI/build/documentation 경로는 확인되지 않았다.
- Expected Basis: `BUILD-001`의 재현 가능한 build tooling과 `IMP-004`의 stale/one-off artifact 독립 감사 기준.
- Actual: source rewrite의 입력/출력 diff, backup, confirmation, deterministic fixture, owner가 없다.
- Impact: 잘못된 실행 또는 regex overmatch가 테스트 source를 손상시키고 이후 build/audit 증거를 오염시킬 수 있다.
- Suggested Action: 일회성 변환은 삭제하거나 별도 non-shipped tool로 이동하고, 필요하다면 read-only dry-run, explicit output path, backup/patch output, fixture test를 제공한다.
- Re-audit Method: `git ls-files`와 build/CI search에서 shipped 호출이 제거되었는지 확인하고, 변환 도구가 source in-place write 없이 deterministic patch를 생성하는지 검증한다.
- Owner: Repository owner / Coder
- Notes: 감사 중 두 script는 실행하지 않았다.

## 6. Cross-Pass Conflicts

### [A06-XPF-001] “Phase 45/53/54 완료” 문서 주장과 실제 release gate 상태의 충돌

- Related Findings: A06-F001, A06-F002, A06-F004, A06-F005
- Conflict: `IMPLEMENTATION_SUMMARY.md:83-104,1045-1055`, `audit_roadmap.md:445-459,636-681`은 CI/CD 및 Phase 53/54를 완료로 기록한다. 그러나 `cargo audit --no-fetch`는 exit 1이고, release workflow는 audit/deny/tag-sync/locked build를 실행하지 않으며, musl/MSVC artifact는 미검증이다.
- Resolution: 문서의 “implemented”와 “release-ready”를 분리한다. A06 범위의 현재 판정은 implementation evidence 일부는 존재하지만 Build/Supply-chain gate가 닫히지 않았으므로 `HOLD`다.
- Gate Impact: Major finding 해소 및 cross-target evidence 전까지 PASS 계열 판정을 금지한다.
- Required Fix Before PASS: A06-F001~F005의 gate controls와 canonical authority를 해결하고 해당 pass를 재감사한다.

## 7. Required Fixes Before PASS

1. A06-F001: 취약 advisory와 `git2`/`anyhow` warning을 direct/transitive·target/feature별로 triage하고 update/patch/만료 exception을 기록한다.
2. A06-F002: CI/release에 `--locked`, version-sync, audit/deny, release-profile build/test와 필요한 Windows/Linux smoke gate를 연결한다.
3. A06-F003: immutable action/toolchain pin, job-level least privilege, checksum/SBOM/license inventory/signature/attestation, version/source provenance 및 rollback policy를 추가한다.
4. A06-F004: 공식 Windows target을 MSVC 또는 GNU 중 하나로 고정하고 musl/MSVC 실제 build와 native prerequisite 문서를 닫는다.
5. A06-F005: `smlcli 3.9.0`과 `doomlike 0.5.1`, Phase 53/54의 `3.9.1/3.9.2`, artifact 이름의 authority를 정리하고 tag gate에 연결한다.
6. A06-F006~F007: generated/local/reference/scratch scope를 명시하고 ignore/exclude 또는 제거·격리한다.

## 8. Accepted and Remaining Risks

현재 A06에서 명시적으로 수용된 위험은 없다. 다음은 `Accepted Risk`가 아니라 재감사 전까지 남는 미검증 경계다.

- `quinn-proto` advisory는 lock graph에서 확인되었으나 default host feature의 실제 runtime reachability는 `cargo tree --target all` 및 feature triage가 필요하다.
- musl/MSVC target 설치와 실제 compile은 사용자 금지 범위(설치/네트워크)로 실행하지 않았다.
- `cargo deny` 결과는 local config/cache 부족으로 `Not Covered`다.
- 실제 GitHub runner, action SHA, release upload race/rollback은 live run 없이 정적 판정했다.
- 현재 local tag 목록이 비어 있어 실제 tag-to-package release 검증은 남아 있다.
- reference/local files가 의도적으로 source distribution에 포함되는지 명세가 없어 shipped scope clarification이 필요하다.

## 9. Needs Spec Clarification

- 현재 canonical project identity/version은 `smlcli 3.9.0`인가, `doomlike 0.5.1`은 폐기된 템플릿인가?
- Phase 53/54의 `v3.9.1`/`v3.9.2`가 package release version인지, 단순 internal Phase label인지?
- Windows 공식 target은 release의 MSVC인가, `build.sh`의 GNU인가? 각 target의 QA와 native dependency owner는 누구인가?
- Linux release는 glibc와 musl 중 어느 artifact를 supported baseline으로 보장하는가?
- checksum/SBOM/signing/attestation/license manifest/rollback의 필수 형식과 보존 기간은 무엇인가?
- `.agents`, `.codex`, `.gemini`, `stitch_modern_tui_redesign`, scratch scripts, 외부 symlink가 source/package/release에 포함되는가?

## 10. Re-audit Checklist

- canonical name/version/Phase/tag authority를 문서와 CI에서 같은 값으로 확인한다.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, `cargo build --release --locked`를 isolated target directory에서 실행한다.
- configured `cargo audit --no-fetch`와 `cargo deny check advisories bans licenses sources`를 같은 lockfile로 실행하고 advisory/exception ledger를 확인한다.
- musl 및 공식 Windows target을 실제 compile하고 artifact type, linker, dependency prerequisite, smoke/QA를 확인한다.
- release workflow의 action/toolchain immutable pin, job-level permissions, tag guard, artifact naming, checksum/SBOM/signature/attestation/provenance/rollback을 정적·실행 증거로 확인한다.
- `git ls-files`, package list, ignore/exclude, symlink, `.pyc`/local settings/reference scope 및 scratch source-write 경로를 재검사한다.

## 11. Perspective Decision

`HOLD` — A06-F001~F005의 Major 또는 Major-impact gate가 남아 있고, musl/MSVC 실제 release artifact와 `cargo deny` 결과가 미검증이다. host-only locked build와 fmt/version-sync PASS는 프로젝트 전체 또는 release readiness PASS를 의미하지 않는다.

## 12. Coder Handoff

```text
`/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_06_release_supplychain.md`를 먼저 읽고, 각 finding을 현재 프로젝트 문서·manifest·workflow·실제 명령 결과에 대조하여 우선순위대로 수정하세요. 계약 변경이 필요하면 canonical version/target/package scope를 관련 문서에 먼저 갱신하고, 수정 후 locked build·advisory/license policy·cross-target artifact·release provenance 증거를 기록한 뒤 A06 관련 pass를 재감사하세요.
```

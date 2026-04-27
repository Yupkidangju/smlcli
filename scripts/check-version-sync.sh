#!/usr/bin/env bash
# [v3.5.0] Phase 45 Task CI-3: 버전 동기화 검증 스크립트.
# Cargo.toml ↔ CHANGELOG.md 버전 일치를 CI에서 자동 검증합니다.
# 실패 시 종료 코드 1을 반환하여 CI 게이트를 차단합니다.

set -euo pipefail

# 1) Cargo.toml에서 버전 추출
CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ -z "$CARGO_VERSION" ]; then
    echo "❌ Cargo.toml에서 버전을 추출할 수 없습니다."
    exit 1
fi
echo "📦 Cargo.toml 버전: $CARGO_VERSION"

# 2) CHANGELOG.md에서 최신 버전 헤더 추출
# "## [X.Y.Z]" 형태의 첫 번째 헤더를 찾습니다.
CHANGELOG_VERSION=$(grep -oP '(?<=## \[)\d+\.\d+\.\d+' CHANGELOG.md | head -1)
if [ -z "$CHANGELOG_VERSION" ]; then
    echo "❌ CHANGELOG.md에서 버전 헤더를 찾을 수 없습니다."
    exit 1
fi
echo "📋 CHANGELOG.md 최신 버전: $CHANGELOG_VERSION"

# 3) 버전 일치 검증
ERRORS=0

if [ "$CARGO_VERSION" != "$CHANGELOG_VERSION" ]; then
    echo "❌ 버전 불일치: Cargo.toml ($CARGO_VERSION) ≠ CHANGELOG.md ($CHANGELOG_VERSION)"
    ERRORS=$((ERRORS + 1))
fi

# 4) 태그 환경에서 실행 시 태그 버전도 검증
if [ -n "${GITHUB_REF_NAME:-}" ] && [[ "${GITHUB_REF_NAME}" == v* ]]; then
    TAG_VERSION="${GITHUB_REF_NAME#v}"
    echo "🏷️  Git 태그 버전: $TAG_VERSION"
    
    if [ "$CARGO_VERSION" != "$TAG_VERSION" ]; then
        echo "❌ 버전 불일치: Cargo.toml ($CARGO_VERSION) ≠ Git Tag ($TAG_VERSION)"
        ERRORS=$((ERRORS + 1))
    fi
fi

# 5) 결과 출력
if [ "$ERRORS" -gt 0 ]; then
    echo ""
    echo "❌ 버전 동기화 검증 실패 ($ERRORS건의 불일치)"
    echo "   Cargo.toml, CHANGELOG.md의 버전을 동기화한 후 다시 시도하세요."
    exit 1
else
    echo ""
    echo "✅ 버전 동기화 검증 통과"
fi

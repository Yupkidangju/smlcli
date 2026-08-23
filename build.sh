#!/usr/bin/env bash
set -euo pipefail

readonly RUST_TOOLCHAIN="1.94.1"
readonly LINUX_TARGET="x86_64-unknown-linux-musl"
readonly WINDOWS_TARGET="x86_64-pc-windows-msvc"

build_linux_release() {
    if [ "$(uname -s)" != "Linux" ]; then
        echo "Linux musl release must be built on Linux." >&2
        return 1
    fi
    command -v musl-gcc >/dev/null || {
        echo "musl-gcc is required (Ubuntu: sudo apt install musl-tools)." >&2
        return 1
    }
    rustup target add "$LINUX_TARGET" --toolchain "$RUST_TOOLCHAIN"
    cargo "+$RUST_TOOLCHAIN" build --release --locked --target "$LINUX_TARGET"
    echo "target/$LINUX_TARGET/release/smlcli"
}

build_windows_release() {
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) ;;
        *)
            echo "Windows MSVC release must be built in a Visual Studio developer environment or the pinned GitHub Windows runner." >&2
            return 1
            ;;
    esac
    rustup target add "$WINDOWS_TARGET" --toolchain "$RUST_TOOLCHAIN"
    cargo "+$RUST_TOOLCHAIN" build --release --locked --target "$WINDOWS_TARGET"
    echo "target/$WINDOWS_TARGET/release/smlcli.exe"
}

build_host_development() {
    echo "Host build is for development only; it is not a canonical release artifact."
    cargo "+$RUST_TOOLCHAIN" build --release --locked
}

echo "smlcli pinned build ($RUST_TOOLCHAIN)"
echo "1) Canonical Linux musl release"
echo "2) Canonical Windows MSVC release"
echo "3) Host development build"
echo "4) Exit"
read -r -p "Enter your choice [1-4]: " choice

case "$choice" in
    1) build_linux_release ;;
    2) build_windows_release ;;
    3) build_host_development ;;
    4) exit 0 ;;
    *) echo "Invalid choice." >&2; exit 1 ;;
esac

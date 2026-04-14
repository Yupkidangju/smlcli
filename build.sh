#!/bin/bash

# smlcli Cross-Platform Build Script

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}==============================================${NC}"
echo -e "${GREEN}   smlcli Interactive Build Script (v0.1.0)  ${NC}"
echo -e "${CYAN}==============================================${NC}"
echo "Select the target platform to build (Release):"
echo "1) Linux Native (x86_64-unknown-linux-gnu)"
echo "2) Windows (x86_64-pc-windows-gnu via MinGW-w64)"
echo "3) Both (Linux & Windows)"
echo "4) Exit"
echo -e "${CYAN}==============================================${NC}"
read -p "Enter your choice [1-4]: " choice

setup_windows_cross_compiler() {
    mkdir -p .cargo
    if [ ! -f ".cargo/config.toml" ] || ! grep -q "x86_64-pc-windows-gnu" ".cargo/config.toml"; then
        echo -e "${YELLOW}[*] Configuring .cargo/config.toml for MinGW-w64 linker...${NC}"
        cat <<EOF >> .cargo/config.toml

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
EOF
    fi
}

build_linux() {
    echo -e "${YELLOW}[*] Building Native Linux Target...${NC}"
    cargo build --release
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}[+] Linux build successful! Binary located at target/release/smlcli${NC}"
    else
        echo -e "${RED}[-] Linux build failed.${NC}"
        exit 1
    fi
}

build_windows() {
    echo -e "${YELLOW}[*] Checking rustup target for Windows...${NC}"
    rustup target add x86_64-pc-windows-gnu
    
    setup_windows_cross_compiler

    echo -e "${YELLOW}[*] Building Windows Target (MinGW-w64)...${NC}"
    cargo build --release --target x86_64-pc-windows-gnu
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}[+] Windows build successful! Binary located at target/x86_64-pc-windows-gnu/release/smlcli.exe${NC}"
    else
        echo -e "${RED}[-] Windows build failed.${NC}"
        echo -e "${YELLOW}Hint: Ensure mingw-w64 is fully installed (e.g., sudo apt install mingw-w64)${NC}"
        exit 1
    fi
}

echo ""

case $choice in
    1)
        build_linux
        ;;
    2)
        build_windows
        ;;
    3)
        build_linux
        echo ""
        build_windows
        ;;
    4)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid choice!${NC}"
        exit 1
        ;;
esac

echo -e "${CYAN}==============================================${NC}"
echo -e "${GREEN} Build process completed successfully!${NC}"
echo -e "${CYAN}==============================================${NC}"

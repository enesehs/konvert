#!/usr/bin/env bash
# ==============================================================================
#  Konvert - Universal One-Line Installer
#  Host: https://enesehs.dev/konvert/install.sh
#  Usage: curl -sSL https://enesehs.dev/konvert/install.sh | bash
# ==============================================================================

set -euo pipefail

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

REPO="enesehs/konvert"
INSTALL_DIR="${HOME}/.local/bin"
TEMP_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "${TEMP_DIR}"
}
trap cleanup EXIT

echo -e "${CYAN}${BOLD}"
cat << "EOF"
  _  __                          _   
 | |/ /___  _ ____   _____ _ __ | |_ 
 | ' // _ \| '_ \ \ / / _ \ '__|| __|
 | . \ (_) | | | \ V /  __/ |   | |_ 
 |_|\_\___/|_| |_|\_/ \___|_|    \__|
EOF
echo -e " Native File Converter for Linux & KDE Plasma${RESET}"
echo -e " --------------------------------------------------\n"

# 1. OS & Architecture Check
OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "${OS}" != "Linux" ]; then
    echo -e "${RED}Error: Konvert only supports Linux operating systems.${RESET}"
    exit 1
fi

if [ "${ARCH}" != "x86_64" ] && [ "${ARCH}" != "aarch64" ]; then
    echo -e "${RED}Error: Unsupported architecture: ${ARCH}.${RESET}"
    exit 1
fi

echo -e "==> 1. Checking system directories..."
mkdir -p "${INSTALL_DIR}"
mkdir -p "${HOME}/.local/share/applications"
mkdir -p "${HOME}/.local/share/metainfo"
mkdir -p "${HOME}/.local/share/icons/hicolor/scalable/apps"

# PATH check
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo -e "${YELLOW}Notice: ${INSTALL_DIR} is not currently in your PATH.${RESET}"
    echo -e "Add this to your shell profile (~/.bashrc or ~/.zshrc): export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

# 2. Download / Install Binaries
echo -e "==> 2. Fetching Konvert release package..."

DOWNLOAD_SUCCESS=false
RELEASE_URL="https://github.com/${REPO}/releases/latest/download/konvert-linux-${ARCH}.tar.gz"

if command -v curl >/dev/null 2>&1; then
    if curl -sSL -f -o "${TEMP_DIR}/konvert.tar.gz" "${RELEASE_URL}" 2>/dev/null; then
        DOWNLOAD_SUCCESS=true
    fi
elif command -v wget >/dev/null 2>&1; then
    if wget -q -O "${TEMP_DIR}/konvert.tar.gz" "${RELEASE_URL}" 2>/dev/null; then
        DOWNLOAD_SUCCESS=true
    fi
fi

if [ "${DOWNLOAD_SUCCESS}" = true ]; then
    echo -e "${GREEN}✓ Release package downloaded successfully.${RESET}"
    tar -xzf "${TEMP_DIR}/konvert.tar.gz" -C "${TEMP_DIR}"
    
    find "${TEMP_DIR}" -type f -name "konvert" -exec cp {} "${INSTALL_DIR}/konvert" \;
    find "${TEMP_DIR}" -type f -name "konvert-settings" -exec cp {} "${INSTALL_DIR}/konvert-settings" \;
    find "${TEMP_DIR}" -type f -name "io.github.enesehs.konvert.desktop" -exec cp {} "${HOME}/.local/share/applications/" \;
    find "${TEMP_DIR}" -type f -name "io.github.enesehs.konvert.metainfo.xml" -exec cp {} "${HOME}/.local/share/metainfo/" \;
    find "${TEMP_DIR}" -type f -name "logo.svg" -exec cp {} "${HOME}/.local/share/icons/hicolor/scalable/apps/konvert.svg" \;
else
    echo -e "${YELLOW}Notice: Precompiled release not found, building quickly from source...${RESET}"
    
    if ! command -v git >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}Error: Building from source requires 'git' and 'cargo' (Rust).${RESET}"
        echo -e "Arch Linux: sudo pacman -S git rust cmake ninja qt6-base"
        echo -e "Ubuntu/Debian: sudo apt install git cargo cmake ninja-build qt6-base-dev"
        exit 1
    fi

    git clone --depth 1 "https://github.com/${REPO}.git" "${TEMP_DIR}/source"
    cd "${TEMP_DIR}/source"
    ./scripts/install.sh
    exit 0
fi

chmod 755 "${INSTALL_DIR}/konvert"
chmod 755 "${INSTALL_DIR}/konvert-settings"
chmod 755 "${HOME}/.local/share/applications/io.github.enesehs.konvert.desktop"

# 3. Synchronize Dolphin Context Menus
echo -e "==> 3. Integrating Dolphin KIO right-click menus..."
"${INSTALL_DIR}/konvert" --sync-menu >/dev/null 2>&1 || true

# Update desktop & icon caches
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
fi

echo -e "\n${GREEN}${BOLD}✨ Konvert installed successfully!${RESET}"
echo -e "--------------------------------------------------"
echo -e "• Open settings GUI:                ${CYAN}konvert -s${RESET}"
echo -e "• Convert files via CLI:            ${CYAN}konvert -t webp image.png${RESET}"
echo -e "• In Dolphin file manager:          ${BOLD}Right-click${RESET} any image or video to use Konvert."
echo -e "• Website: ${CYAN}https://enesehs.dev${RESET} • GitHub: ${CYAN}https://github.com/${REPO}${RESET}\n"

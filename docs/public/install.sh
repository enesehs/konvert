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

# Detect execution context (root / sudo vs standard user)
if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
    REAL_USER="${SUDO_USER}"
    REAL_HOME="$(getent passwd "$SUDO_USER" | cut -d: -f6)"
else
    REAL_USER="$(id -un)"
    REAL_HOME="${HOME}"
fi

INSTALL_DIR="${REAL_HOME}/.local/bin"
ICON_DIR="${REAL_HOME}/.local/share/icons/hicolor"
DESKTOP_DIR="${REAL_HOME}/.local/share/applications"
METAINFO_DIR="${REAL_HOME}/.local/share/metainfo"
SERVICEMENU_DIR="${REAL_HOME}/.local/share/kio/servicemenus"
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
mkdir -p "${DESKTOP_DIR}"
mkdir -p "${METAINFO_DIR}"
mkdir -p "${ICON_DIR}/scalable/apps"
mkdir -p "${ICON_DIR}/scalable/actions"
mkdir -p "${SERVICEMENU_DIR}"

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

    # Verify sha256 checksum if available
    RELEASE_SHA_URL="${RELEASE_URL}.sha256"
    if curl -sSL -f -o "${TEMP_DIR}/konvert.tar.gz.sha256" "${RELEASE_SHA_URL}" 2>/dev/null; then
        echo "==> Verifying package checksum..."
        (cd "${TEMP_DIR}" && sha256sum -c konvert.tar.gz.sha256) || {
            echo -e "${RED}Error: Checksum verification failed for downloaded release package!${RESET}" >&2
            exit 1
        }
    fi

    tar -xzf "${TEMP_DIR}/konvert.tar.gz" -C "${TEMP_DIR}"
    
    rm -f "${INSTALL_DIR}/konvert" "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
    find "${TEMP_DIR}" -type f -name "konvert" -exec cp -f {} "${INSTALL_DIR}/konvert" \;
    find "${TEMP_DIR}" -type f -name "konvert-settings" -exec cp -f {} "${INSTALL_DIR}/konvert-settings" \;
    find "${TEMP_DIR}" -type f -name "io.github.enesehs.konvert.desktop" -exec cp -f {} "${DESKTOP_DIR}/" \;
    find "${TEMP_DIR}" -type f -name "io.github.enesehs.konvert.metainfo.xml" -exec cp -f {} "${METAINFO_DIR}/" \;
    find "${TEMP_DIR}" -type f -name "logo.svg" -exec cp -f {} "${ICON_DIR}/scalable/apps/konvert.svg" \;
    find "${TEMP_DIR}" -type f -path "*/icons/*.svg" -exec cp -f {} "${ICON_DIR}/scalable/actions/" \; 2>/dev/null || true
else
    echo -e "${YELLOW}Notice: Precompiled release not found, building quickly from source...${RESET}"
    
    if ! command -v git >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}Error: Building from source requires 'git' and 'cargo' (Rust).${RESET}"
        echo -e "Arch Linux: sudo pacman -S git rust cmake ninja qt6-base qt6-svg"
        echo -e "Ubuntu/Debian: sudo apt install git cargo cmake ninja-build qt6-base-dev libqt6svg6-dev"
        exit 1
    fi

    git clone --depth 1 "https://github.com/${REPO}.git" "${TEMP_DIR}/source"
    cd "${TEMP_DIR}/source"
    chmod +x ./scripts/install.sh 2>/dev/null || true
    bash ./scripts/install.sh
    exit 0
fi

chmod 755 "${INSTALL_DIR}/konvert" 2>/dev/null || true
chmod 755 "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
chmod 644 "${DESKTOP_DIR}/io.github.enesehs.konvert.desktop" 2>/dev/null || true

# Install Man page & Shell completions
MAN_DIR="${REAL_HOME}/.local/share/man/man1"
BASH_COMP_DIR="${REAL_HOME}/.local/share/bash-completion/completions"
FISH_COMP_DIR="${REAL_HOME}/.config/fish/completions"
ZSH_COMP_DIR="${REAL_HOME}/.local/share/zsh/site-functions"
mkdir -p "${MAN_DIR}" "${BASH_COMP_DIR}" "${FISH_COMP_DIR}" "${ZSH_COMP_DIR}" 2>/dev/null || true
find "${TEMP_DIR}" -type f -name "konvert.1" -exec cp -f {} "${MAN_DIR}/konvert.1" \; 2>/dev/null || true
"${INSTALL_DIR}/konvert" --completions bash > "${BASH_COMP_DIR}/konvert" 2>/dev/null || true
"${INSTALL_DIR}/konvert" --completions fish > "${FISH_COMP_DIR}/konvert.fish" 2>/dev/null || true
"${INSTALL_DIR}/konvert" --completions zsh > "${ZSH_COMP_DIR}/_konvert" 2>/dev/null || true

# 3. Synchronize Dolphin Context Menus
echo -e "==> 3. Integrating Dolphin KIO right-click menus..."
"${INSTALL_DIR}/konvert" --sync-menu --menu-dir "${SERVICEMENU_DIR}" >/dev/null 2>&1 || true

# Update desktop & icon caches
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${ICON_DIR}" >/dev/null 2>&1 || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
fi

# Fix ownership if run with sudo
if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
    chown -R "${REAL_USER}:" "${INSTALL_DIR}/konvert" 2>/dev/null || true
    [ -f "${INSTALL_DIR}/konvert-settings" ] && chown -R "${REAL_USER}:" "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
    chown -R "${REAL_USER}:" "${DESKTOP_DIR}/io.github.enesehs.konvert.desktop" "${METAINFO_DIR}/io.github.enesehs.konvert.metainfo.xml" 2>/dev/null || true
    chown -R "${REAL_USER}:" "${SERVICEMENU_DIR}"/konvert_*.desktop 2>/dev/null || true
    chown -R "${REAL_USER}:" "${ICON_DIR}" 2>/dev/null || true
    chown -R "${REAL_USER}:" "${MAN_DIR}" "${BASH_COMP_DIR}" "${FISH_COMP_DIR}" "${ZSH_COMP_DIR}" 2>/dev/null || true
fi

echo -e "\n${GREEN}${BOLD}✨ Konvert installed successfully!${RESET}"
echo -e "--------------------------------------------------"
echo -e "• Open settings GUI:                ${CYAN}konvert -s${RESET}"
echo -e "• Convert files via CLI:            ${CYAN}konvert -t webp image.png${RESET}"
echo -e "• In Dolphin file manager:          ${BOLD}Right-click${RESET} any image or video to use Konvert."
echo -e "• Website: ${CYAN}https://enesehs.dev${RESET} • GitHub: ${CYAN}https://github.com/${REPO}${RESET}\n"

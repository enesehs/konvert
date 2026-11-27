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
SKIP_VERIFY=false

for arg in "$@"; do
    case "${arg}" in
        --skip-verify)
            SKIP_VERIFY=true
            ;;
    esac
done

# Detect execution context (root / sudo vs standard user)
if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
    REAL_USER="${SUDO_USER}"
    REAL_HOME="$(getent passwd "$SUDO_USER" | cut -d: -f6)"
else
    REAL_USER="$(id -un)"
    REAL_HOME="${HOME}"
fi

run_as_real_user() {
    if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
        sudo -u "${REAL_USER}" env HOME="${REAL_HOME}" PATH="${REAL_HOME}/.cargo/bin:${PATH}" "$@"
    else
        "$@"
    fi
}

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

fetch_file() {
    local url="$1"
    local dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -sSL -f -o "${dest}" "${url}" 2>/dev/null
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "${dest}" "${url}" 2>/dev/null
    else
        return 1
    fi
}

resolve_latest_tag() {
    if command -v curl >/dev/null 2>&1; then
        curl -sSL -o /dev/null -w "%{url_effective}" "https://github.com/${REPO}/releases/latest" 2>/dev/null | sed 's#.*/tag/##'
    elif command -v wget >/dev/null 2>&1; then
        wget -q --max-redirect=10 --server-response -O /dev/null "https://github.com/${REPO}/releases/latest" 2>&1 \
            | awk '/^  Location:/{url=$2} END{sub(".*/tag/", "", url); gsub(/\r/, "", url); print url}'
    else
        echo ""
    fi
}

DOWNLOAD_SUCCESS=false
LATEST_TAG="$(resolve_latest_tag)"
if [ -n "${LATEST_TAG}" ] && [ "${LATEST_TAG}" != "latest" ]; then
    VERSIONED_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/konvert-${LATEST_TAG}-linux-${ARCH}.tar.gz"
else
    VERSIONED_URL=""
fi
FALLBACK_URL="https://github.com/${REPO}/releases/latest/download/konvert-linux-${ARCH}.tar.gz"

if [ -n "${VERSIONED_URL}" ] && fetch_file "${VERSIONED_URL}" "${TEMP_DIR}/konvert.tar.gz"; then
    DOWNLOAD_SUCCESS=true
elif fetch_file "${FALLBACK_URL}" "${TEMP_DIR}/konvert.tar.gz"; then
    DOWNLOAD_SUCCESS=true
fi

if [ "${DOWNLOAD_SUCCESS}" = true ]; then
    echo -e "${GREEN}[OK] Release package downloaded successfully.${RESET}"

    # Verify sha256 checksum fail-closed unless --skip-verify is passed
    EXPECTED_HASH=""
    if [ -n "${LATEST_TAG}" ] && fetch_file "https://github.com/${REPO}/releases/download/${LATEST_TAG}/SHA256SUMS.txt" "${TEMP_DIR}/SHA256SUMS.txt"; then
        EXPECTED_HASH="$(grep -E "konvert.*${ARCH}.*\.tar\.gz" "${TEMP_DIR}/SHA256SUMS.txt" | head -n 1 | awk '{print $1}')"
    elif fetch_file "${FALLBACK_URL}.sha256" "${TEMP_DIR}/konvert.tar.gz.sha256"; then
        EXPECTED_HASH="$(awk '{print $1}' "${TEMP_DIR}/konvert.tar.gz.sha256" | head -n 1)"
    fi

    if [ -n "${EXPECTED_HASH}" ]; then
        echo "==> Verifying package SHA-256 checksum..."
        ACTUAL_HASH="$(sha256sum "${TEMP_DIR}/konvert.tar.gz" | awk '{print $1}')"
        if [ "${EXPECTED_HASH}" != "${ACTUAL_HASH}" ]; then
            echo -e "${RED}Error: Checksum mismatch! Expected ${EXPECTED_HASH}, got ${ACTUAL_HASH}${RESET}" >&2
            exit 1
        fi
        echo -e "${GREEN}[OK] Checksum verified (${ACTUAL_HASH:0:16}...)${RESET}"
    elif [ "${SKIP_VERIFY}" = true ]; then
        echo -e "${YELLOW}Warning: No release checksum found; skipping verification (--skip-verify).${RESET}"
    else
        echo -e "${RED}Error: Unable to download or parse release SHA-256 checksum. Pass --skip-verify to bypass.${RESET}" >&2
        exit 1
    fi

    tar -xzf "${TEMP_DIR}/konvert.tar.gz" -C "${TEMP_DIR}"

    BUNDLED_INSTALLER="$(find "${TEMP_DIR}" -maxdepth 2 -type f -name "install.sh" | head -n 1 || true)"
    if [ -n "${BUNDLED_INSTALLER}" ] && [ -f "${BUNDLED_INSTALLER}" ]; then
        chmod +x "${BUNDLED_INSTALLER}" 2>/dev/null || true
        bash "${BUNDLED_INSTALLER}"
        exit $?
    fi

    rm -f "${INSTALL_DIR}/konvert" "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
    find "${TEMP_DIR}" -type f -name "konvert" -exec cp -f {} "${INSTALL_DIR}/konvert" \;
    find "${TEMP_DIR}" -type f -name "konvert-settings" -exec cp -f {} "${INSTALL_DIR}/konvert-settings" \;
    find "${TEMP_DIR}" -type f -name "io.github.enesehs.konvert.desktop" -exec cp -f {} "${DESKTOP_DIR}/" \;
    find "${TEMP_DIR}" -type f -name "io.github.enesehs.konvert.metainfo.xml" -exec cp -f {} "${METAINFO_DIR}/" \;
    find "${TEMP_DIR}" -type f -name "logo.svg" -exec cp -f {} "${ICON_DIR}/scalable/apps/konvert.svg" \;
    [ -f "${ICON_DIR}/scalable/apps/konvert.svg" ] && cp -f "${ICON_DIR}/scalable/apps/konvert.svg" "${ICON_DIR}/scalable/apps/io.github.enesehs.konvert.svg" 2>/dev/null || true
    [ -f "${ICON_DIR}/scalable/apps/konvert.svg" ] && cp -f "${ICON_DIR}/scalable/apps/konvert.svg" "${ICON_DIR}/scalable/actions/konvert.svg" 2>/dev/null || true
    find "${TEMP_DIR}" -type f -name "konvert-symbolic.svg" -exec cp -f {} "${ICON_DIR}/scalable/apps/konvert-symbolic.svg" \; 2>/dev/null || true
    [ -f "${ICON_DIR}/scalable/apps/konvert-symbolic.svg" ] && cp -f "${ICON_DIR}/scalable/apps/konvert-symbolic.svg" "${ICON_DIR}/scalable/actions/konvert-symbolic.svg" 2>/dev/null || true
    find "${TEMP_DIR}" -type f -path "*/icons/*.svg" -exec cp -f {} "${ICON_DIR}/scalable/actions/" \; 2>/dev/null || true
    if [ -f "${ICON_DIR}/scalable/apps/konvert.svg" ]; then
        for size in 16 24 32 48 64 128 256 512; do
            mkdir -p "${ICON_DIR}/${size}x${size}/apps"
            if command -v rsvg-convert >/dev/null 2>&1; then
                rsvg-convert -w "${size}" -h "${size}" "${ICON_DIR}/scalable/apps/konvert.svg" -o "${ICON_DIR}/${size}x${size}/apps/konvert.png" 2>/dev/null || true
            elif command -v magick >/dev/null 2>&1; then
                magick -background none "${ICON_DIR}/scalable/apps/konvert.svg" -resize "${size}x${size}" "${ICON_DIR}/${size}x${size}/apps/konvert.png" 2>/dev/null || true
            fi
            if [ -f "${ICON_DIR}/${size}x${size}/apps/konvert.png" ]; then
                cp -f "${ICON_DIR}/${size}x${size}/apps/konvert.png" "${ICON_DIR}/${size}x${size}/apps/io.github.enesehs.konvert.png" 2>/dev/null || true
            fi
        done
    fi
    mkdir -p "${REAL_HOME}/.local/share/mime/packages" 2>/dev/null || true
    find "${TEMP_DIR}" -type f -name "konvert-mime.xml" -exec cp -f {} "${REAL_HOME}/.local/share/mime/packages/konvert-mime.xml" \; 2>/dev/null || true
    if command -v update-mime-database >/dev/null 2>&1; then
        update-mime-database "${REAL_HOME}/.local/share/mime" >/dev/null 2>&1 || true
    fi
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

# 3. Synchronize File Manager Context Menus
echo -e "==> 3. Integrating file manager right-click menus..."
run_as_real_user "${INSTALL_DIR}/konvert" --install-all >/dev/null 2>&1 || true
run_as_real_user "${INSTALL_DIR}/konvert" --sync-menu --menu-dir "${SERVICEMENU_DIR}" >/dev/null 2>&1 || true

# Verify Qt6 runtime shared libraries for konvert-settings
if [ -f "${INSTALL_DIR}/konvert-settings" ] && command -v ldd >/dev/null 2>&1; then
    if ldd "${INSTALL_DIR}/konvert-settings" 2>/dev/null | grep -q "not found"; then
        echo -e "${YELLOW}[WARN] Missing Qt6 runtime libraries detected for konvert-settings:${RESET}"
        ldd "${INSTALL_DIR}/konvert-settings" 2>/dev/null | grep "not found" || true
        INSTALL_CMD=""
        if command -v apt-get >/dev/null 2>&1; then
            if command -v apt-cache >/dev/null 2>&1 && apt-cache show libqt6widgets6t64 >/dev/null 2>&1; then
                INSTALL_CMD="apt-get update && apt-get install -y libqt6widgets6t64 libqt6gui6t64 libqt6core6t64 libqt6network6t64 libqt6svg6"
            else
                INSTALL_CMD="apt-get update && apt-get install -y libqt6widgets6 libqt6gui6 libqt6core6 libqt6network6 libqt6svg6"
            fi
        elif command -v dnf >/dev/null 2>&1; then
            INSTALL_CMD="dnf install -y qt6-qtbase qt6-qtsvg"
        elif command -v pacman >/dev/null 2>&1; then
            INSTALL_CMD="pacman -S --needed qt6-base qt6-svg"
        fi
        if [ -n "${INSTALL_CMD}" ]; then
            if [ "$(id -u)" -eq 0 ]; then
                sh -c "${INSTALL_CMD}" || echo -e "${YELLOW}Run manually: sudo ${INSTALL_CMD}${RESET}" >&2
            elif command -v sudo >/dev/null 2>&1 && [ -r /dev/tty ]; then
                printf "Install missing Qt6 runtime libraries now with sudo? [Y/n] " > /dev/tty
                read -r REPLY < /dev/tty || REPLY="n"
                if [ -z "${REPLY}" ] || [[ "${REPLY}" =~ ^[YyEe]$ ]]; then
                    sudo sh -c "${INSTALL_CMD}" || echo -e "${YELLOW}Run manually: sudo ${INSTALL_CMD}${RESET}" >&2
                else
                    echo -e "${YELLOW}To fix konvert-settings, run: sudo ${INSTALL_CMD}${RESET}" >&2
                fi
            else
                echo -e "${YELLOW}To fix konvert-settings, run: sudo ${INSTALL_CMD}${RESET}" >&2
            fi
        fi
    fi
fi

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
    chown -R "${REAL_USER}:" "${SERVICEMENU_DIR}" 2>/dev/null || true
    chown -R "${REAL_USER}:" "${ICON_DIR}" "${REAL_HOME}/.local/share/mime" 2>/dev/null || true
    chown -R "${REAL_USER}:" "${MAN_DIR}" "${BASH_COMP_DIR}" "${FISH_COMP_DIR}" "${ZSH_COMP_DIR}" 2>/dev/null || true
fi

echo -e "\n${GREEN}${BOLD}[OK] Konvert installed successfully!${RESET}"
echo -e "--------------------------------------------------"
echo -e "• Open settings GUI:                ${CYAN}konvert -s${RESET}"
echo -e "• Convert files via CLI:            ${CYAN}konvert -t webp image.png${RESET}"
echo -e "• In Dolphin file manager:          ${BOLD}Right-click${RESET} any image or video to use Konvert."
echo -e "• Website: ${CYAN}https://enesehs.dev${RESET} • GitHub: ${CYAN}https://github.com/${REPO}${RESET}\n"

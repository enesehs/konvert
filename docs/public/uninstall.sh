#!/usr/bin/env bash
# ==============================================================================
#  Konvert - Remote Uninstaller
#  Host: https://enesehs.dev/konvert/uninstall.sh
#  Usage: curl -fsSL https://enesehs.dev/konvert/uninstall.sh | bash
# ==============================================================================

set -euo pipefail

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${YELLOW}${BOLD}==> Removing Konvert from system...${RESET}"

if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
    REAL_USER="${SUDO_USER}"
    REAL_HOME="$(getent passwd "$SUDO_USER" | cut -d: -f6)"
else
    REAL_USER="$(id -un)"
    REAL_HOME="${HOME}"
fi

run_as_real_user() {
    if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
        sudo -u "${REAL_USER}" env HOME="${REAL_HOME}" "$@"
    else
        "$@"
    fi
}

# 1. Unintegrate file manager actions
if [ -x "${REAL_HOME}/.local/bin/konvert" ]; then
    run_as_real_user "${REAL_HOME}/.local/bin/konvert" --uninstall-all >/dev/null 2>&1 || true
elif command -v konvert >/dev/null 2>&1; then
    run_as_real_user konvert --uninstall-all >/dev/null 2>&1 || true
fi

# 2. Remove binaries, desktop files, MIME types, and completions
rm -f "${REAL_HOME}/.local/bin/konvert"
rm -f "${REAL_HOME}/.local/bin/konvert-settings"
rm -f "${REAL_HOME}/.local/lib/konvert/konvert-settings"
rmdir "${REAL_HOME}/.local/lib/konvert" 2>/dev/null || true
rm -f "${REAL_HOME}/.local/share/applications/io.github.enesehs.konvert.desktop"
rm -f "${REAL_HOME}/.local/share/metainfo/io.github.enesehs.konvert.metainfo.xml"
rm -f "${REAL_HOME}/.local/share/kio/servicemenus"/konvert*.desktop 2>/dev/null || true
rm -f "${REAL_HOME}/.local/share/nemo/actions"/konvert*.nemo_action 2>/dev/null || true
rm -rf "${REAL_HOME}/.local/share/nautilus/scripts/Konvert" 2>/dev/null || true
rm -f "${REAL_HOME}/.local/share/mime/packages/konvert-mime.xml" 2>/dev/null || true
rm -f "${REAL_HOME}/.local/share/man/man1/konvert.1" "${REAL_HOME}/.local/share/man/man1/konvert.1.gz" 2>/dev/null || true
rm -f "${REAL_HOME}/.local/share/bash-completion/completions/konvert" 2>/dev/null || true
rm -f "${REAL_HOME}/.config/fish/completions/konvert.fish" 2>/dev/null || true
rm -f "${REAL_HOME}/.local/share/zsh/site-functions/_konvert" 2>/dev/null || true

# 3. Remove icons
find "${REAL_HOME}/.local/share/icons/hicolor" -name "*konvert*" -delete 2>/dev/null || true

# 4. Update system caches
if command -v update-mime-database >/dev/null 2>&1 && [ -d "${REAL_HOME}/.local/share/mime" ]; then
    run_as_real_user update-mime-database "${REAL_HOME}/.local/share/mime" >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1 && [ -d "${REAL_HOME}/.local/share/applications" ]; then
    run_as_real_user update-desktop-database "${REAL_HOME}/.local/share/applications" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1 && [ -d "${REAL_HOME}/.local/share/icons/hicolor" ]; then
    run_as_real_user gtk-update-icon-cache -f -t "${REAL_HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    run_as_real_user kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
fi

echo -e "\n${GREEN}${BOLD}[OK] Konvert has been removed from your system successfully.${RESET}\n"

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

# 1. Unintegrate file manager actions
if command -v konvert >/dev/null 2>&1; then
    konvert --uninstall-all >/dev/null 2>&1 || true
elif [ -f "${HOME}/.local/bin/konvert" ]; then
    "${HOME}/.local/bin/konvert" --uninstall-all >/dev/null 2>&1 || true
fi

# 2. Remove binaries and desktop files
rm -f "${HOME}/.local/bin/konvert"
rm -f "${HOME}/.local/bin/konvert-settings"
rm -f "${HOME}/.local/share/applications/io.github.enesehs.konvert.desktop"
rm -f "${HOME}/.local/share/metainfo/io.github.enesehs.konvert.metainfo.xml"
rm -f "${HOME}/.local/share/kio/servicemenus"/konvert*.desktop 2>/dev/null || true

# 3. Remove icons
find "${HOME}/.local/share/icons/hicolor" -name "*konvert*" -delete 2>/dev/null || true

# 4. Update system caches
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
fi

echo -e "\n${GREEN}${BOLD}✨ Konvert has been removed from your system successfully.${RESET}\n"

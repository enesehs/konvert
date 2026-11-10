#!/usr/bin/env bash
set -euo pipefail

echo "==> Removing file manager integrations..."
if command -v konvert >/dev/null 2>&1; then
    konvert --uninstall-all >/dev/null 2>&1 || true
elif [ -f "${HOME}/.local/bin/konvert" ]; then
    "${HOME}/.local/bin/konvert" --uninstall-all >/dev/null 2>&1 || true
fi

echo "==> Removing binaries, menus, and icons..."
rm -f "${HOME}/.local/bin/konvert"
rm -f "${HOME}/.local/bin/konvert-settings"
rm -f "${HOME}/.local/share/applications/io.github.enesehs.konvert.desktop"
rm -f "${HOME}/.local/share/metainfo/io.github.enesehs.konvert.metainfo.xml"
rm -f "${HOME}/.local/share/kio/servicemenus"/konvert*.desktop 2>/dev/null || true
find "${HOME}/.local/share/icons/hicolor" -name "*konvert*" -delete 2>/dev/null || true

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
fi

echo "✨ Konvert uninstalled from system successfully."

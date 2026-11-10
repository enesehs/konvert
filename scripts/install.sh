#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

echo "==> 1. Building Rust Core Engine (konvert)..."
cargo build --release

echo "==> 2. Building Qt6 / KDE Settings GUI (konvert-settings)..."
mkdir -p gui/build
cd gui/build
cmake -GNinja ..
ninja
cd "${ROOT_DIR}"

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

echo "==> Copying binaries to ${INSTALL_DIR}..."
install -m 755 target/release/konvert "${INSTALL_DIR}/konvert"
install -m 755 gui/build/konvert-settings "${INSTALL_DIR}/konvert-settings"

echo "==> Installing system icons (KDE & Hicolor)..."
ICON_DIR="${HOME}/.local/share/icons/hicolor"
mkdir -p "${ICON_DIR}/scalable/apps"
cp "${ROOT_DIR}/assets/logo.svg" "${ICON_DIR}/scalable/apps/konvert.svg"
cp "${ROOT_DIR}/assets/logo.svg" "${ICON_DIR}/scalable/apps/io.github.enesehs.konvert.svg"

for size in 16 24 32 48 64 128 256 512; do
  mkdir -p "${ICON_DIR}/${size}x${size}/apps"
  if command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w $size -h $size "${ROOT_DIR}/assets/logo.svg" -o "${ICON_DIR}/${size}x${size}/apps/konvert.png"
  elif command -v magick >/dev/null 2>&1; then
    magick -background none "${ROOT_DIR}/assets/logo.svg" -resize ${size}x${size} "${ICON_DIR}/${size}x${size}/apps/konvert.png"
  fi
  if [ -f "${ICON_DIR}/${size}x${size}/apps/konvert.png" ]; then
    cp "${ICON_DIR}/${size}x${size}/apps/konvert.png" "${ICON_DIR}/${size}x${size}/apps/io.github.enesehs.konvert.png"
  fi
done

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${ICON_DIR}" 2>/dev/null || true
fi

if command -v kbuildsycoca6 >/dev/null 2>&1; then
  kbuildsycoca6 --noincremental 2>/dev/null || true
fi

echo "==> Installing desktop launcher and AppStream metainfo..."
mkdir -p "${HOME}/.local/share/applications"
mkdir -p "${HOME}/.local/share/metainfo"
cp io.github.enesehs.konvert.desktop "${HOME}/.local/share/applications/"
cp packaging/io.github.enesehs.konvert.metainfo.xml "${HOME}/.local/share/metainfo/"
chmod 755 "${HOME}/.local/share/applications/io.github.enesehs.konvert.desktop"

echo "==> Synchronizing Dolphin right-click context menus..."
rm -f "${HOME}/.local/share/kio/servicemenus/konvert.desktop"
"${INSTALL_DIR}/konvert" --sync-menu
chmod -f 755 "${HOME}/.local/share/kio/servicemenus"/konvert_*.desktop || true

echo ""
echo "✨ Konvert installed and synchronized successfully!"
echo "To open settings: konvert -s or launch Konvert from your applications menu."

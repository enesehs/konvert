#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

# Detect execution context (root / sudo vs standard user)
if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
  REAL_USER="${SUDO_USER}"
  REAL_HOME="$(getent passwd "$SUDO_USER" | cut -d: -f6)"
else
  REAL_USER="$(id -un)"
  REAL_HOME="${HOME}"
fi

echo "==> 1. Building Rust Core Engine (konvert)..."
cargo build --release

echo "==> 2. Building Qt6 / KDE Settings GUI (konvert-settings)..."
if command -v cmake >/dev/null 2>&1 && command -v ninja >/dev/null 2>&1; then
  mkdir -p gui/build
  cd gui/build
  cmake -GNinja .. || true
  ninja || true
  cd "${ROOT_DIR}"
fi

INSTALL_DIR="${REAL_HOME}/.local/bin"
ICON_DIR="${REAL_HOME}/.local/share/icons/hicolor"
DESKTOP_DIR="${REAL_HOME}/.local/share/applications"
METAINFO_DIR="${REAL_HOME}/.local/share/metainfo"
SERVICEMENU_DIR="${REAL_HOME}/.local/share/kio/servicemenus"

mkdir -p "${INSTALL_DIR}"
mkdir -p "${DESKTOP_DIR}"
mkdir -p "${METAINFO_DIR}"
mkdir -p "${SERVICEMENU_DIR}"

echo "==> Copying binaries to ${INSTALL_DIR}..."
# Remove existing targets first to avoid ETXTBSY (Text file busy) or root-owned permission lockouts
rm -f "${INSTALL_DIR}/konvert" 2>/dev/null || sudo rm -f "${INSTALL_DIR}/konvert" 2>/dev/null || true
install -m 755 target/release/konvert "${INSTALL_DIR}/konvert"

if [ -f gui/build/konvert-settings ]; then
  rm -f "${INSTALL_DIR}/konvert-settings" 2>/dev/null || sudo rm -f "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
  install -m 755 gui/build/konvert-settings "${INSTALL_DIR}/konvert-settings"
fi

echo "==> Installing system icons (KDE & Hicolor) & Lucide action icons..."
mkdir -p "${ICON_DIR}/scalable/apps"
mkdir -p "${ICON_DIR}/scalable/actions"

cp -f "${ROOT_DIR}/assets/logo.svg" "${ICON_DIR}/scalable/apps/konvert.svg"
cp -f "${ROOT_DIR}/assets/logo.svg" "${ICON_DIR}/scalable/apps/io.github.enesehs.konvert.svg"

# Copy Lucide action icons for right-click context menu
if [ -d "${ROOT_DIR}/assets/icons" ]; then
  cp -f "${ROOT_DIR}/assets/icons/"*.svg "${ICON_DIR}/scalable/actions/" 2>/dev/null || true
fi

for size in 16 24 32 48 64 128 256 512; do
  mkdir -p "${ICON_DIR}/${size}x${size}/apps"
  if command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w $size -h $size "${ROOT_DIR}/assets/logo.svg" -o "${ICON_DIR}/${size}x${size}/apps/konvert.png" 2>/dev/null || true
  elif command -v magick >/dev/null 2>&1; then
    magick -background none "${ROOT_DIR}/assets/logo.svg" -resize ${size}x${size} "${ICON_DIR}/${size}x${size}/apps/konvert.png" 2>/dev/null || true
  fi
  if [ -f "${ICON_DIR}/${size}x${size}/apps/konvert.png" ]; then
    cp -f "${ICON_DIR}/${size}x${size}/apps/konvert.png" "${ICON_DIR}/${size}x${size}/apps/io.github.enesehs.konvert.png" 2>/dev/null || true
  fi
done

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${ICON_DIR}" 2>/dev/null || true
fi

if command -v kbuildsycoca6 >/dev/null 2>&1; then
  kbuildsycoca6 --noincremental 2>/dev/null || true
fi

echo "==> Installing desktop launcher and AppStream metainfo..."
cp -f io.github.enesehs.konvert.desktop "${DESKTOP_DIR}/"
cp -f packaging/io.github.enesehs.konvert.metainfo.xml "${METAINFO_DIR}/"
chmod 755 "${DESKTOP_DIR}/io.github.enesehs.konvert.desktop" 2>/dev/null || true

echo "==> Synchronizing Dolphin right-click context menus..."
rm -f "${SERVICEMENU_DIR}/konvert.desktop" 2>/dev/null || true
"${INSTALL_DIR}/konvert" --sync-menu --menu-dir "${SERVICEMENU_DIR}" || true
chmod -f 755 "${SERVICEMENU_DIR}"/konvert_*.desktop 2>/dev/null || true

# If script was run with sudo, fix ownership so standard user owns their local files
if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
  echo "==> Ensuring user ownership for ${REAL_USER}..."
  chown -R "${REAL_USER}:" "${INSTALL_DIR}/konvert" 2>/dev/null || true
  [ -f "${INSTALL_DIR}/konvert-settings" ] && chown -R "${REAL_USER}:" "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
  chown -R "${REAL_USER}:" "${DESKTOP_DIR}/io.github.enesehs.konvert.desktop" "${METAINFO_DIR}/io.github.enesehs.konvert.metainfo.xml" 2>/dev/null || true
  chown -R "${REAL_USER}:" "${SERVICEMENU_DIR}"/konvert_*.desktop 2>/dev/null || true
  chown -R "${REAL_USER}:" "${ICON_DIR}" 2>/dev/null || true
fi

echo ""
echo "✨ Konvert installed and synchronized successfully!"
echo "To open settings: konvert -s or launch Konvert from your applications menu."

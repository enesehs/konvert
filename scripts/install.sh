#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# İki yerleşim desteklenir:
#   1) Kaynak ağacı:  <repo>/scripts/install.sh   (ROOT = repo kökü)
#   2) Release tarball: <paket>/install.sh + hazır binary'ler (ROOT = paket dizini)
if [ -f "${SCRIPT_DIR}/Cargo.toml" ]; then
  ROOT_DIR="${SCRIPT_DIR}"
elif [ -f "${SCRIPT_DIR}/../Cargo.toml" ]; then
  ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
else
  ROOT_DIR="${SCRIPT_DIR}"
fi
cd "${ROOT_DIR}"

PREBUILT=0
if [ -x "${SCRIPT_DIR}/konvert" ] && [ -x "${SCRIPT_DIR}/konvert-settings" ]; then
  PREBUILT=1
fi

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

if [ "${PREBUILT}" -eq 1 ]; then
  echo "==> 1. Using prebuilt binaries from ${SCRIPT_DIR} (no compilation needed)."
else
  echo "==> 1. Building Rust Core Engine (konvert)..."
  run_as_real_user cargo build --release

  echo "==> 2. Building Qt6 / KDE Settings GUI (konvert-settings)..."
  if command -v cmake >/dev/null 2>&1; then
    run_as_real_user cmake -B gui/build -S gui -DCMAKE_BUILD_TYPE=Release
    run_as_real_user cmake --build gui/build
  else
    echo "Warning: cmake not found; the settings GUI will not be installed." >&2
  fi
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

if [ "${PREBUILT}" -eq 1 ]; then
  install -m 755 "${SCRIPT_DIR}/konvert" "${INSTALL_DIR}/konvert"
  rm -f "${INSTALL_DIR}/konvert-settings" 2>/dev/null || sudo rm -f "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
  install -m 755 "${SCRIPT_DIR}/konvert-settings" "${INSTALL_DIR}/konvert-settings"
else
  install -m 755 target/release/konvert "${INSTALL_DIR}/konvert"

  if [ -f gui/build/konvert-settings ]; then
    rm -f "${INSTALL_DIR}/konvert-settings" 2>/dev/null || sudo rm -f "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
    install -m 755 gui/build/konvert-settings "${INSTALL_DIR}/konvert-settings"
  fi
fi

echo "==> Installing system icons (KDE & Hicolor) & Lucide action icons..."
mkdir -p "${ICON_DIR}/scalable/apps"
mkdir -p "${ICON_DIR}/scalable/actions"

cp -f "${ROOT_DIR}/assets/logo.svg" "${ICON_DIR}/scalable/apps/konvert.svg"
cp -f "${ROOT_DIR}/assets/logo.svg" "${ICON_DIR}/scalable/apps/io.github.enesehs.konvert.svg"
cp -f "${ROOT_DIR}/assets/logo.svg" "${ICON_DIR}/scalable/actions/konvert.svg"
[ -f "${ROOT_DIR}/assets/icons/konvert-symbolic.svg" ] && cp -f "${ROOT_DIR}/assets/icons/konvert-symbolic.svg" "${ICON_DIR}/scalable/apps/konvert-symbolic.svg"
[ -f "${ROOT_DIR}/assets/icons/konvert-symbolic.svg" ] && cp -f "${ROOT_DIR}/assets/icons/konvert-symbolic.svg" "${ICON_DIR}/scalable/actions/konvert-symbolic.svg"

# Copy Lucide action icons for right-click context menu
if [ -d "${ROOT_DIR}/assets/icons" ]; then
  cp -f "${ROOT_DIR}/assets/icons/"*.svg "${ICON_DIR}/scalable/actions/" 2>/dev/null || true
fi

# Install custom MIME types (e.g. application/x-udf for *.udf)
MIME_DIR="${REAL_HOME}/.local/share/mime/packages"
mkdir -p "${MIME_DIR}"
if [ -f "${ROOT_DIR}/packaging/konvert-mime.xml" ]; then
  cp -f "${ROOT_DIR}/packaging/konvert-mime.xml" "${MIME_DIR}/konvert-mime.xml"
  if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database "${REAL_HOME}/.local/share/mime" 2>/dev/null || true
  fi
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
chmod 644 "${DESKTOP_DIR}/io.github.enesehs.konvert.desktop" 2>/dev/null || true

echo "==> Synchronizing desktop file manager context menus..."
rm -f "${SERVICEMENU_DIR}/konvert.desktop" 2>/dev/null || true
run_as_real_user "${INSTALL_DIR}/konvert" --install-all >/dev/null 2>&1 || true
run_as_real_user "${INSTALL_DIR}/konvert" --sync-menu --menu-dir "${SERVICEMENU_DIR}" || true
chmod -f 755 "${SERVICEMENU_DIR}"/konvert_*.desktop 2>/dev/null || true

# Verify Qt6 runtime shared libraries for konvert-settings
if [ -f "${INSTALL_DIR}/konvert-settings" ] && command -v ldd >/dev/null 2>&1; then
  LDD_OUT="$(ldd "${INSTALL_DIR}/konvert-settings" 2>&1 || true)"
  if echo "${LDD_OUT}" | grep -q "=> not found"; then
    echo ""
    echo "[WARN] Missing Qt6 runtime libraries detected for konvert-settings:"
    echo "${LDD_OUT}" | grep "=> not found" || true
    INSTALL_CMD=""
    if command -v apt-get >/dev/null 2>&1; then
      if command -v apt-cache >/dev/null 2>&1 && apt-cache show libqt6widgets6t64 2>/dev/null | grep -q '^Filename:'; then
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
        echo "==> Installing missing Qt6 runtime libraries..."
        sh -c "${INSTALL_CMD}" || echo "[WARN] Automatic installation failed. Run: sudo ${INSTALL_CMD}" >&2
      elif command -v sudo >/dev/null 2>&1 && [ -r /dev/tty ]; then
        printf "Install missing Qt6 runtime libraries now with sudo? [Y/n] " > /dev/tty
        read -r REPLY < /dev/tty || REPLY="n"
        if [ -z "${REPLY}" ] || [[ "${REPLY}" =~ ^[YyEe]$ ]]; then
          sudo sh -c "${INSTALL_CMD}" || echo "[WARN] Run manually: sudo ${INSTALL_CMD}" >&2
        else
          echo "[WARN] To fix konvert-settings, run: sudo ${INSTALL_CMD}" >&2
        fi
      else
        echo "[WARN] To fix konvert-settings, run: sudo ${INSTALL_CMD}" >&2
      fi
    fi
  elif echo "${LDD_OUT}" | grep -q "version .* not found"; then
    echo ""
    echo "[WARN] konvert-settings requires a newer system library version than installed on this host:"
    echo "${LDD_OUT}" | grep "version .* not found" || true
    echo "[WARN] Use the AppImage release or build the GUI locally against your system Qt6."
  fi
fi

# Man page & Shell completions
MAN_DIR="${REAL_HOME}/.local/share/man/man1"
BASH_COMP_DIR="${REAL_HOME}/.local/share/bash-completion/completions"
FISH_COMP_DIR="${REAL_HOME}/.config/fish/completions"
ZSH_COMP_DIR="${REAL_HOME}/.local/share/zsh/site-functions"
mkdir -p "${MAN_DIR}" "${BASH_COMP_DIR}" "${FISH_COMP_DIR}" "${ZSH_COMP_DIR}" 2>/dev/null || true
[ -f "${ROOT_DIR}/docs/konvert.1" ] && cp -f "${ROOT_DIR}/docs/konvert.1" "${MAN_DIR}/konvert.1" 2>/dev/null || true
chmod 644 "${MAN_DIR}/konvert.1" 2>/dev/null || true
"${INSTALL_DIR}/konvert" --completions bash > "${BASH_COMP_DIR}/konvert" 2>/dev/null || true
"${INSTALL_DIR}/konvert" --completions fish > "${FISH_COMP_DIR}/konvert.fish" 2>/dev/null || true
"${INSTALL_DIR}/konvert" --completions zsh > "${ZSH_COMP_DIR}/_konvert" 2>/dev/null || true

# If script was run with sudo, fix ownership so standard user owns their local files
if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
  echo "==> Ensuring user ownership for ${REAL_USER}..."
  chown -R "${REAL_USER}:" "${INSTALL_DIR}/konvert" 2>/dev/null || true
  [ -f "${INSTALL_DIR}/konvert-settings" ] && chown -R "${REAL_USER}:" "${INSTALL_DIR}/konvert-settings" 2>/dev/null || true
  chown -R "${REAL_USER}:" "${DESKTOP_DIR}/io.github.enesehs.konvert.desktop" "${METAINFO_DIR}/io.github.enesehs.konvert.metainfo.xml" 2>/dev/null || true
  chown -R "${REAL_USER}:" "${SERVICEMENU_DIR}" 2>/dev/null || true
  chown -R "${REAL_USER}:" "${ICON_DIR}" "${REAL_HOME}/.local/share/mime" 2>/dev/null || true
  chown -R "${REAL_USER}:" "${MAN_DIR}" "${BASH_COMP_DIR}" "${FISH_COMP_DIR}" "${ZSH_COMP_DIR}" 2>/dev/null || true
fi

echo ""
echo "[OK] Konvert installed and synchronized successfully!"
echo "To open settings: konvert -s or launch Konvert from your applications menu."

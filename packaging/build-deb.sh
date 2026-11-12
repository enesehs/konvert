#!/usr/bin/env bash
# Konvert Debian (.deb) Package Generator
# Generates standard Debian/Ubuntu package for x86_64 / amd64 architecture.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${ROOT_DIR}/build-deb-tmp"

if [ -f "${ROOT_DIR}/VERSION" ]; then
    VERSION="$(tr -d 'v' < "${ROOT_DIR}/VERSION" | tr -d ' \n\r')"
else
    VERSION="0.3.0"
fi
PKG_NAME="konvert"
ARCH="${DEB_ARCH:-$(dpkg --print-architecture 2>/dev/null || echo amd64)}"
DEB_FILE="${PKG_NAME}_${VERSION}_${ARCH}.deb"

echo "==> 1. Generating Konvert .deb package (v${VERSION} - ${ARCH})..."

# Build check
if [ ! -f "${ROOT_DIR}/target/release/konvert" ]; then
    echo "Error: ${ROOT_DIR}/target/release/konvert not found. Please run 'cargo build --release' first."
    exit 1
fi

if [ ! -f "${ROOT_DIR}/gui/build/konvert-settings" ]; then
    echo "Error: ${ROOT_DIR}/gui/build/konvert-settings not found. Please build the GUI first."
    exit 1
fi

rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/applications"
mkdir -p "${BUILD_DIR}/usr/share/metainfo"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${BUILD_DIR}/usr/share/man/man1"

# 1. İkili dosyaları kopyala
cp "${ROOT_DIR}/target/release/konvert" "${BUILD_DIR}/usr/bin/"
cp "${ROOT_DIR}/gui/build/konvert-settings" "${BUILD_DIR}/usr/bin/"
chmod 755 "${BUILD_DIR}/usr/bin/konvert"
chmod 755 "${BUILD_DIR}/usr/bin/konvert-settings"

# 2. Desktop, AppStream ve Dolphin ServiceMenu dosyaları
cp "${ROOT_DIR}/io.github.enesehs.konvert.desktop" "${BUILD_DIR}/usr/share/applications/"
cp "${ROOT_DIR}/packaging/io.github.enesehs.konvert.metainfo.xml" "${BUILD_DIR}/usr/share/metainfo/"
cp "${ROOT_DIR}/docs/konvert.1" "${BUILD_DIR}/usr/share/man/man1/konvert.1"
chmod 644 "${BUILD_DIR}/usr/share/man/man1/konvert.1"
chmod 644 "${BUILD_DIR}/usr/share/applications/io.github.enesehs.konvert.desktop"
chmod 644 "${BUILD_DIR}/usr/share/metainfo/io.github.enesehs.konvert.metainfo.xml"

# Dolphin KIO ServiceMenus
mkdir -p "${BUILD_DIR}/usr/share/kio/servicemenus"
"${ROOT_DIR}/target/release/konvert" --sync-menu --menu-dir "${BUILD_DIR}/usr/share/kio/servicemenus" --menu-exec "konvert"
chmod 644 "${BUILD_DIR}/usr/share/kio/servicemenus"/*.desktop

# Shell completions (bash, zsh, fish)
mkdir -p "${BUILD_DIR}/usr/share/bash-completion/completions"
mkdir -p "${BUILD_DIR}/usr/share/zsh/site-functions"
mkdir -p "${BUILD_DIR}/usr/share/fish/vendor_completions.d"
"${ROOT_DIR}/target/release/konvert" --completions bash > "${BUILD_DIR}/usr/share/bash-completion/completions/konvert"
"${ROOT_DIR}/target/release/konvert" --completions zsh > "${BUILD_DIR}/usr/share/zsh/site-functions/_konvert"
"${ROOT_DIR}/target/release/konvert" --completions fish > "${BUILD_DIR}/usr/share/fish/vendor_completions.d/konvert.fish"
chmod 644 "${BUILD_DIR}/usr/share/bash-completion/completions/konvert"
chmod 644 "${BUILD_DIR}/usr/share/zsh/site-functions/_konvert"
chmod 644 "${BUILD_DIR}/usr/share/fish/vendor_completions.d/konvert.fish"

# 3. Simgeleri yerleştir
cp "${ROOT_DIR}/assets/logo.svg" "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/konvert.svg"
cp "${ROOT_DIR}/assets/logo.svg" "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/io.github.enesehs.konvert.svg"

mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/actions"
if [ -d "${ROOT_DIR}/assets/icons" ]; then
    cp -f "${ROOT_DIR}/assets/icons/"*.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/actions/"
    chmod 644 "${BUILD_DIR}/usr/share/icons/hicolor/scalable/actions/"*.svg
fi

for SIZE in 16 24 32 48 64 128 256; do
    DIR="${BUILD_DIR}/usr/share/icons/hicolor/${SIZE}x${SIZE}/apps"
    mkdir -p "${DIR}"
    if command -v rsvg-convert >/dev/null 2>&1; then
        rsvg-convert -w "${SIZE}" -h "${SIZE}" "${ROOT_DIR}/assets/logo.svg" -o "${DIR}/konvert.png"
    elif command -v magick >/dev/null 2>&1; then
        magick -background none -density 300 "${ROOT_DIR}/assets/logo.svg" -resize "${SIZE}x${SIZE}" "${DIR}/konvert.png"
    fi
    if [ -f "${DIR}/konvert.png" ]; then
        cp "${DIR}/konvert.png" "${DIR}/io.github.enesehs.konvert.png"
    fi
done

# 4. DEBIAN/control dosyası
cat << EOF > "${BUILD_DIR}/DEBIAN/control"
Package: konvert
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: enesehs <enesehs@protonmail.com>
Depends: libc6, libqt6core6, libqt6gui6, libqt6widgets6, libqt6svg6, ffmpeg, imagemagick
Recommends: poppler-utils, ghostscript, libreoffice, libjxl-tools
Homepage: https://github.com/enesehs/konvert
Description: High performance file converter and optimizer for Linux and KDE Plasma
 Konvert is an ultra-fast, offline, multi-threaded file conversion and optimization
 utility with Dolphin context menu integration, smart sizing presets, and zero telemetry.
EOF

# 5. DEBIAN/postinst tetikleyicisi
cat << 'EOF' > "${BUILD_DIR}/DEBIAN/postinst"
#!/bin/sh
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications 2>/dev/null || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 --noincremental 2>/dev/null || true
fi
exit 0
EOF
chmod 755 "${BUILD_DIR}/DEBIAN/postinst"

# 6. .deb paketini üret
OUTPUT_DIR="${ROOT_DIR}/packaging"
OUTPUT_DEB="${OUTPUT_DIR}/${DEB_FILE}"

if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "${BUILD_DIR}" "${OUTPUT_DEB}"
else
    # dpkg-deb olmayan sistemlerde (Arch vb.) standart ar/tar ile derle
    TEMP_AR="${ROOT_DIR}/build-deb-ar"
    rm -rf "${TEMP_AR}"
    mkdir -p "${TEMP_AR}"

    echo "2.0" > "${TEMP_AR}/debian-binary"
    (cd "${BUILD_DIR}/DEBIAN" && tar -czf "${TEMP_AR}/control.tar.gz" control postinst)
    (cd "${BUILD_DIR}" && tar --exclude='./DEBIAN' -czf "${TEMP_AR}/data.tar.gz" .)

    (cd "${TEMP_AR}" && ar rcs "${OUTPUT_DEB}" debian-binary control.tar.gz data.tar.gz)
    rm -rf "${TEMP_AR}"
fi

rm -rf "${BUILD_DIR}"

echo "✨ Debian package created successfully:"
echo "   ${OUTPUT_DEB}"
ls -lh "${OUTPUT_DEB}"

#!/usr/bin/env bash
# Generates an AppImage bundle for Konvert
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VERSION="${1:-$(grep '^version =' "${ROOT_DIR}/Cargo.toml" | head -n1 | cut -d'"' -f2)}"
ARCH="$(uname -m)"
APPDIR="${ROOT_DIR}/target/AppDir"

echo "==> 1. Building Konvert release binaries..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> 2. Preparing AppDir layout..."
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/share/metainfo"

cp "${ROOT_DIR}/target/release/konvert" "${APPDIR}/usr/bin/"
if [ -f "${ROOT_DIR}/gui/build/konvert-gui" ]; then
    cp "${ROOT_DIR}/gui/build/konvert-gui" "${APPDIR}/usr/bin/"
fi

# Icon & desktop files
cp "${ROOT_DIR}/assets/logo.svg" "${APPDIR}/konvert.svg"
cp "${ROOT_DIR}/assets/logo.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/konvert.svg"
cp "${ROOT_DIR}/packaging/io.github.enesehs.konvert.metainfo.xml" "${APPDIR}/usr/share/metainfo/"

cat << 'EOF' > "${APPDIR}/konvert.desktop"
[Desktop Entry]
Name=Konvert
Comment=Universal fast media and document converter
Exec=konvert-gui %F
Icon=konvert
Terminal=false
Type=Application
Categories=Utility;Graphics;AudioVideo;
MimeType=image/png;image/jpeg;image/webp;application/pdf;
EOF

cp "${APPDIR}/konvert.desktop" "${APPDIR}/usr/share/applications/"

# AppRun entrypoint
cat << 'EOF' > "${APPDIR}/AppRun"
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

if [ -f "${HERE}/usr/bin/konvert-gui" ] && [ $# -eq 0 ]; then
    exec "${HERE}/usr/bin/konvert-gui" "$@"
else
    exec "${HERE}/usr/bin/konvert" "$@"
fi
EOF
chmod +x "${APPDIR}/AppRun"

echo "==> 3. Generating AppImage output..."
OUTPUT_FILE="${ROOT_DIR}/packaging/Konvert-${VERSION}-${ARCH}.AppImage"

if command -v appimagetool >/dev/null 2>&1; then
    ARCH="${ARCH}" appimagetool "${APPDIR}" "${OUTPUT_FILE}"
elif [ -f "${ROOT_DIR}/appimagetool" ]; then
    ARCH="${ARCH}" "${ROOT_DIR}/appimagetool" "${APPDIR}" "${OUTPUT_FILE}"
else
    echo "Notice: appimagetool not found in PATH."
    echo "AppDir structure prepared successfully at: ${APPDIR}"
    echo "You can package it using: ARCH=${ARCH} appimagetool ${APPDIR} ${OUTPUT_FILE}"
    exit 0
fi

echo "==> AppImage created at ${OUTPUT_FILE}"

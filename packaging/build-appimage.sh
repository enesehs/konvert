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
cmake -B "${ROOT_DIR}/gui/build" -S "${ROOT_DIR}/gui" -DCMAKE_BUILD_TYPE=Release
cmake --build "${ROOT_DIR}/gui/build"

echo "==> 2. Preparing AppDir layout..."
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/actions"
mkdir -p "${APPDIR}/usr/share/metainfo"
mkdir -p "${APPDIR}/usr/share/mime/packages"

cp "${ROOT_DIR}/target/release/konvert" "${APPDIR}/usr/bin/"
cp "${ROOT_DIR}/gui/build/konvert-settings" "${APPDIR}/usr/bin/"

# Icon, MIME & desktop files
cp "${ROOT_DIR}/assets/logo.svg" "${APPDIR}/konvert.svg"
cp "${ROOT_DIR}/assets/logo.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/konvert.svg"
cp "${ROOT_DIR}/assets/icons/konvert-symbolic.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/konvert-symbolic.svg"
cp -f "${ROOT_DIR}/assets/icons/"*.svg "${APPDIR}/usr/share/icons/hicolor/scalable/actions/"
cp "${ROOT_DIR}/packaging/konvert-mime.xml" "${APPDIR}/usr/share/mime/packages/"
cp "${ROOT_DIR}/packaging/io.github.enesehs.konvert.metainfo.xml" "${APPDIR}/usr/share/metainfo/"

cat << 'EOF' > "${APPDIR}/konvert.desktop"
[Desktop Entry]
Name=Konvert
Comment=Fast, offline media and document converter
Exec=konvert-settings %F
Icon=konvert
Terminal=false
Type=Application
Categories=Utility;
MimeType=image/png;image/jpeg;image/webp;application/pdf;
StartupWMClass=konvert-settings
EOF

cp "${APPDIR}/konvert.desktop" "${APPDIR}/usr/share/applications/io.github.enesehs.konvert.desktop"

# AppRun entrypoint
cat << 'EOF' > "${APPDIR}/AppRun"
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

if [ -x "${HERE}/usr/bin/konvert-settings" ] && [ $# -eq 0 ]; then
    exec "${HERE}/usr/bin/konvert-settings" "$@"
else
    exec "${HERE}/usr/bin/konvert" "$@"
fi
EOF
chmod +x "${APPDIR}/AppRun"

echo "==> 3. Bundling Qt6 & generating AppImage output..."
OUTPUT_FILE="${ROOT_DIR}/packaging/Konvert-${VERSION}-${ARCH}.AppImage"
export APPIMAGE_EXTRACT_AND_RUN="${APPIMAGE_EXTRACT_AND_RUN:-1}"

if command -v linuxdeploy >/dev/null 2>&1; then
    if command -v linuxdeploy-plugin-qt >/dev/null 2>&1; then
        OUTPUT="${OUTPUT_FILE}" LDAI_OUTPUT="${OUTPUT_FILE}" \
            QMAKE="$(command -v qmake6 || command -v qmake || true)" \
            linuxdeploy --appdir "${APPDIR}" --plugin qt --output appimage
    else
        OUTPUT="${OUTPUT_FILE}" LDAI_OUTPUT="${OUTPUT_FILE}" \
            linuxdeploy --appdir "${APPDIR}" --output appimage
    fi
elif command -v appimagetool >/dev/null 2>&1; then
    ARCH="${ARCH}" appimagetool "${APPDIR}" "${OUTPUT_FILE}"
elif [ -x "${ROOT_DIR}/appimagetool" ]; then
    ARCH="${ARCH}" "${ROOT_DIR}/appimagetool" "${APPDIR}" "${OUTPUT_FILE}"
else
    echo "Notice: appimagetool not found in PATH."
    echo "AppDir structure prepared successfully at: ${APPDIR}"
    echo "You can package it using: ARCH=${ARCH} appimagetool ${APPDIR} ${OUTPUT_FILE}"
    exit 0
fi

echo "==> AppImage created at ${OUTPUT_FILE}"

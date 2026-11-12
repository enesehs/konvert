#!/usr/bin/env bash
# Updates sha256sums in packaging/PKGBUILD for the specified version/tag or downloaded archive.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKGBUILD_FILE="${SCRIPT_DIR}/PKGBUILD"

if [ ! -f "${PKGBUILD_FILE}" ]; then
    echo "Error: PKGBUILD not found at ${PKGBUILD_FILE}" >&2
    exit 1
fi

VERSION="${1:-$(grep -E '^pkgver=' "${PKGBUILD_FILE}" | cut -d= -f2 | tr -d ' ')}"
ARCHIVE_URL="https://github.com/enesehs/konvert/archive/refs/tags/v${VERSION}.tar.gz"

echo "==> Computing sha256 checksum for v${VERSION}..."
TEMP_FILE="$(mktemp)"
trap 'rm -f "${TEMP_FILE}"' EXIT

if curl -sSL -f -o "${TEMP_FILE}" "${ARCHIVE_URL}" 2>/dev/null; then
    SHA256="$(sha256sum "${TEMP_FILE}" | awk '{print $1}')"
    echo "==> Got SHA256: ${SHA256}"
    sed -i "s/^sha256sums=('.*')/sha256sums=('${SHA256}')/" "${PKGBUILD_FILE}"
    echo "==> Updated ${PKGBUILD_FILE} with checksum."
else
    echo "Notice: Release tag v${VERSION} not yet published on GitHub."
    echo "If you have a local archive, run: ./packaging/update-checksums.sh <path-to-tar.gz>"
    if [ -f "${1:-}" ]; then
        SHA256="$(sha256sum "$1" | awk '{print $1}')"
        sed -i "s/^sha256sums=('.*')/sha256sums=('${SHA256}')/" "${PKGBUILD_FILE}"
        echo "==> Updated PKGBUILD from local file: ${SHA256}"
    fi
fi

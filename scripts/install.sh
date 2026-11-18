#!/usr/bin/env bash
#
# install.sh — One-line installer for Lexic
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Crazygiscool/Lexic/main/scripts/install.sh | sh
#
# Detects OS/arch, downloads the matching GitHub release asset, and installs
# the lexic binary to a directory on your PATH.
#
set -euo pipefail

REPO="Crazygiscool/Lexic"
BIN_NAME="lexic"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
os_detect() {
  case "$(uname -s)" in
    Linux*)     echo "linux" ;;
    Darwin*)    echo "macos" ;;
    CYGWIN*|MINGW*|MSYS*|Windows*) echo "windows" ;;
    *)          echo "unsupported" ;;
  esac
}

arch_detect() {
  case "$(uname -m)" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *)            echo "$(uname -m)" ;;
  esac
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "ERROR: required command not found: $1" >&2
    exit 1
  fi
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
OS="$(os_detect)"
ARCH="$(arch_detect)"

if [ "$OS" = "unsupported" ]; then
  echo "ERROR: unsupported operating system: $(uname -s)" >&2
  exit 1
fi

need_cmd curl
need_cmd tar

# Determine the latest release tag
LATEST_URL="https://api.github.com/repos/${REPO}/releases/latest"
TAG=$(curl -sL "${LATEST_URL}" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$TAG" ]; then
  echo "ERROR: could not determine latest release version" >&2
  exit 1
fi

echo "==> Installing Lexic ${TAG} for ${OS}/${ARCH}..."

# Build the download URL based on platform
case "$OS" in
  linux)
    ASSET="lexic-linux-${ARCH}.tar.gz"
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "${INSTALL_DIR}"
    TMPDIR="$(mktemp -d)"
    curl -fsSL "https://github.com/${REPO}/releases/download/${TAG}/${ASSET}" -o "${TMPDIR}/lexic.tar.gz"
    tar -xzf "${TMPDIR}/lexic.tar.gz" -C "${TMPDIR}"
    # tarball contains lexic-linux-<arch>/lexic
    BIN_PATH=$(find "${TMPDIR}" -name "$BIN_NAME" -type f | head -1)
    cp -f "${BIN_PATH}" "${INSTALL_DIR}/${BIN_NAME}"
    chmod +x "${INSTALL_DIR}/${BIN_NAME}"
    rm -rf "${TMPDIR}"
    ;;
  macos)
    ASSET="lexic-macos.zip"
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "${INSTALL_DIR}"
    TMPDIR="$(mktemp -d)"
    curl -fsSL "https://github.com/${REPO}/releases/download/${TAG}/${ASSET}" -o "${TMPDIR}/lexic.zip"
    (cd "${TMPDIR}" && unzip -o lexic.zip >/dev/null 2>&1 || tar -xzf lexic.zip)
    BIN_PATH=$(find "${TMPDIR}" -name "$BIN_NAME" -type f | head -1)
    cp -f "${BIN_PATH}" "${INSTALL_DIR}/${BIN_NAME}"
    chmod +x "${INSTALL_DIR}/${BIN_NAME}"
    rm -rf "${TMPDIR}"
    ;;
  windows)
    echo "NOTE: On Windows, download and run the NSIS installer from:" >&2
    echo "  https://github.com/${REPO}/releases/download/${TAG}/" >&2
    echo "Or use Chocolatey: choco install lexic" >&2
    exit 1
    ;;
esac

# Verify install
if command -v "${BIN_NAME}" >/dev/null 2>&1 || [ -x "${INSTALL_DIR}/${BIN_NAME}" ]; then
  echo "==> Lexic installed to ${INSTALL_DIR}/${BIN_NAME}"
  echo "==> Add ${INSTALL_DIR} to your PATH if it isn't already."
  echo "==> Run: ${BIN_NAME}"
else
  echo "ERROR: installation failed — ${BIN_NAME} not found after install" >&2
  exit 1
fi

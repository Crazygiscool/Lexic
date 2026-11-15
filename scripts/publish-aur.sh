#!/usr/bin/env bash
#
# publish-aur.sh — Publish Lexic to the Arch User Repository (AUR)
#
# Modeled after lumen's channel-gated approach:
#   Each channel is gated on its own secret (AUR_SSH_KEY here).
#
# Required environment:
#   AUR_SSH_KEY — SSH private key registered with AUR
#   AUR_PACKAGE — AUR package name (default: lexic)
#
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

VERSION="${VERSION#v}"
AUR_PACKAGE="${AUR_PACKAGE:-lexic}"

if [[ -z "${AUR_SSH_KEY:-}" ]]; then
  echo "WARNING: AUR_SSH_KEY not set, skipping AUR publish."
  exit 0
fi

echo "==> Publishing Lexic v${VERSION} to AUR..."

# Set up SSH key for AUR
mkdir -p ~/.ssh
echo "${AUR_SSH_KEY}" > ~/.ssh/aur_key
chmod 600 ~/.ssh/aur_key

# AUR uses SSH with the aur user
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/aur_key

# Clone the AUR package repo
AUR_URL="ssh://aur@aur.archlinux.org/${AUR_PACKAGE}.git"
AUR_TMP="$(mktemp -d)"
git clone "${AUR_URL}" "${AUR_TMP}"

# Copy PKGBUILD and .SRCINFO templates from the repo
if [[ -f "${REPO_ROOT}/scripts/aur/PKGBUILD" ]]; then
  cp "${REPO_ROOT}/scripts/aur/PKGBUILD" "${AUR_TMP}/PKGBUILD"
else
  echo "ERROR: PKGBUILD template not found at scripts/aur/PKGBUILD"
  exit 1
fi

# Update PKGBUILD version and source checksum
cd "${AUR_TMP}"
sed -i "s/^pkgver=.*/pkgver=${VERSION}/" PKGBUILD
sed -i "s/^pkgrel=.*/pkgrel=1/" PKGBUILD

# Download the source tarball from GitHub release to get its checksum
SOURCE_URL="https://github.com/Crazygiscool/Lexic/archive/refs/tags/v${VERSION}.tar.gz"
SOURCE_SHA256=$(curl -sL "${SOURCE_URL}" | sha256sum | awk '{print $1}')

sed -i "s|^source=.*|source=(\"https://github.com/Crazygiscool/Lexic/archive/refs/tags/v\${pkgver}.tar.gz\")|" PKGBUILD
sed -i "s|^sha256sums=.*|sha256sums=('${SOURCE_SHA256}')|" PKGBUILD

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push to AUR
git add PKGBUILD .SRCINFO
git commit -m "${AUR_PACKAGE}: v${VERSION}" || true
git push origin main

# Cleanup
cd "${REPO_ROOT}"
rm -rf "${AUR_TMP}"
ssh-agent -k

echo "==> Published ${AUR_PACKAGE} v${VERSION} to AUR"

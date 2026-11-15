#!/usr/bin/env bash
#
# publish-homebrew.sh — Update Homebrew tap formula for Lexic
#
# Modeled after lumen's Homebrew tap approach:
#   https://github.com/fzlzjerry/lumen/blob/main/docs/RELEASING.md
#
# Required environment:
#   HOMEBREW_TAP_TOKEN — PAT with contents:write on the tap repo
#   HOMEBREW_TAP_REPO  — Tap repository (default: Crazygiscool/homebrew-lexic)
#
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

VERSION="${VERSION#v}"
HOMEBREW_TAP_REPO="${HOMEBREW_TAP_REPO:-Crazygiscool/homebrew-lexic}"

if [[ -z "${HOMEBREW_TAP_TOKEN:-}" ]]; then
  echo "WARNING: HOMEBREW_TAP_TOKEN not set, skipping Homebrew publish."
  exit 0
fi

echo "==> Updating Homebrew tap formula for Lexic v${VERSION}..."

# Download the macOS release asset to get its checksum
# We need the x86_64-apple-darwin and aarch64-apple-darwin tarballs
ASSET_INTEL="https://github.com/Crazygiscool/Lexic/releases/download/v${VERSION}/lexic-x86_64-apple-darwin.tar.gz"
ASSET_ARM="https://github.com/Crazygiscool/Lexic/releases/download/v${VERSION}/lexic-aarch64-apple-darwin.tar.gz"

SHA256_INTEL=$(curl -sL "${ASSET_INTEL}" | sha256sum | awk '{print $1}')
SHA256_ARM=$(curl -sL "${ASSET_ARM}" | sha256sum | awk '{print $1}')

# Clone the Homebrew tap
TAP_TMP="$(mktemp -d)"
git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/${HOMEBREW_TAP_REPO}.git" "${TAP_TMP}"

# Write the formula
cat > "${TAP_TMP}/Formula/lexic.rb" << EOF
class Lexic < Formula
  desc "Offline-first dictionary + SM-2 flashcard desktop app for humanities study"
  homepage "https://github.com/Crazygiscool/Lexic"
  version "${VERSION}"

  on_intel do
    url "https://github.com/Crazygiscool/Lexic/releases/download/v${VERSION}/lexic-x86_64-apple-darwin.tar.gz"
    sha256 "${SHA256_INTEL}"
  end

  on_arm do
    url "https://github.com/Crazygiscool/Lexic/releases/download/v${VERSION}/lexic-aarch64-apple-darwin.tar.gz"
    sha256 "${SHA256_ARM}"
  end

  def install
    bin.install "lexic"
  end
end
EOF

# Commit and push
cd "${TAP_TMP}"
git add Formula/lexic.rb
git commit -m "${HOMEBREW_TAP_REPO##*/}: bump lexic to ${VERSION}" || true
git push origin main

# Cleanup
cd "${REPO_ROOT}"
rm -rf "${TAP_TMP}"

echo "==> Homebrew tap updated: brew install Crazygiscool/lexic/lexic"

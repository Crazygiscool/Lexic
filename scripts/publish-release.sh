#!/usr/bin/env bash
#
# publish-release.sh — Publish Lexic to all package channels
#
# Modeled after lumen's release approach:
#   https://github.com/fzlzjerry/lumen/blob/main/docs/RELEASING.md
#
# Cutting a release:
#   1. Bump version in Cargo.toml
#   2. Commit with "[release]" keyword on main
#   3. Push — the GitHub Actions workflow handles the rest
#
# Or run this script directly:
#   ./scripts/publish-release.sh <version>
#
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Strip leading 'v' if present
VERSION="${VERSION#v}"

echo "=============================================="
echo "  Lexic Release Script"
echo "=============================================="
echo "Version: ${VERSION}"
echo "Repo:    ${REPO_ROOT}"
echo ""

# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------
for cmd in git cargo sha256sum; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "ERROR: Required command not found: ${cmd}"
    exit 1
  fi
done

# Verify we're on a clean tree
if ! git -C "${REPO_ROOT}" diff --quiet || ! git -C "${REPO_ROOT}" diff --cached --quiet; then
  echo "ERROR: Working tree is not clean. Commit or stash changes first."
  git -C "${REPO_ROOT}" status --short
  exit 1
fi

# ---------------------------------------------------------------------------
# 1. Update Cargo.toml version across all crates
# ---------------------------------------------------------------------------
echo "==> Updating version to ${VERSION}..."
sed -i "s/^version = .*/version = \"${VERSION}\"/" "${REPO_ROOT}/crates/lexic-core/Cargo.toml"
sed -i "s/^version = .*/version = \"${VERSION}\"/" "${REPO_ROOT}/crates/lexic-store/Cargo.toml"
sed -i "s/^version = .*/version = \"${VERSION}\"/" "${REPO_ROOT}/crates/lexic-dict/Cargo.toml"
sed -i "s/^version = .*/version = \"${VERSION}\"/" "${REPO_ROOT}/crates/lexic-gui/Cargo.toml"
sed -i "s/pub const VERSION: &str = .*/pub const VERSION: \&str = \"${VERSION}\";/" "${REPO_ROOT}/crates/lexic-core/src/lib.rs"
sed -i "s/pub const VERSION: &str = .*/pub const VERSION: \&str = \"${VERSION}\";/" "${REPO_ROOT}/crates/lexic-store/src/lib.rs"
sed -i "s/pub const VERSION: &str = .*/pub const VERSION: \&str = \"${VERSION}\";/" "${REPO_ROOT}/crates/lexic-dict/src/lib.rs"
sed -i "s/^version = .*/version = \"${VERSION}\"/" "${REPO_ROOT}/Packager.toml"

# Update Cargo.lock
cargo generate-lockfile --manifest-path "${REPO_ROOT}/Cargo.toml"

# ---------------------------------------------------------------------------
# 2. Commit version changes
# ---------------------------------------------------------------------------
echo "==> Committing version changes..."
git -C "${REPO_ROOT}" add Cargo.toml Cargo.lock crates/*/Cargo.toml crates/*/src/lib.rs Packager.toml
git -C "${REPO_ROOT}" commit -m "chore: bump version to ${VERSION}" || true

# ---------------------------------------------------------------------------
# 3. Tag and push (triggers the release workflow)
# ---------------------------------------------------------------------------
echo "==> Tagging and pushing v${VERSION}..."
git -C "${REPO_ROOT}" tag -a "v${VERSION}" -m "Release v${VERSION}"
git -C "${REPO_ROOT}" push origin main
git -C "${REPO_ROOT}" push origin "v${VERSION}"

echo ""
echo "=============================================="
echo "  Release v${VERSION} triggered!"
echo "=============================================="
echo "  The GitHub Actions release workflow will now:"
echo "    1. Build, test, and lint the project"
echo "    2. Build release binaries for Linux/macOS/Windows"
echo "    3. Create GitHub Release with assets"
echo "    4. Publish to crates.io (if CARGO_REGISTRY_TOKEN set)"
echo "    5. Update Homebrew tap (if HOMEBREW_TAP_TOKEN set)"
echo "    6. Publish to Chocolatey (if CHOCOLATEY_API_KEY set)"
echo "    7. Publish to AUR (if AUR_SSH_KEY set)"
echo "    8. Build Windows NSIS installer (Windows runner)"
echo "=============================================="

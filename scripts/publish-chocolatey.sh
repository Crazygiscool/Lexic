#!/usr/bin/env bash
#
# publish-chocolatey.sh — Publish Lexic to Chocolatey
#
# Modeled after ddev's Chocolatey publishing approach:
#   https://github.com/ddev/ddev/blob/main/docs/content/developers/release-management.md
#
# Required environment:
#   CHOCOLATEY_API_KEY — API key for push.chocolatey.org
#
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

VERSION="${VERSION#v}"

if [[ -z "${CHOCOLATEY_API_KEY:-}" ]]; then
  echo "WARNING: CHOCOLATEY_API_KEY not set, skipping Chocolatey publish."
  exit 0
fi

echo "==> Publishing Lexic v${VERSION} to Chocolatey..."

# Chocolatey requires Windows to build .nupkg
# We build the nupkg on Windows in the CI job, then push from Linux here
CHOCO_TMP="$(mktemp -d)"

# The .nuspec file describes the Chocolatey package
cat > "${CHOCO_TMP}/lexic.nuspec" << EOF
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>lexic</id>
    <title>Lexic</title>
    <version>${VERSION}</version>
    <authors>Crazygiscool</authors>
    <owners>Crazygiscool</owners>
    <projectUrl>https://github.com/Crazygiscool/Lexic</projectUrl>
    <iconUrl>https://raw.githubusercontent.com/Crazygiscool/Lexic/main/assets/icons/lexic.png</iconUrl>
    <licenseUrl>https://raw.githubusercontent.com/Crazygiscool/Lexic/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <description>Offline-first dictionary + SM-2 flashcard desktop app for humanities study.</description>
    <releaseNotes>https://github.com/Crazygiscool/Lexic/releases/tag/v${VERSION}</releaseNotes>
    <tags>lexic dictionary flashcards srs humanities study</tags>
  </metadata>
  <files>
    <file src="tools\**" target="tools" />
  </files>
</package>
EOF

# Chocolatey install script
mkdir -p "${CHOCO_TMP}/tools"
cat > "${CHOCO_TMP}/tools/chocolateyinstall.ps1" << 'EOF'
$ErrorActionPreference = 'Stop';
$toolsDir   = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64      = "https://github.com/Crazygiscool/Lexic/releases/download/v${VERSION}/lexic-windows.zip"
$checksum64 = "{{CHECKSUM64}}"
$checksumType64 = "sha256"

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  unzipLocation = $toolsDir
  url64bit      = $url64
  checksum64    = $checksum64
  checksumType64= $checksumType64
  destination   = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
}

Install-ChocolateyZipPackage @packageArgs

# Find the exe and create a shim
$exePath = Get-ChildItem -Path $toolsDir -Filter "*.exe" -Recurse | Select-Object -First 1
if ($exePath) {
  Install-BinFile -Name "lexic" -Path $exePath.FullName
}
EOF

# Replace the checksum placeholder with the actual checksum from the release
# The Windows release asset URL
ASSET_URL="https://github.com/Crazygiscool/Lexic/releases/download/v${VERSION}/lexic-windows.zip"
CHECKSUM64=$(curl -sL "${ASSET_URL}" | sha256sum | awk '{print $1}')
sed -i "s/{{CHECKSUM64}}/${CHECKSUM64}/" "${CHOCO_TMP}/tools/chocolateyinstall.ps1"

# Build the .nupkg using chocolatey pack
if command -v choco >/dev/null 2>&1; then
  cd "${CHOCO_TMP}"
  choco pack lexic.nuspec
  NUPKG=$(ls *.nupkg | head -1)
  
  # Push to Chocolatey
  choco apikey --key "${CHOCOLATEY_API_KEY}" --source push.chocolatey.org/
  choco push "${NUPKG}" --source push.chocolatey.org/
  
  echo "==> Published lexic v${VERSION} to Chocolatey"
else
  echo "WARNING: choco command not found. Build the .nupkg manually from ${CHOCO_TMP}"
  echo "   Then push with: choco push <nupkg> --source push.chocolatey.org/ --api-key <key>"
fi

# Cleanup
rm -rf "${CHOCO_TMP}"

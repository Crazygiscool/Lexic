#!/usr/bin/env bash
#
# build-nsis.sh — Build Windows NSIS installer for Lexic
#
# Required on Windows runner:
#   - NSIS installed (choco install nsis -y)
#   - The lexic-windows.zip release asset already built
#
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

VERSION="${VERSION#v}"

echo "==> Building Windows NSIS installer for Lexic v${VERSION}..."

DIST_DIR="${REPO_ROOT}/dist"
WIN_ASSETS="${DIST_DIR}/lexic-windows"

if [ ! -d "$WIN_ASSETS" ]; then
  echo "ERROR: missing Windows assets at ${WIN_ASSETS}. Run the Package/Extract Windows assets steps first." >&2
  exit 1
fi

# Create the NSIS installer script inside dist/ so that NSIS resolves relative
# paths (OutFile, File) against dist/, where dist/lexic-windows/ actually lives.
cat > "${DIST_DIR}/lexic.nsi" << 'EOF'
!define NAME "Lexic"
!define VERSION "__LEXIC_VERSION__"
!define PUBLISHER "Crazygiscool"
!define WEBSITE "https://github.com/Crazygiscool/Lexic"
!define INSTALLER_NAME "lexic-${VERSION}-windows-installer.exe"

!include "MUI2.nsh"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Name "${NAME} ${VERSION}"
OutFile "lexic-${VERSION}-windows-installer.exe"
InstallDir "\$PROGRAMFILES64\\${NAME}"
ShowInstDetails show

Section "MainSection" SEC01
  SetOutPath "\$INSTDIR"
  File "lexic-windows/lexic-gui.exe"

  ; Create Start Menu shortcuts
  CreateShortCut "\$SMPROGRAMS\\${NAME}.lnk" "\$INSTDIR\\lexic-gui.exe"

  ; Create uninstaller
  WriteUninstaller "\$INSTDIR\\Uninstall.exe"

  ; Registry entries
  WriteRegStr HKLM "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${NAME}" "DisplayName" "\${NAME} \${VERSION}"
  WriteRegStr HKLM "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${NAME}" "DisplayVersion" "\${VERSION}"
  WriteRegStr HKLM "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${NAME}" "Publisher" "\${PUBLISHER}"
  WriteRegStr HKLM "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${NAME}" "InstallLocation" "\$INSTDIR"
  WriteRegStr HKLM "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${NAME}" "UninstallString" "\$INSTDIR\\Uninstall.exe"
  WriteRegStr HKLM "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${NAME}" "URLInfoAbout" "\${WEBSITE}"
SectionEnd

Section "Uninstall"
  Delete "\$INSTDIR\\lexic-gui.exe"
  Delete "\$INSTDIR\\Uninstall.exe"
  RMDir "\$INSTDIR"
  Delete "\$SMPROGRAMS\\${NAME}.lnk"

  DeleteRegKey HKLM "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${NAME}"
SectionEnd
EOF

# Inject the real version (passed as $1) into the NSIS script. The quoted
# heredoc above prevents bash from expanding ${VERSION}, so NSIS would see an
# empty self-referential !define. Substitute the placeholder after writing.
sed -i "s/__LEXIC_VERSION__/$VERSION/" "${DIST_DIR}/lexic.nsi"

# Build the installer with NSIS
MAKENSIS="$(command -v makensis 2>/dev/null || true)"
if [ -z "$MAKENSIS" ]; then
  # NSIS installs makensis to a Program Files dir not always on PATH; probe it.
  for p in "/c/Program Files/NSIS/makensis.exe" "/c/Program Files (x86)/NSIS/makensis.exe"; do
    if [ -f "$p" ]; then MAKENSIS="$p"; break; fi
  done
fi

INSTALLER="${DIST_DIR}/lexic-${VERSION}-windows-installer.exe"

if [ -n "$MAKENSIS" ]; then
  # Run makensis from dist/ so relative paths in the .nsi resolve against dist/.
  ( cd "${DIST_DIR}" && "$MAKENSIS" lexic.nsi )
  rm -f "${DIST_DIR}/lexic.nsi"
  if [ ! -f "$INSTALLER" ]; then
    echo "ERROR: makensis ran but ${INSTALLER} was not created" >&2
    exit 1
  fi
  echo "==> NSIS installer built: ${INSTALLER}"
else
  echo "ERROR: makensis not found. Install NSIS (choco install nsis) to build the Windows installer." >&2
  echo "   NSIS script saved to: ${DIST_DIR}/lexic.nsi" >&2
  exit 1
fi

content = r'''#!/usr/bin/env bash
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

# Create the NSIS installer script
cat > "${REPO_ROOT}/scripts/lexic.nsi" << EOF
!define NAME "Lexic"
!define VERSION "${VERSION}"
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

Name "\\${NAME} \${VERSION}"
OutFile "\${REPO_ROOT}/dist/lexic-\${VERSION}-windows-installer.exe"
InstallDir "\$PROGRAMFILES64\\${NAME}"
ShowInstDetails show

Section "MainSection" SEC01
  SetOutPath "\$INSTDIR"
  File "/r=1 \${REPO_ROOT}/dist/lexic-windows/*"

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

# Build the installer with NSIS
if command -v makensis >/dev/null 2>&1; then
  makensis "${REPO_ROOT}/scripts/lexic.nsi"
  echo "==> NSIS installer built: dist/lexic-${VERSION}-windows-installer.exe"
else
  echo "WARNING: makensis not found. Install NSIS (choco install nsis) to build the Windows installer."
  echo "   NSIS script saved to: scripts/lexic.nsi"
fi
'''
with open('scripts/build-nsis.sh', 'w') as f:
    f.write(content)
print("done")
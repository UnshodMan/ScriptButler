; Installer hooks for sbctl PATH integration.
; Included through bundle.windows.nsis.installerHooks (absolute path),
; which is the only reliable way to pull in extra files from the project.
; sbctl.nsh (page + localized strings) lives next to this file.

!addincludedir "${__FILEDIR__}"
!include "sbctl.nsh"

!macro NSIS_HOOK_POSTINSTALL
  ${If} $SbctlAddToPath != "0"
    DetailPrint "Adding sbctl to PATH..."
    nsExec::ExecToLog '"$INSTDIR\sbctl.exe" env add'
    Pop $0
    ${If} $0 != "0"
      DetailPrint "sbctl env add exited with $0"
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog '"$INSTDIR\sbctl.exe" env remove'
  Pop $0
!macroend

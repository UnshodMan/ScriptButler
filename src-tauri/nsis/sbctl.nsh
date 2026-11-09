; sbctl PATH option page and localized strings.
; Included by installer.nsi; keep this file version-agnostic.

Var SbctlAddToPath
Var SbctlPathCheckbox

LangString sbctlPathTitle ${LANG_ENGLISH} "Command-line tool"
LangString sbctlPathSubtitle ${LANG_ENGLISH} "Choose whether to make the sbctl command available in your terminal."
LangString sbctlPathCheckbox ${LANG_ENGLISH} "Add sbctl to PATH (recommended)"

LangString sbctlPathTitle ${LANG_SIMPCHINESE} "命令行工具"
LangString sbctlPathSubtitle ${LANG_SIMPCHINESE} "选择是否让终端可以直接使用 sbctl 命令。"
LangString sbctlPathCheckbox ${LANG_SIMPCHINESE} "将 sbctl 加入 PATH（推荐）"

LangString sbctlPathTitle ${LANG_TRADCHINESE} "命令列工具"
LangString sbctlPathSubtitle ${LANG_TRADCHINESE} "選擇是否讓終端機可以直接使用 sbctl 命令。"
LangString sbctlPathCheckbox ${LANG_TRADCHINESE} "將 sbctl 加入 PATH（推薦）"

LangString sbctlPathTitle ${LANG_JAPANESE} "コマンドラインツール"
LangString sbctlPathSubtitle ${LANG_JAPANESE} "ターミナルから sbctl コマンドを利用できるようにします。"
LangString sbctlPathCheckbox ${LANG_JAPANESE} "sbctl を PATH に追加する（推奨）"

LangString sbctlPathTitle ${LANG_KOREAN} "명령줄 도구"
LangString sbctlPathSubtitle ${LANG_KOREAN} "터미널에서 sbctl 명령을 사용할 수 있도록 합니다."
LangString sbctlPathCheckbox ${LANG_KOREAN} "sbctl을 PATH에 추가(권장)"

LangString sbctlPathTitle ${LANG_SPANISH} "Herramienta de línea de comandos"
LangString sbctlPathSubtitle ${LANG_SPANISH} "Permite usar el comando sbctl desde la terminal."
LangString sbctlPathCheckbox ${LANG_SPANISH} "Añadir sbctl al PATH (recomendado)"

LangString sbctlPathTitle ${LANG_FRENCH} "Outil en ligne de commande"
LangString sbctlPathSubtitle ${LANG_FRENCH} "Permet d'utiliser la commande sbctl depuis le terminal."
LangString sbctlPathCheckbox ${LANG_FRENCH} "Ajouter sbctl au PATH (recommandé)"

LangString sbctlPathTitle ${LANG_GERMAN} "Kommandozeilenwerkzeug"
LangString sbctlPathSubtitle ${LANG_GERMAN} "Ermöglicht die Nutzung von sbctl im Terminal."
LangString sbctlPathCheckbox ${LANG_GERMAN} "sbctl zum PATH hinzufügen (empfohlen)"

LangString sbctlPathTitle ${LANG_RUSSIAN} "Инструмент командной строки"
LangString sbctlPathSubtitle ${LANG_RUSSIAN} "Позволяет использовать команду sbctl в терминале."
LangString sbctlPathCheckbox ${LANG_RUSSIAN} "Добавить sbctl в PATH (рекомендуется)"

Function SbctlPathPage
  ${If} $PassiveMode = 1
    Abort
  ${EndIf}
  ${If} ${Silent}
    Abort
  ${EndIf}

  !insertmacro MUI_HEADER_TEXT "$(sbctlPathTitle)" "$(sbctlPathSubtitle)"

  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}

  ${NSD_CreateCheckbox} 0 0 100% 12u "$(sbctlPathCheckbox)"
  Pop $SbctlPathCheckbox
  ${If} $SbctlAddToPath == "0"
    ${NSD_Uncheck} $SbctlPathCheckbox
  ${Else}
    ${NSD_Check} $SbctlPathCheckbox
  ${EndIf}

  nsDialogs::Show
FunctionEnd

Function SbctlPathPageLeave
  ${NSD_GetState} $SbctlPathCheckbox $0
  ${If} $0 == ${BST_CHECKED}
    StrCpy $SbctlAddToPath 1
  ${Else}
    StrCpy $SbctlAddToPath 0
  ${EndIf}
FunctionEnd

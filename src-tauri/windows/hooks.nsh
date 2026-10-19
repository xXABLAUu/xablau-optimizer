; Ganchos do instalador NSIS do Xablau Optimizer (usados pelo Tauri em bundle > windows > nsis > installerHooks).

!macro NSIS_HOOK_PREINSTALL
  ; Fecha o vigia e o kit de emergência para os arquivos poderem ser substituídos.
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM xablau-vigia.exe'
  Pop $0
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM xablau-restaurar.exe'
  Pop $0
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Se o "Iniciar com o Windows" estiver ligado, a tarefa agendada religa o vigia na hora.
  nsExec::Exec '"$SYSDIR\schtasks.exe" /Run /TN "\XablauOptimizer\Vigia"'
  Pop $0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM xablau-vigia.exe'
  Pop $0
  ; Durante uma ATUALIZAÇÃO o desinstalador antigo também roda: aí nada é apagado.
  ${If} $UpdateMode <> 1
    nsExec::Exec '"$SYSDIR\schtasks.exe" /Delete /TN "\XablauOptimizer\Vigia" /F'
    Pop $0
    nsExec::Exec '"$SYSDIR\schtasks.exe" /Delete /TN "\XablauOptimizer\Manutencao" /F'
    Pop $0
    Delete "$SMPROGRAMS\Xablau - Restaurar tudo.lnk"
    ; Os backups e o diário em C:\ProgramData\XablauOptimizer são mantidos de propósito,
    ; para ainda ser possível desfazer as alterações depois de desinstalar.
  ${EndIf}
!macroend

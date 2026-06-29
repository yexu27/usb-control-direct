!macro customInit
  UserInfo::GetAccountType
  Pop $0
  ${If} $0 != "Admin"
    MessageBox MB_ICONSTOP "安装 USB安全管理系统 需要管理员权限。"
    Abort
  ${EndIf}
!macroend

!macro customUnInstall
  RMDir /r "$APPDATA\USB安全管理系统"
  RMDir /r "$LOCALAPPDATA\USB安全管理系统"
!macroend

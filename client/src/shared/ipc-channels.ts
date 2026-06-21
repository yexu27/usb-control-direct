export const IpcChannels = {
  tlsConnect: 'tls:connect',
  tlsDisconnect: 'tls:disconnect',
  tlsSend: 'tls:send',
  tlsApplyStateEvent: 'tls:apply-state-event',
  connectionStateChanged: 'connection:state-changed',
  dialogOpenFile: 'dialog:open-file',
  dialogSaveFile: 'dialog:save-file',
  dialogReadFile: 'dialog:read-file',
  dialogWriteFile: 'dialog:write-file',
} as const

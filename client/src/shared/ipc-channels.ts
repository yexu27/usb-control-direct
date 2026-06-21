export const IpcChannels = {
  tlsConnect: 'tls:connect',
  tlsDisconnect: 'tls:disconnect',
  tlsSend: 'tls:send',
  connectionStateChanged: 'connection:state-changed',
  dialogOpenFile: 'dialog:open-file',
  dialogSaveFile: 'dialog:save-file',
} as const

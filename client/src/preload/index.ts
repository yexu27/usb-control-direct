import { contextBridge, ipcRenderer, type IpcRendererEvent } from 'electron'
import type { ConnectionEvent, ConnectionStatus } from '../shared/connection-state'
import type { TlsResponse } from '../shared/tls-response'
import { IpcChannels } from '../shared/ipc-channels'

export interface OpenFileOptions {
  title?: string
  filters?: Array<{ name: string; extensions: string[] }>
}

export interface SaveFileOptions {
  title?: string
  defaultPath?: string
  filters?: Array<{ name: string; extensions: string[] }>
}

const desktopApi = {
  tls: {
    connect: (ip: string): Promise<void> => ipcRenderer.invoke(IpcChannels.tlsConnect, ip),

    disconnect: (): Promise<void> => ipcRenderer.invoke(IpcChannels.tlsDisconnect),

    applyStateEvent: (event: ConnectionEvent): Promise<void> =>
      ipcRenderer.invoke(IpcChannels.tlsApplyStateEvent, event),

    send: (
      msgType: number,
      payload: Uint8Array,
      timeout?: number,
    ): Promise<TlsResponse> =>
      ipcRenderer.invoke(IpcChannels.tlsSend, msgType, payload, timeout),

    onStateChanged: (callback: (status: ConnectionStatus) => void): (() => void) => {
      const handler = (_event: IpcRendererEvent, status: ConnectionStatus) => callback(status)
      ipcRenderer.on(IpcChannels.connectionStateChanged, handler)
      return () => {
        ipcRenderer.removeListener(IpcChannels.connectionStateChanged, handler)
      }
    },
  },

  dialog: {
    openFile: (options: OpenFileOptions): Promise<{ canceled: boolean; filePaths: string[] }> =>
      ipcRenderer.invoke(IpcChannels.dialogOpenFile, options),

    saveFile: (options: SaveFileOptions): Promise<{ canceled: boolean; filePath?: string }> =>
      ipcRenderer.invoke(IpcChannels.dialogSaveFile, options),

    readFile: (filePath: string): Promise<Uint8Array> =>
      ipcRenderer.invoke(IpcChannels.dialogReadFile, filePath),

    writeFile: (filePath: string, content: Uint8Array): Promise<void> =>
      ipcRenderer.invoke(IpcChannels.dialogWriteFile, filePath, content),
  },
}

contextBridge.exposeInMainWorld('desktopApi', desktopApi)

export type DesktopApi = typeof desktopApi

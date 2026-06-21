import { contextBridge, ipcRenderer, type IpcRendererEvent } from 'electron'
import type { ConnectionStatus } from '../shared/connection-state'

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
    connect: (ip: string): Promise<void> => ipcRenderer.invoke('tls:connect', ip),

    disconnect: (): Promise<void> => ipcRenderer.invoke('tls:disconnect'),

    send: (
      msgType: number,
      payload: Uint8Array,
      timeout?: number,
    ): Promise<Uint8Array> => ipcRenderer.invoke('tls:send', msgType, payload, timeout),

    onStateChanged: (callback: (status: ConnectionStatus) => void): (() => void) => {
      const handler = (_event: IpcRendererEvent, status: ConnectionStatus) => callback(status)
      ipcRenderer.on('connection:state-changed', handler)
      return () => {
        ipcRenderer.removeListener('connection:state-changed', handler)
      }
    },
  },

  dialog: {
    openFile: (options: OpenFileOptions): Promise<{ canceled: boolean; filePaths: string[] }> =>
      ipcRenderer.invoke('dialog:open-file', options),

    saveFile: (options: SaveFileOptions): Promise<{ canceled: boolean; filePath?: string }> =>
      ipcRenderer.invoke('dialog:save-file', options),
  },
}

contextBridge.exposeInMainWorld('desktopApi', desktopApi)

export type DesktopApi = typeof desktopApi

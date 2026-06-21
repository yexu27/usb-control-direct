import { ipcMain, dialog, type BrowserWindow } from 'electron'
import { IpcChannels } from '../../shared/ipc-channels'
import { SelectedFileAccess } from './selected-file-access'
import { assertTrustedSender } from './trusted-sender'

interface FileFilter {
  name: string
  extensions: string[]
}

interface OpenFileOptions {
  title?: string
  filters?: FileFilter[]
}

interface SaveFileOptions extends OpenFileOptions {
  defaultPath?: string
}

function parseFilters(input: unknown): FileFilter[] | undefined {
  if (!Array.isArray(input)) {
    return undefined
  }

  const filters: FileFilter[] = []
  for (const item of input) {
    if (typeof item !== 'object' || item == null) {
      throw new Error('文件类型过滤条件无效')
    }

    const candidate = item as Record<string, unknown>
    if (
      typeof candidate.name !== 'string' ||
      !Array.isArray(candidate.extensions) ||
      !candidate.extensions.every((extension) => typeof extension === 'string')
    ) {
      throw new Error('文件类型过滤条件无效')
    }

    filters.push({
      name: candidate.name,
      extensions: candidate.extensions,
    })
  }

  return filters
}

function parseOpenFileOptions(input: unknown): OpenFileOptions {
  if (typeof input !== 'object' || input == null) {
    return {}
  }

  const candidate = input as Record<string, unknown>
  return {
    title: typeof candidate.title === 'string' ? candidate.title : undefined,
    filters: parseFilters(candidate.filters),
  }
}

function parseSaveFileOptions(input: unknown): SaveFileOptions {
  const options = parseOpenFileOptions(input)
  const candidate = typeof input === 'object' && input != null ? (input as Record<string, unknown>) : {}
  return {
    ...options,
    defaultPath: typeof candidate.defaultPath === 'string' ? candidate.defaultPath : undefined,
  }
}

export function registerDialogIpc(getMainWindow: () => BrowserWindow | null): void {
  const selectedFileAccess = new SelectedFileAccess()

  ipcMain.handle(IpcChannels.dialogOpenFile, async (event, options: unknown) => {
    const mainWindow = getMainWindow()
    assertTrustedSender(event, mainWindow)
    const result = await dialog.showOpenDialog(mainWindow, {
      ...parseOpenFileOptions(options),
      properties: ['openFile'],
    })
    selectedFileAccess.allowRead(result.filePaths)
    return {
      canceled: result.canceled,
      filePaths: result.filePaths,
    }
  })

  ipcMain.handle(IpcChannels.dialogSaveFile, async (event, options: unknown) => {
    const mainWindow = getMainWindow()
    assertTrustedSender(event, mainWindow)
    const result = await dialog.showSaveDialog(mainWindow, {
      ...parseSaveFileOptions(options),
    })
    if (result.filePath != null) {
      selectedFileAccess.allowWrite(result.filePath)
    }
    return {
      canceled: result.canceled,
      filePath: result.filePath,
    }
  })

  ipcMain.handle(IpcChannels.dialogReadFile, async (event, filePath: unknown) => {
    assertTrustedSender(event, getMainWindow())
    return selectedFileAccess.readSelectedFile(filePath)
  })

  ipcMain.handle(
    IpcChannels.dialogWriteFile,
    async (event, filePath: unknown, content: unknown) => {
      assertTrustedSender(event, getMainWindow())
      await selectedFileAccess.writeSelectedFile(filePath, content)
    },
  )
}

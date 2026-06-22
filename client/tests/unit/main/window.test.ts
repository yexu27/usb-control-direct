import { beforeEach, describe, expect, it, vi } from 'vitest'

const loadURL = vi.fn()
const loadFile = vi.fn()
const on = vi.fn()
const browserWindow = vi.fn()

vi.mock('electron', () => ({
  app: {
    isPackaged: true,
  },
  BrowserWindow: browserWindow,
}))

browserWindow.mockImplementation(() => ({
  loadURL,
  loadFile,
  on,
}))

describe('main window', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('uses the prototype app surface as the default BrowserWindow size', async () => {
    const { createMainWindow } = await import('../../../src/main/window')

    createMainWindow()

    expect(browserWindow).toHaveBeenCalledWith(
      expect.objectContaining({
        width: 1120,
        height: 700,
        minWidth: 1024,
        minHeight: 700,
        frame: false,
      }),
    )
  })
})

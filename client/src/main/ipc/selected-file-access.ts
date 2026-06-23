import { isAbsolute } from 'node:path'
import { readFile, writeFile } from 'node:fs/promises'

export class SelectedFileAccess {
  private readonly readablePaths = new Set<string>()
  private readonly writablePaths = new Set<string>()

  allowRead(filePaths: string[]): void {
    for (const filePath of filePaths) {
      if (isAbsolute(filePath)) {
        this.readablePaths.add(filePath)
      }
    }
  }

  allowWrite(filePath: string): void {
    if (isAbsolute(filePath)) {
      this.writablePaths.add(filePath)
    }
  }

  revoke(input: unknown): void {
    if (typeof input !== 'string' || !isAbsolute(input)) {
      throw new Error('文件路径无效')
    }

    this.readablePaths.delete(input)
    this.writablePaths.delete(input)
  }

  async readSelectedFile(input: unknown): Promise<Uint8Array> {
    const filePath = this.parseSelectedPath(input, this.readablePaths)
    const fileContent = await readFile(filePath)
    return new Uint8Array(fileContent)
  }

  async writeSelectedFile(filePathInput: unknown, contentInput: unknown): Promise<void> {
    const filePath = this.parseSelectedPath(filePathInput, this.writablePaths)
    if (!(contentInput instanceof Uint8Array)) {
      throw new Error('写入内容格式无效')
    }

    await writeFile(filePath, contentInput)
  }

  private parseSelectedPath(input: unknown, allowedPaths: Set<string>): string {
    if (typeof input !== 'string' || !isAbsolute(input)) {
      throw new Error('文件路径无效')
    }

    if (!allowedPaths.delete(input)) {
      throw new Error('文件路径未经用户选择')
    }

    return input
  }
}

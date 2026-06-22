function basename(filePath: string): string {
  return filePath.split(/[\\/]/).pop() ?? filePath
}

function parseVersion(filePath: string, pattern: RegExp, errorMessage: string): string {
  const fileName = basename(filePath)
  const match = fileName.match(pattern)

  if (match == null || match[1] == null) {
    throw new Error(errorMessage)
  }

  return match[1]
}

export function parseSystemUpgradeVersion(filePath: string): string {
  return parseVersion(
    filePath,
    /^usb-control-system-(v\d+\.\d+\.\d+)\.bin$/i,
    '系统升级包文件名格式错误，请使用 usb-control-system-vX.Y.Z.bin',
  )
}

export function parseVirusdbUpgradeVersion(filePath: string): string {
  return parseVersion(
    filePath,
    /^usb-control-virusdb-(v\d+\.\d+\.\d+)\.zip$/i,
    '病毒库升级包文件名格式错误，请使用 usb-control-virusdb-vX.Y.Z.zip',
  )
}

export async function calculateSha256Hex(bytes: Uint8Array): Promise<string> {
  const buffer = new ArrayBuffer(bytes.byteLength)
  new Uint8Array(buffer).set(bytes)
  const digest = await crypto.subtle.digest('SHA-256', buffer)

  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')
}

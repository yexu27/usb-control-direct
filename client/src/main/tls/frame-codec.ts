export const FRAME_MAGIC = 0x55534243
export const FRAME_HEADER_SIZE = 20
export const MAX_PAYLOAD_SIZE = 128 * 1024 * 1024

export interface FrameHeader {
  magic: number
  msgType: number
  seqId: number
  payloadLen: number
  crc32: number
}

const CRC32_TABLE = buildCrc32Table()

function buildCrc32Table(): Uint32Array {
  const table = new Uint32Array(256)
  for (let i = 0; i < 256; i++) {
    let crc = i
    for (let j = 0; j < 8; j++) {
      crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1
    }
    table[i] = crc >>> 0
  }
  return table
}

export function calculateCrc32(data: Uint8Array): number {
  if (data.length === 0) {
    return 0x00000000
  }
  let crc = 0xffffffff
  for (let i = 0; i < data.length; i++) {
    crc = (crc >>> 8) ^ CRC32_TABLE[(crc ^ data[i]) & 0xff]
  }
  return (crc ^ 0xffffffff) >>> 0
}

export function encodeFrame(msgType: number, seqId: number, payload: Uint8Array): Buffer {
  if (payload.length > MAX_PAYLOAD_SIZE) {
    throw new Error(`消息载荷超过上限：${MAX_PAYLOAD_SIZE} 字节`)
  }

  const frame = Buffer.alloc(FRAME_HEADER_SIZE + payload.length)
  frame.writeUInt32BE(FRAME_MAGIC, 0)
  frame.writeUInt32BE(msgType, 4)
  frame.writeUInt32BE(seqId, 8)
  frame.writeUInt32BE(payload.length, 12)
  frame.writeUInt32BE(calculateCrc32(payload), 16)
  if (payload.length > 0) {
    frame.set(payload, FRAME_HEADER_SIZE)
  }
  return frame
}

export function decodeFrameHeader(buffer: Buffer): FrameHeader | null {
  if (buffer.length < FRAME_HEADER_SIZE) {
    return null
  }
  return {
    magic: buffer.readUInt32BE(0),
    msgType: buffer.readUInt32BE(4),
    seqId: buffer.readUInt32BE(8),
    payloadLen: buffer.readUInt32BE(12),
    crc32: buffer.readUInt32BE(16),
  }
}

export class FrameStreamParser {
  private buffer: Buffer = Buffer.alloc(0)
  onFrame: ((header: FrameHeader, payload: Uint8Array) => void) | null = null

  feed(chunk: Buffer): void {
    this.buffer = Buffer.concat([this.buffer, chunk])
    this.tryParse()
  }

  private tryParse(): void {
    while (this.buffer.length >= FRAME_HEADER_SIZE) {
      const header = decodeFrameHeader(this.buffer)
      if (header == null) {
        break
      }

      if (header.magic !== FRAME_MAGIC) {
        this.buffer = this.buffer.subarray(4)
        continue
      }

      if (header.payloadLen > MAX_PAYLOAD_SIZE) {
        this.buffer = this.buffer.subarray(4)
        continue
      }

      const totalLen = FRAME_HEADER_SIZE + header.payloadLen
      if (this.buffer.length < totalLen) {
        break
      }

      const payload = new Uint8Array(this.buffer.subarray(FRAME_HEADER_SIZE, totalLen))

      const expectedCrc = calculateCrc32(payload)
      if (header.crc32 !== expectedCrc) {
        this.buffer = this.buffer.subarray(totalLen)
        continue
      }

      this.buffer = this.buffer.subarray(totalLen)

      if (this.onFrame != null) {
        this.onFrame(header, payload)
      }
    }
  }
}

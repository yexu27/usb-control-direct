import { describe, it, expect, vi } from 'vitest'
import {
  FRAME_MAGIC,
  FRAME_HEADER_SIZE,
  MAX_PAYLOAD_SIZE,
  encodeFrame,
  decodeFrameHeader,
  calculateCrc32,
  FrameStreamParser,
} from '../../../src/main/tls/frame-codec'

describe('calculateCrc32', () => {
  it('returns 0x00000000 for empty data', () => {
    expect(calculateCrc32(new Uint8Array(0))).toBe(0x00000000)
  })

  it('computes correct CRC32 for known input', () => {
    const data = new TextEncoder().encode('123456789')
    expect(calculateCrc32(data)).toBe(0xcbf43926)
  })
})

describe('encodeFrame', () => {
  it('produces 20-byte header + payload', () => {
    const payload = new Uint8Array([0x01, 0x02, 0x03])
    const frame = encodeFrame(0x0001, 1, payload)
    expect(frame.length).toBe(FRAME_HEADER_SIZE + payload.length)
  })

  it('writes magic number at offset 0 as big-endian', () => {
    const frame = encodeFrame(0x0001, 1, new Uint8Array(0))
    expect(frame.readUInt32BE(0)).toBe(FRAME_MAGIC)
  })

  it('writes message type at offset 4', () => {
    const frame = encodeFrame(0x0102, 1, new Uint8Array(0))
    expect(frame.readUInt32BE(4)).toBe(0x0102)
  })

  it('writes sequence id at offset 8', () => {
    const frame = encodeFrame(0x0001, 42, new Uint8Array(0))
    expect(frame.readUInt32BE(8)).toBe(42)
  })

  it('writes payload length at offset 12', () => {
    const payload = new Uint8Array(100)
    const frame = encodeFrame(0x0001, 1, payload)
    expect(frame.readUInt32BE(12)).toBe(100)
  })

  it('writes CRC32 at offset 16 as 0 for empty payload', () => {
    const frame = encodeFrame(0x0001, 1, new Uint8Array(0))
    expect(frame.readUInt32BE(16)).toBe(0x00000000)
  })

  it('writes correct CRC32 for non-empty payload', () => {
    const payload = new TextEncoder().encode('123456789')
    const frame = encodeFrame(0x0001, 1, payload)
    expect(frame.readUInt32BE(16)).toBe(0xcbf43926)
  })
})

describe('decodeFrameHeader', () => {
  it('returns null for buffer shorter than 20 bytes', () => {
    expect(decodeFrameHeader(Buffer.alloc(19))).toBeNull()
  })

  it('decodes header from a valid frame', () => {
    const payload = new Uint8Array([0xaa, 0xbb])
    const frame = encodeFrame(0x0200, 7, payload)
    const header = decodeFrameHeader(frame)
    expect(header).not.toBeNull()
    expect(header!.magic).toBe(FRAME_MAGIC)
    expect(header!.msgType).toBe(0x0200)
    expect(header!.seqId).toBe(7)
    expect(header!.payloadLen).toBe(2)
  })
})

describe('FrameStreamParser', () => {
  it('parses a complete frame delivered in one chunk', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const payload = new Uint8Array([0x01, 0x02])
    const frame = encodeFrame(0x0001, 1, payload)
    parser.feed(frame)

    expect(onFrame).toHaveBeenCalledTimes(1)
    const [header, parsedPayload] = onFrame.mock.calls[0]
    expect(header.msgType).toBe(0x0001)
    expect(header.seqId).toBe(1)
    expect(parsedPayload).toEqual(payload)
  })

  it('handles TCP fragmentation (split delivery)', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const payload = new Uint8Array([0x0a, 0x0b, 0x0c])
    const frame = encodeFrame(0x0002, 2, payload)

    parser.feed(frame.subarray(0, 10))
    expect(onFrame).not.toHaveBeenCalled()

    parser.feed(frame.subarray(10))
    expect(onFrame).toHaveBeenCalledTimes(1)
  })

  it('handles multiple frames in one chunk (TCP coalescing)', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const frame1 = encodeFrame(0x0001, 1, new Uint8Array([0x01]))
    const frame2 = encodeFrame(0x0002, 2, new Uint8Array([0x02]))
    const combined = Buffer.concat([frame1, frame2])

    parser.feed(combined)
    expect(onFrame).toHaveBeenCalledTimes(2)
  })

  it('discards frame with bad magic and continues', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const badFrame = Buffer.alloc(20)
    badFrame.writeUInt32BE(0xdeadbeef, 0)
    badFrame.writeUInt32BE(0, 12)

    const goodFrame = encodeFrame(0x0001, 1, new Uint8Array(0))
    parser.feed(Buffer.concat([badFrame, goodFrame]))

    expect(onFrame).toHaveBeenCalledTimes(1)
    expect(onFrame.mock.calls[0][0].msgType).toBe(0x0001)
  })

  it('discards frame with bad CRC32', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const payload = new Uint8Array([0x01, 0x02])
    const frame = encodeFrame(0x0001, 1, payload)
    frame.writeUInt32BE(0xffffffff, 16)

    parser.feed(frame)
    expect(onFrame).not.toHaveBeenCalled()
  })
})

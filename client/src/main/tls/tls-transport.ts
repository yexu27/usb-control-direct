import * as tls from 'tls'
import { EventEmitter } from 'events'
import { assertCertificateFingerprint } from './certificate-fingerprint'

const CONNECT_TIMEOUT = 15_000

export class TlsTransport extends EventEmitter {
  private socket: tls.TLSSocket | null = null

  async connect(host: string, port: number): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        if (this.socket != null) {
          this.socket.destroy()
          this.socket = null
        }
        reject(new Error('连接超时'))
      }, CONNECT_TIMEOUT)

      this.socket = tls.connect(
        {
          host,
          port,
          rejectUnauthorized: false,
        },
        () => {
          clearTimeout(timer)

          const cert = this.socket!.getPeerCertificate()
          try {
            assertCertificateFingerprint(
              cert.fingerprint256,
              __USB_CONTROL_CERT_FINGERPRINT__,
            )
          } catch (error: unknown) {
            this.socket!.destroy()
            this.socket = null
            reject(error)
            return
          }

          resolve()
        },
      )

      this.socket.on('data', (chunk: Buffer) => {
        this.emit('data', chunk)
      })

      this.socket.on('close', () => {
        this.socket = null
        this.emit('close')
      })

      this.socket.on('error', (err: Error) => {
        clearTimeout(timer)
        this.socket = null
        this.emit('error', err)
        reject(err)
      })
    })
  }

  disconnect(): void {
    if (this.socket != null) {
      this.socket.destroy()
      this.socket = null
    }
  }

  write(data: Buffer): void {
    if (this.socket != null && !this.socket.destroyed) {
      this.socket.write(data)
    }
  }

  isConnected(): boolean {
    return this.socket != null && !this.socket.destroyed
  }
}

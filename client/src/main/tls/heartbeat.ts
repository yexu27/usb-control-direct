const HEARTBEAT_INTERVAL = 30_000
const MAX_MISS_COUNT = 3

export class HeartbeatManager {
  private intervalId: ReturnType<typeof setInterval> | null = null
  private missCount = 0
  onTimeout: (() => void) | null = null

  start(sendFn: () => Promise<void>): void {
    this.stop()
    this.missCount = 0

    this.intervalId = setInterval(() => {
      this.missCount++

      if (this.missCount >= MAX_MISS_COUNT) {
        this.stop()
        if (this.onTimeout != null) {
          this.onTimeout()
        }
        return
      }

      sendFn().catch(() => {})
    }, HEARTBEAT_INTERVAL)
  }

  stop(): void {
    if (this.intervalId != null) {
      clearInterval(this.intervalId)
      this.intervalId = null
    }
    this.missCount = 0
  }

  onHeartbeatResponse(): void {
    this.missCount = 0
  }
}

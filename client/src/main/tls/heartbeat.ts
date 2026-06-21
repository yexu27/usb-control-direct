const HEARTBEAT_INTERVAL = 30_000
const MAX_MISS_COUNT = 3

export class HeartbeatManager {
  private intervalId: ReturnType<typeof setInterval> | null = null
  private missCount = 0
  private isAwaitingResponse = false
  onTimeout: (() => void) | null = null

  start(sendFn: () => Promise<void>): void {
    this.stop()
    this.missCount = 0
    this.isAwaitingResponse = false

    this.intervalId = setInterval(() => {
      if (this.isAwaitingResponse) {
        this.missCount++
      }

      if (this.missCount >= MAX_MISS_COUNT) {
        this.stop()
        if (this.onTimeout != null) {
          this.onTimeout()
        }
        return
      }

      this.isAwaitingResponse = true
      sendFn().catch(() => {})
    }, HEARTBEAT_INTERVAL)
  }

  stop(): void {
    if (this.intervalId != null) {
      clearInterval(this.intervalId)
      this.intervalId = null
    }
    this.missCount = 0
    this.isAwaitingResponse = false
  }

  onHeartbeatResponse(): void {
    this.missCount = 0
    this.isAwaitingResponse = false
  }
}

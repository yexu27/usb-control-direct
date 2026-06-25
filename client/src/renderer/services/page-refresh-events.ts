export type PageRefreshReason = 'reconnect'
export type PageRefreshListener = (reason: PageRefreshReason) => void

const pageRefreshListeners = new Set<PageRefreshListener>()

export function emitPageRefresh(reason: PageRefreshReason): void {
  for (const listener of Array.from(pageRefreshListeners)) {
    listener(reason)
  }
}

export function onPageRefresh(listener: PageRefreshListener): () => void {
  pageRefreshListeners.add(listener)
  return () => {
    pageRefreshListeners.delete(listener)
  }
}

export function resetPageRefreshListenersForTest(): void {
  if (import.meta.env.MODE !== 'test') {
    return
  }
  pageRefreshListeners.clear()
}

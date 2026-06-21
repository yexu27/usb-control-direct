import type { ServiceError } from './send-command'

type ServiceErrorListener = (error: ServiceError) => void

const serviceErrorListeners = new Set<ServiceErrorListener>()

export function emitServiceError(error: ServiceError): void {
  for (const listener of serviceErrorListeners) {
    listener(error)
  }
}

export function onServiceError(listener: ServiceErrorListener): () => void {
  serviceErrorListeners.add(listener)
  return () => {
    serviceErrorListeners.delete(listener)
  }
}

import type { ConnectionStatus, ConnectionEvent } from '../../shared/connection-state'

type TransitionMap = Partial<Record<ConnectionEvent, ConnectionStatus>>
type StateTransitions = Record<ConnectionStatus, TransitionMap>

const TRANSITIONS: StateTransitions = {
  DISCONNECTED: {
    CONNECT_START: 'CONNECTING',
  },
  CONNECTING: {
    CONNECT_SUCCESS: 'AUTHENTICATING',
    CONNECT_FAIL: 'DISCONNECTED',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  AUTHENTICATING: {
    AUTH_SUCCESS: 'CHECK_LICENSE',
    AUTH_FAIL: 'AUTHENTICATING',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  CHECK_LICENSE: {
    LICENSE_AUTHORIZED: 'LOADING_CONFIG',
    LICENSE_UNAUTHORIZED: 'AUTH_REQUIRED',
    LICENSE_EXPIRED: 'LICENSE_EXPIRED',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  AUTH_REQUIRED: {
    LICENSE_UPLOAD_SUCCESS: 'DISCONNECTED',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  LICENSE_EXPIRED: {
    NETWORK_ERROR: 'DISCONNECTED',
  },
  LOADING_CONFIG: {
    CONFIG_LOADED: 'CONNECTED',
    CONFIG_FAILED: 'DISCONNECTED',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  CONNECTED: {
    HEARTBEAT_TIMEOUT: 'DISCONNECTED',
    NETWORK_ERROR: 'DISCONNECTED',
    LOGOUT: 'DISCONNECTED',
  },
}

export class ConnectionStateMachine {
  private state: ConnectionStatus = 'DISCONNECTED'
  onStateChange:
    | ((from: ConnectionStatus, to: ConnectionStatus, event: ConnectionEvent) => void)
    | null = null

  get current(): ConnectionStatus {
    return this.state
  }

  transition(event: ConnectionEvent): ConnectionStatus {
    const transitions = TRANSITIONS[this.state]
    const nextState = transitions[event]

    if (nextState == null) {
      throw new Error(
        `Invalid transition: state=${this.state}, event=${event}`,
      )
    }

    const prevState = this.state
    this.state = nextState

    if (this.onStateChange != null) {
      this.onStateChange(prevState, nextState, event)
    }

    return nextState
  }
}

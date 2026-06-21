export type ConnectionStatus =
  | 'DISCONNECTED'
  | 'CONNECTING'
  | 'AUTHENTICATING'
  | 'CHECK_LICENSE'
  | 'AUTH_REQUIRED'
  | 'LICENSE_EXPIRED'
  | 'LOADING_CONFIG'
  | 'CONNECTED'

export type ConnectionEvent =
  | 'CONNECT_START'
  | 'CONNECT_SUCCESS'
  | 'CONNECT_FAIL'
  | 'AUTH_SUCCESS'
  | 'AUTH_FAIL'
  | 'LICENSE_AUTHORIZED'
  | 'LICENSE_UNAUTHORIZED'
  | 'LICENSE_EXPIRED'
  | 'CONFIG_LOADED'
  | 'CONFIG_FAILED'
  | 'HEARTBEAT_TIMEOUT'
  | 'NETWORK_ERROR'
  | 'LOGOUT'
  | 'LICENSE_UPLOAD_SUCCESS'

export type UserRole = 'admin' | 'operator' | 'auditor'

export type AuthStatus = 'authorized' | 'unauthorized' | 'expired' | 'failed' | ''

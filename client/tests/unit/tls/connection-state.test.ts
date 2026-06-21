import { describe, it, expect, vi } from 'vitest'
import { ConnectionStateMachine } from '../../../src/main/tls/connection-state'

describe('ConnectionStateMachine', () => {
  it('starts in DISCONNECTED state', () => {
    const sm = new ConnectionStateMachine()
    expect(sm.current).toBe('DISCONNECTED')
  })

  it('transitions DISCONNECTED → CONNECTING on CONNECT_START', () => {
    const sm = new ConnectionStateMachine()
    expect(sm.transition('CONNECT_START')).toBe('CONNECTING')
  })

  it('transitions CONNECTING → AUTHENTICATING on CONNECT_SUCCESS', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    expect(sm.transition('CONNECT_SUCCESS')).toBe('AUTHENTICATING')
  })

  it('transitions CONNECTING → DISCONNECTED on CONNECT_FAIL', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    expect(sm.transition('CONNECT_FAIL')).toBe('DISCONNECTED')
  })

  it('transitions CONNECTING → DISCONNECTED on LOGOUT', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    expect(sm.transition('LOGOUT')).toBe('DISCONNECTED')
  })

  it('transitions AUTHENTICATING → CHECK_LICENSE on AUTH_SUCCESS', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    expect(sm.transition('AUTH_SUCCESS')).toBe('CHECK_LICENSE')
  })

  it('stays AUTHENTICATING on AUTH_FAIL', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    expect(sm.transition('AUTH_FAIL')).toBe('AUTHENTICATING')
  })

  it('transitions CHECK_LICENSE → LOADING_CONFIG on LICENSE_AUTHORIZED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    expect(sm.transition('LICENSE_AUTHORIZED')).toBe('LOADING_CONFIG')
  })

  it('transitions CHECK_LICENSE → AUTH_REQUIRED on LICENSE_UNAUTHORIZED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    expect(sm.transition('LICENSE_UNAUTHORIZED')).toBe('AUTH_REQUIRED')
  })

  it('transitions CHECK_LICENSE → LICENSE_EXPIRED on LICENSE_EXPIRED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    expect(sm.transition('LICENSE_EXPIRED')).toBe('LICENSE_EXPIRED')
  })

  it('transitions LICENSE_EXPIRED → DISCONNECTED on LICENSE_UPLOAD_SUCCESS', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_EXPIRED')
    expect(sm.transition('LICENSE_UPLOAD_SUCCESS')).toBe('DISCONNECTED')
  })

  it('transitions AUTH_REQUIRED → DISCONNECTED on LICENSE_UPLOAD_SUCCESS', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_UNAUTHORIZED')
    expect(sm.transition('LICENSE_UPLOAD_SUCCESS')).toBe('DISCONNECTED')
  })

  it('transitions LOADING_CONFIG → CONNECTED on CONFIG_LOADED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    expect(sm.transition('CONFIG_LOADED')).toBe('CONNECTED')
  })

  it('transitions LOADING_CONFIG → DISCONNECTED on CONFIG_FAILED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    expect(sm.transition('CONFIG_FAILED')).toBe('DISCONNECTED')
  })

  it('transitions CONNECTED → DISCONNECTED on HEARTBEAT_TIMEOUT', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    sm.transition('CONFIG_LOADED')
    expect(sm.transition('HEARTBEAT_TIMEOUT')).toBe('DISCONNECTED')
  })

  it('transitions CONNECTED → DISCONNECTED on NETWORK_ERROR', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    sm.transition('CONFIG_LOADED')
    expect(sm.transition('NETWORK_ERROR')).toBe('DISCONNECTED')
  })

  it('transitions CONNECTED → DISCONNECTED on LOGOUT', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    sm.transition('CONFIG_LOADED')
    expect(sm.transition('LOGOUT')).toBe('DISCONNECTED')
  })

  it('NETWORK_ERROR transitions any state to DISCONNECTED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    expect(sm.transition('NETWORK_ERROR')).toBe('DISCONNECTED')
  })

  it('throws on invalid transition', () => {
    const sm = new ConnectionStateMachine()
    expect(() => sm.transition('AUTH_SUCCESS')).toThrow()
  })

  it('calls onStateChange callback on valid transition', () => {
    const sm = new ConnectionStateMachine()
    const callback = vi.fn()
    sm.onStateChange = callback

    sm.transition('CONNECT_START')
    expect(callback).toHaveBeenCalledWith('DISCONNECTED', 'CONNECTING', 'CONNECT_START')
  })
})

interface LogEvent {
  level?: string
  message?: string
}

export function logMessage(event: LogEvent): void {
  const level = event.level || 'info'
  const msg = event.message || JSON.stringify(event)
  if (level === 'error') {
    console.error(`[WASM] ${msg}`)
  } else if (level === 'warn') {
    console.warn(`[WASM] ${msg}`)
  } else {
    console.log(`[WASM] ${msg}`)
  }
}

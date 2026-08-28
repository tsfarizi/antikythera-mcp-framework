const STATE_PREFIX = 'antikythera_state_'

export async function saveState(contextId: string, stateJson: string): Promise<void> {
  localStorage.setItem(STATE_PREFIX + contextId, stateJson)
}

export async function loadState(contextId: string): Promise<string | null> {
  return localStorage.getItem(STATE_PREFIX + contextId)
}

export function clearAllStates(): void {
  const keys = Object.keys(localStorage).filter((k) => k.startsWith(STATE_PREFIX))
  keys.forEach((k) => localStorage.removeItem(k))
}

import { runner } from 'antikythera-agent/component'

// The jco-transpiled component exposes one namespace `runner` with camelCase
// functions. Keep the module's historical snake_case surface intact for its
// consumers (App.vue, slices/chat/flow/send-message.ts) via these aliases.
const initSession = runner.init
const prepare_user_turn = runner.prepareUserTurn
const commit_llm_response = runner.commitLlmResponse
const commit_llm_stream = runner.commitLlmStream
const drain_events = runner.drainEvents
const get_state = runner.getState
const register_tools = runner.registerTools
const get_tools_prompt = runner.getToolsPrompt
const reset_session = runner.resetSession
const append_llm_chunk = runner.appendLlmChunk
const process_llm_response_for_session = runner.processLlmResponseForSession
const process_tool_result_for_session = runner.processToolResultForSession
const set_context_policy = runner.setContextPolicy

let initialized = false

export async function initWasm(): Promise<void> {
  if (initialized) {
    console.log('[WASM] Already initialized, skipping')
    return
  }
  console.log('[WASM] Starting initialization from npm package...')
  try {
    // The static import above already evaluates the jco module, whose top-level
    // await instantiates the component exactly once (ESM module cache). This
    // dynamic import resolves the same cached module record — no double-instantiation.
    await import('antikythera-agent/component')
    initialized = true
    console.log('[WASM] Initialization successful (using npm package)')
  } catch (e) {
    console.error('[WASM] Initialization FAILED:', e)
    throw e
  }
}

// Wrap all exports with logging
export function initSessionLogged(configJson: string): string {
  console.log('[WASM] initSession called with config:', configJson)
  try {
    const result = initSession(configJson)
    console.log('[WASM] initSession result:', result)
    return result
  } catch (e) {
    console.error('[WASM] initSession FAILED:', e)
    throw e
  }
}

export function prepare_user_turnLogged(requestJson: string): string {
  console.log('[WASM] prepare_user_turn called with request:', requestJson)
  try {
    const result = prepare_user_turn(requestJson)
    console.log('[WASM] prepare_user_turn result:', result)
    return result
  } catch (e) {
    console.error('[WASM] prepare_user_turn FAILED:', e)
    throw e
  }
}

export function append_llm_chunkLogged(sessionId: string, chunk: string, correlationId?: string): boolean {
  console.log('[WASM] append_llm_chunk called:', { sessionId, chunkLength: chunk.length, correlationId })
  try {
    const result = append_llm_chunk(sessionId, chunk, correlationId)
    console.log('[WASM] append_llm_chunk result:', result)
    return result
  } catch (e) {
    console.error('[WASM] append_llm_chunk FAILED:', e)
    throw e
  }
}

export function commit_llm_streamLogged(preparedTurnJson: string): string {
  console.log('[WASM] commit_llm_stream called with preparedTurn:', preparedTurnJson.substring(0, 200) + '...')
  try {
    const result = commit_llm_stream(preparedTurnJson)
    console.log('[WASM] commit_llm_stream result:', result)
    return result
  } catch (e) {
    console.error('[WASM] commit_llm_stream FAILED:', e)
    throw e
  }
}

export function drain_eventsLogged(sessionId: string): string {
  console.log('[WASM] drain_events called for session:', sessionId)
  try {
    const result = drain_events(sessionId)
    console.log('[WASM] drain_events result:', result)
    return result
  } catch (e) {
    console.error('[WASM] drain_events FAILED:', e)
    throw e
  }
}

export function reset_sessionLogged(sessionId: string): boolean {
  console.log('[WASM] reset_session called for session:', sessionId)
  try {
    const result = reset_session(sessionId)
    console.log('[WASM] reset_session result:', result)
    return result
  } catch (e) {
    console.error('[WASM] reset_session FAILED:', e)
    throw e
  }
}

// Re-export raw functions for direct access if needed
export {
  initSession,
  prepare_user_turn,
  commit_llm_response,
  commit_llm_stream,
  drain_events,
  get_state,
  register_tools,
  get_tools_prompt,
  reset_session,
  append_llm_chunk,
  process_llm_response_for_session,
  process_tool_result_for_session,
  set_context_policy,
}

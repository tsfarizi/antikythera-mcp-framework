import { ref } from 'vue'
import {
  initWasm,
  initSessionLogged as initSession,
  prepare_user_turnLogged as prepare_user_turn,
  commit_llm_streamLogged as commit_llm_stream,
  drain_eventsLogged as drain_events,
  append_llm_chunkLogged as append_llm_chunk,
  reset_sessionLogged as reset_session,
} from '@/shared/wasm'
import { streamOllama, getOllamaModel } from '@/shared/adapters/llm-adapter'
import { eventBus } from '@/shared/bus/event-bus'
import { createMessage, type Message } from '../core/message'
import type { ChatStatus } from '../core/state'

const messages = ref<Message[]>([])
const status = ref<ChatStatus>('idle')
const sessionId = ref<string | null>(null)
const streamingContent = ref('')
const error = ref<string | null>(null)

export function useChat() {
  async function initSession_() {
    console.log('[Chat] initSession_ called')
    await initWasm()
    console.log('[Chat] WASM initialized, creating session...')
    const result = initSession(JSON.stringify({
      max_steps: 10,
      verbose: false,
      auto_execute_tools: false,
      session_timeout_secs: 300,
    }))
    sessionId.value = result
    console.log('[Chat] Session created:', result)
    eventBus.emit('session:created', {
      sessionId: result,
      title: 'New Chat',
    })
  }

  async function sendMessage(content: string) {
    console.log('[Chat] sendMessage called with:', content)

    if (!sessionId.value) {
      console.log('[Chat] No active session, initializing...')
      await initSession_()
    }

    console.log('[Chat] Using session:', sessionId.value)

    const userMsg = createMessage('user', content)
    messages.value.push(userMsg)
    status.value = 'streaming'
    streamingContent.value = ''
    error.value = null

    eventBus.emit('chat:message-sent', {
      sessionId: sessionId.value!,
      content,
      timestamp: Date.now(),
    })

    let preparedJson: string | null = null

    try {
      // 1. Prepare turn — FSM: Idle → UserTurnPrepared → LlmStreaming
      console.log('[Chat] Step 1: prepare_user_turn')
      preparedJson = prepare_user_turn(JSON.stringify({
        session_id: sessionId.value,
        prompt: content,
      }))
      console.log('[Chat] prepare_user_turn succeeded')

      const prepared = JSON.parse(preparedJson)
      console.log('[Chat] Prepared turn:', {
        sessionId: prepared.session_id,
        step: prepared.step,
        messagesCount: prepared.messages_json ? JSON.parse(prepared.messages_json).length : 0,
      })

      const messagesForLlm = prepared.messages_json
        ? JSON.parse(prepared.messages_json)
        : []

      // 2. Real streaming from Ollama (F4 fix)
      const model = getOllamaModel()
      console.log('[Chat] Step 2: streamOllama with model:', model, 'messages:', messagesForLlm.length)
      let fullResponse = ''
      let tokenCount = 0

      console.log('[Chat] Starting streaming...')
      for await (const token of streamOllama(model, messagesForLlm)) {
        tokenCount++
        // 3. Append each token to WASM session
        append_llm_chunk(sessionId.value!, token, undefined)
        fullResponse += token
        streamingContent.value = fullResponse

        if (tokenCount % 10 === 0) {
          console.log(`[Chat] Streaming token ${tokenCount}, length: ${fullResponse.length}`)
        }

        eventBus.emit('chat:token-received', {
          sessionId: sessionId.value!,
          token,
        })
      }
      console.log('[Chat] Streaming complete, total tokens:', tokenCount, 'response length:', fullResponse.length)

      // 4. Commit stream — joins chunks, processes response
      console.log('[Chat] Step 4: commit_llm_stream')
      commit_llm_stream(preparedJson)
      console.log('[Chat] commit_llm_stream succeeded')

      // 5. Drain WASM events
      console.log('[Chat] Step 5: drain_events')
      const eventsJson = drain_events(sessionId.value!)
      const events = JSON.parse(eventsJson)
      console.log('[Chat] drain_events returned', events.length, 'events')

      // 6. Finalize
      console.log('[Chat] Step 6: Finalizing')
      const assistantMsg = createMessage('assistant', fullResponse)
      messages.value.push(assistantMsg)
      streamingContent.value = ''
      status.value = 'idle'
      console.log('[Chat] Message sent successfully')

      eventBus.emit('chat:response-completed', {
        sessionId: sessionId.value!,
        content: fullResponse,
      })
    } catch (e: unknown) {
      console.error('[Chat] ERROR in sendMessage:', e)
      console.error('[Chat] Error stack:', e instanceof Error ? e.stack : 'No stack trace')

      // F3 fix: Reset WASM session on error to recover FSM state
      const failedSessionId = sessionId.value || 'unknown'
      if (sessionId.value) {
        console.log('[Chat] Resetting WASM session for recovery...')
        try {
          reset_session(sessionId.value)
          console.log('[Chat] WASM session reset succeeded')
        } catch (resetErr) {
          console.error('[Chat] WASM session reset FAILED:', resetErr)
        }
        // Re-create session for next message
        sessionId.value = null
        console.log('[Chat] Session ID cleared for re-creation')
      }

      status.value = 'error'
      const errMsg = e instanceof Error ? e.message : 'Unknown error'
      error.value = errMsg
      console.error('[Chat] Emitting error event:', { failedSessionId, errMsg })
      eventBus.emit('chat:error-occurred', {
        sessionId: failedSessionId,
        error: errMsg,
      })
    }
  }

  function clearMessages() {
    messages.value = []
    streamingContent.value = ''
    error.value = null
  }

  async function switchSession(newSessionId: string) {
    messages.value = []
    streamingContent.value = ''
    sessionId.value = newSessionId
    status.value = 'idle'
    eventBus.emit('session:switched', { sessionId: newSessionId })
  }

  return {
    messages,
    status,
    sessionId,
    streamingContent,
    error,
    sendMessage,
    clearMessages,
    switchSession,
    initSession: initSession_,
  }
}

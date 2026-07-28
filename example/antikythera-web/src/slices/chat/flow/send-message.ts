import { ref } from 'vue'
import {
  initWasm,
  initSessionLogged as initSession,
  prepare_user_turnLogged as prepare_user_turn,
  commit_llm_streamLogged as commit_llm_stream,
  drain_eventsLogged as drain_events,
  append_llm_chunkLogged as append_llm_chunk,
  reset_sessionLogged as reset_session,
  register_tools,
  get_tools_prompt,
  process_tool_result_for_session,
} from '@/shared/wasm'
import { streamOllama, getOllamaModel } from '@/shared/adapters/llm-adapter'
import { executeBrowserTool, toolsDefinitionJson } from '@/shared/adapters/tool-adapter'
import { eventBus } from '@/shared/bus/event-bus'
import { createMessage, type Message } from '../core/message'
import type { ChatStatus } from '../core/state'

const messages = ref<Message[]>([])
const status = ref<ChatStatus>('idle')
const sessionId = ref<string | null>(null)
const streamingContent = ref('')
const error = ref<string | null>(null)

/** Register browser-side MCP tools with the WASM agent. */
function registerBrowserTools() {
  const json = toolsDefinitionJson()
  const count = register_tools(json)
  console.log(`[Chat] Registered ${count} browser MCP tool(s)`)

  const prompt = get_tools_prompt()
  if (prompt) {
    console.log('[Chat] Tool prompt:', prompt)
  }
}

/** Process tool-call events from drain_events, execute in browser, feed results back.
 *  Returns true if any tool was executed. */
async function processToolEvents(events: Array<{ kind: string; payload?: string }>): Promise<boolean> {
  let hadToolCalls = false
  for (const event of events) {
    if (event.kind !== 'tool_requested') continue
    hadToolCalls = true

    const payload = event.payload ? JSON.parse(event.payload) : {}
    const toolName = payload.tool
    const toolInput = payload.input || {}
    const stepId = payload.step_id || 0

    console.log(`[Chat] Tool requested: ${toolName}`, toolInput)

    const result = executeBrowserTool(toolName, toolInput, stepId)
    console.log(`[Chat] Tool result:`, result)

    const resultJson = JSON.stringify(result)
    process_tool_result_for_session(sessionId.value!, resultJson)
    console.log(`[Chat] Tool result fed back to WASM agent`)
  }
  return hadToolCalls
}

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

    // Register browser MCP tools
    registerBrowserTools()

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
      const messagesForLlm: Array<{ role: string; content: string }> = prepared.messages_json
        ? JSON.parse(prepared.messages_json)
        : []

      // Enhance system prompt with tool-calling format instructions
      const toolPrompt = get_tools_prompt()
      if (toolPrompt) {
        const toolInstructions = `${toolPrompt}

## Tool Calling Format
When you need to call a tool, respond with EXACTLY this JSON format and nothing else:
{"action":"call_tool","tool":"tool_name","input":{param1:"value1"}}

When you have a final answer (no tool needed), respond normally in plain text.
Do NOT mix JSON tool calls with regular text.`

        let foundSystem = false
        for (const msg of messagesForLlm) {
          if (msg.role === 'system') {
            msg.content = msg.content + toolInstructions
            foundSystem = true
            break
          }
        }
        if (!foundSystem) {
          messagesForLlm.unshift({ role: 'system', content: toolInstructions })
        }
        console.log('[Chat] Enhanced system prompt with tool-calling instructions')
      }

      // 2. Stream from Ollama
      const model = getOllamaModel()
      console.log('[Chat] Step 2: streamOllama with model:', model)
      let fullResponse = ''
      let tokenCount = 0

      for await (const token of streamOllama(model, messagesForLlm)) {
        tokenCount++
        append_llm_chunk(sessionId.value!, token, undefined)
        fullResponse += token
        streamingContent.value = fullResponse
        eventBus.emit('chat:token-received', {
          sessionId: sessionId.value!,
          token,
        })
      }
      console.log('[Chat] Streaming complete, tokens:', tokenCount)

      // 3. Commit stream
      console.log('[Chat] Step 3: commit_llm_stream')
      commit_llm_stream(preparedJson)

      // 4. Drain events and process tool calls
      console.log('[Chat] Step 4: drain_events')
      const eventsJson = drain_events(sessionId.value!)
      const events = JSON.parse(eventsJson)
      console.log('[Chat] drain_events returned', events.length, 'events')

      // Process any tool_requested events
      const hadToolCalls = await processToolEvents(events)

      // 5. After tool execution, drain again for tool_result events
      const eventsJson2 = drain_events(sessionId.value!)
      const events2 = JSON.parse(eventsJson2)
      console.log('[Chat] drain_events (post-tool) returned', events2.length, 'events')

      // If LLM returned empty content but tools were executed,
      // do another LLM turn to get the final response with tool results
      if (fullResponse.trim() === '' && hadToolCalls) {
        console.log('[Chat] Empty response after tool execution, doing another LLM turn...')
        const secondPreparedJson = prepare_user_turn(JSON.stringify({
          session_id: sessionId.value,
          prompt: 'Based on the tool results above, provide your answer to the user.',
        }))
        const secondPrepared = JSON.parse(secondPreparedJson)
        const secondMessages: Array<{ role: string; content: string }> = secondPrepared.messages_json
          ? JSON.parse(secondPrepared.messages_json)
          : []

        fullResponse = ''
        tokenCount = 0
        for await (const token of streamOllama(model, secondMessages)) {
          tokenCount++
          append_llm_chunk(sessionId.value!, token, undefined)
          fullResponse += token
          streamingContent.value = fullResponse
          eventBus.emit('chat:token-received', {
            sessionId: sessionId.value!,
            token,
          })
        }
        console.log('[Chat] Second LLM turn complete, tokens:', tokenCount)
        commit_llm_stream(secondPreparedJson)
      }

      // 6. Finalize
      console.log('[Chat] Step 6: Finalizing')
      const assistantMsg = createMessage('assistant', fullResponse)
      messages.value.push(assistantMsg)
      streamingContent.value = ''
      status.value = 'idle'

      eventBus.emit('chat:response-completed', {
        sessionId: sessionId.value!,
        content: fullResponse,
      })
    } catch (e: unknown) {
      console.error('[Chat] ERROR in sendMessage:', e)

      const failedSessionId = sessionId.value || 'unknown'
      if (sessionId.value) {
        try {
          reset_session(sessionId.value)
        } catch (resetErr) {
          console.error('[Chat] WASM session reset FAILED:', resetErr)
        }
        sessionId.value = null
      }

      status.value = 'error'
      const errMsg = e instanceof Error ? e.message : 'Unknown error'
      error.value = errMsg
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

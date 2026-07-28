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
import { createMessage, type Message, type ToolCallRecord } from '../core/message'
import type { ChatStatus } from '../core/state'

const MAX_TOOL_ROUNDS = 5

const messages = ref<Message[]>([])
const status = ref<ChatStatus>('idle')
const sessionId = ref<string | null>(null)
const streamingContent = ref('')
const error = ref<string | null>(null)

function registerBrowserTools() {
  const json = toolsDefinitionJson()
  const count = register_tools(json)
  console.log(`[Chat] Registered ${count} browser MCP tool(s)`)
}

/**
 * Enhance system prompt with tool-calling format instructions.
 */
function enhanceSystemPrompt(msgs: Array<{ role: string; content: string }>) {
  const toolPrompt = get_tools_prompt()
  if (!toolPrompt) return

  const toolInstructions = `${toolPrompt}

## Tool Calling Format
When you need to call a tool, respond with EXACTLY this JSON and nothing else:
{"action":"call_tool","tool":"tool_name","input":{}}

When you have your final answer, respond normally in plain text.
Do NOT mix JSON tool calls with regular text.`

  let found = false
  for (const msg of msgs) {
    if (msg.role === 'system') {
      msg.content += toolInstructions
      found = true
      break
    }
  }
  if (!found) {
    msgs.unshift({ role: 'system', content: toolInstructions })
  }
}

/**
 * Extract tool call records from drain_events.
 */
function extractToolCalls(events: Array<{ kind: string; payload?: unknown }>): Array<{ name: string; input: Record<string, unknown>; stepId: number }> {
  const calls: Array<{ name: string; input: Record<string, unknown>; stepId: number }> = []
  for (const event of events) {
    if (event.kind !== 'tool_requested') continue
    const payload = typeof event.payload === 'string'
      ? JSON.parse(event.payload)
      : (event.payload || {})
    calls.push({
      name: payload.tool,
      input: payload.input || {},
      stepId: payload.step_id || 0,
    })
  }
  return calls
}

/**
 * Execute tool calls and feed results back to WASM.
 * Returns ToolCallRecord array for UI display.
 */
function executeToolCalls(
  calls: Array<{ name: string; input: Record<string, unknown>; stepId: number }>,
): ToolCallRecord[] {
  const records: ToolCallRecord[] = []
  for (const call of calls) {
    console.log(`[Chat] Executing tool: ${call.name}`, call.input)
    const result = executeBrowserTool(call.name, call.input, call.stepId)
    console.log(`[Chat] Tool result:`, result)

    const output = result.success
      ? JSON.parse(result.output_json)
      : null

    records.push({
      tool: call.name,
      input: call.input,
      output,
      success: result.success,
      error: result.error_message || undefined,
    })

    process_tool_result_for_session(sessionId.value!, JSON.stringify(result))
  }
  return records
}

/**
 * Run a single LLM turn: stream tokens → commit → drain events.
 * Returns { response, toolCalls, events }
 */
async function runLlmTurn(
  prompt: string,
  model: string,
): Promise<{ response: string; toolCalls: Array<{ name: string; input: Record<string, unknown>; stepId: number }>; rawEvents: unknown[] }> {
  // Prepare turn
  const preparedJson = prepare_user_turn(JSON.stringify({
    session_id: sessionId.value,
    prompt,
  }))
  const prepared = JSON.parse(preparedJson)
  const msgs: Array<{ role: string; content: string }> = prepared.messages_json
    ? JSON.parse(prepared.messages_json)
    : []

  enhanceSystemPrompt(msgs)

  // Stream
  let response = ''
  for await (const token of streamOllama(model, msgs)) {
    append_llm_chunk(sessionId.value!, token, undefined)
    response += token
    streamingContent.value = response
  }

  // Commit
  commit_llm_stream(preparedJson)

  // Drain
  const rawEvents = JSON.parse(drain_events(sessionId.value!))
  const toolCalls = extractToolCalls(rawEvents)

  return { response, toolCalls, rawEvents }
}

export function useChat() {
  async function initSession_() {
    await initWasm()
    const result = initSession(JSON.stringify({
      max_steps: 10,
      verbose: false,
      auto_execute_tools: false,
      session_timeout_secs: 300,
    }))
    sessionId.value = result
    registerBrowserTools()
    eventBus.emit('session:created', { sessionId: result, title: 'New Chat' })
  }

  async function sendMessage(content: string) {
    if (!sessionId.value) await initSession_()

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

    try {
      const model = getOllamaModel()
      let round = 0
      const allToolCalls: ToolCallRecord[] = []
      let finalResponse = ''

      // Agentic loop: keep calling LLM until no more tool requests
      while (round < MAX_TOOL_ROUNDS) {
        round++
        console.log(`[Chat] LLM round ${round}`)

        const prompt = round === 1
          ? content
          : 'Based on the tool results above, provide your answer to the user.'

        const { response, toolCalls } = await runLlmTurn(prompt, model)
        finalResponse = response

        if (toolCalls.length === 0) {
          // No tool calls — this is the final answer
          break
        }

        // Tool calls detected — execute and loop
        const records = executeToolCalls(toolCalls)
        allToolCalls.push(...records)
        streamingContent.value = ''

        // Drain tool_result events
        drain_events(sessionId.value!)
      }

      // Finalize — show message with tool call history
      const assistantMsg = createMessage('assistant', finalResponse, allToolCalls)
      messages.value.push(assistantMsg)
      streamingContent.value = ''
      status.value = 'idle'

      eventBus.emit('chat:response-completed', {
        sessionId: sessionId.value!,
        content: finalResponse,
      })
    } catch (e: unknown) {
      console.error('[Chat] ERROR:', e)
      if (sessionId.value) {
        try { reset_session(sessionId.value) } catch {}
        sessionId.value = null
      }
      status.value = 'error'
      error.value = e instanceof Error ? e.message : 'Unknown error'
      eventBus.emit('chat:error-occurred', {
        sessionId: sessionId.value || 'unknown',
        error: error.value,
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

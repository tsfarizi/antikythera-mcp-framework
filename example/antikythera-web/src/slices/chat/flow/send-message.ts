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
 * Enhance system prompt with explicit tool-calling instructions.
 */
function enhanceSystemPrompt(msgs: Array<{ role: string; content: string }>) {
  const toolPrompt = get_tools_prompt()
  if (!toolPrompt) return

  const instructions = `

${toolPrompt}

## CRITICAL: Tool Calling Rules

You have tools that provide REAL data. You MUST call them — never make up data.

### Triggers (call the tool when user asks):
- Time, date, timezone, clock → call \`get_current_time\`

### Output Format:
When calling a tool, output ONLY this JSON on its own line. Nothing else:
{"action":"call_tool","tool":"get_current_time","input":{}}

After receiving the tool result, give your final answer as plain text.
Never use {{template}} syntax. Never output placeholder text.`

  let found = false
  for (const msg of msgs) {
    if (msg.role === 'system') {
      msg.content += instructions
      found = true
      break
    }
  }
  if (!found) {
    msgs.unshift({ role: 'system', content: instructions })
  }
}

/**
 * Detect tool calls from raw response text.
 * Handles: pure JSON, mixed JSON+text, and template placeholders.
 */
function detectToolCallsFromText(text: string): Array<{ name: string; input: Record<string, unknown>; stepId: number }> {
  const calls: Array<{ name: string; input: Record<string, unknown>; stepId: number }> = []

  // 1. Try to find JSON tool call (may be concatenated with text)
  const jsonMatch = text.match(/\{"action"\s*:\s*"call_tool"[\s\S]*?"tool"\s*:\s*"([^"]+)"[\s\S]*?"input"\s*:\s*(\{[^}]*\})/)

  if (jsonMatch) {
    try {
      const toolName = jsonMatch[1]
      const input = JSON.parse(jsonMatch[2])
      calls.push({ name: toolName, input, stepId: 0 })
      console.log('[Chat] Detected JSON tool call:', toolName)
      return calls
    } catch { /* parse error, continue */ }
  }

  // 2. Try template placeholders: {{time}}, {{date}}, etc.
  const templateRegex = /\{\{(\w+)\}\}/g
  let match
  while ((match = templateRegex.exec(text)) !== null) {
    const placeholder = match[1].toLowerCase()
    if (['time', 'date', 'datetime', 'clock'].includes(placeholder)) {
      if (!calls.some(c => c.name === 'get_current_time')) {
        calls.push({ name: 'get_current_time', input: {}, stepId: 0 })
        console.log('[Chat] Detected template placeholder:', placeholder)
      }
    }
  }

  return calls
}

/**
 * Strip all tool call artifacts from display text.
 * Removes JSON objects and template placeholders.
 */
function cleanResponseForDisplay(text: string): string {
  let result = text
  // Remove JSON tool call objects (any format)
  result = result.replace(/\{"action"\s*:\s*"call_tool"[\s\S]*?\}/g, '')
  // Remove template placeholders
  result = result.replace(/\{\{\w+\}\}/g, '')
  // Remove orphaned braces
  result = result.replace(/^\s*\}\s*/g, '')
  result = result.replace(/\s*\{\s*$/g, '')
  return result.trim()
}

/**
 * Execute tool calls in the browser.
 * Returns ToolCallRecord array for UI display and next-turn context.
 * Does NOT call process_tool_result_for_session — the host loop handles context.
 */
function executeToolCalls(
  calls: Array<{ name: string; input: Record<string, unknown>; stepId: number }>,
): ToolCallRecord[] {
  const records: ToolCallRecord[] = []
  for (const call of calls) {
    console.log(`[Chat] Executing tool: ${call.name}`, call.input)
    try {
      const result = executeBrowserTool(call.name, call.input, call.stepId)
      console.log(`[Chat] Tool result:`, result)

      const output = result.success ? JSON.parse(result.output_json) : null

      records.push({
        tool: call.name,
        input: call.input,
        output,
        success: result.success,
        error: result.error_message || undefined,
      })
    } catch (e) {
      console.error(`[Chat] Tool execution failed:`, e)
      records.push({
        tool: call.name,
        input: call.input,
        output: null,
        success: false,
        error: e instanceof Error ? e.message : String(e),
      })
    }
  }
  return records
}

/**
 * Run a single LLM turn: prepare → stream → commit → drain.
 */
async function runLlmTurn(
  prompt: string,
  model: string,
): Promise<{ response: string; rawEvents: unknown[] }> {
  const preparedJson = prepare_user_turn(JSON.stringify({
    session_id: sessionId.value,
    prompt,
  }))
  const prepared = JSON.parse(preparedJson)
  const msgs: Array<{ role: string; content: string }> = prepared.messages_json
    ? JSON.parse(prepared.messages_json)
    : []

  enhanceSystemPrompt(msgs)

  // Stream tokens (don't display during tool rounds)
  let response = ''
  for await (const token of streamOllama(model, msgs)) {
    append_llm_chunk(sessionId.value!, token, undefined)
    response += token
  }

  commit_llm_stream(preparedJson)
  const rawEvents = JSON.parse(drain_events(sessionId.value!))

  return { response, rawEvents }
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
      const allToolCalls: ToolCallRecord[] = []
      let finalResponse = ''
      let isToolRound = false
      let toolResultContext = ''

      for (let round = 0; round < MAX_TOOL_ROUNDS; round++) {
        console.log(`[Chat] LLM round ${round + 1}`)

        const prompt = round === 0
          ? content
          : `The user asked: "${content}"\n\nHere are the tool results:\n${toolResultContext}\n\nBased on these results, provide a clear answer to the user.`

        const { response, rawEvents } = await runLlmTurn(prompt, model)

        // Detect tool calls: WASM events → raw text → templates
        let toolCalls = extractFromEvents(rawEvents)
        if (toolCalls.length === 0) {
          toolCalls = detectToolCallsFromText(response)
        }

        if (toolCalls.length === 0) {
          // Final answer — no tool calls
          finalResponse = cleanResponseForDisplay(response)
          isToolRound = false
          break
        }

        // Tool call round — execute and loop
        isToolRound = true
        streamingContent.value = '' // Hide raw streaming during tool execution

        const records = executeToolCalls(toolCalls)
        allToolCalls.push(...records)

        // Show progress
        streamingContent.value = records.map(r =>
          r.success
            ? `✓ ${r.tool}`
            : `✗ ${r.tool}: ${r.error}`
        ).join(' | ')

        // Build tool result context for next LLM turn
        toolResultContext = records
          .filter(r => r.success)
          .map(r => `Tool "${r.tool}" result: ${JSON.stringify(r.output)}`)
          .join('\n')

        // Brief pause so user sees the tool execution
        await new Promise(resolve => setTimeout(resolve, 300))
      }

      // Finalize
      streamingContent.value = ''

      // If we exhausted rounds without a final answer, show last known response
      if (!finalResponse && !isToolRound) {
        finalResponse = '(No response from model)'
      }

      const assistantMsg = createMessage('assistant', finalResponse, allToolCalls)
      messages.value.push(assistantMsg)
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

/**
 * Extract tool calls from WASM drain_events.
 */
function extractFromEvents(events: unknown[]): Array<{ name: string; input: Record<string, unknown>; stepId: number }> {
  const calls: Array<{ name: string; input: Record<string, unknown>; stepId: number }> = []
  for (const event of events) {
    const e = event as { kind?: string; payload?: unknown }
    if (e.kind !== 'tool_requested') continue
    const payload = typeof e.payload === 'string' ? JSON.parse(e.payload) : (e.payload || {})
    calls.push({
      name: payload.tool,
      input: payload.input || {},
      stepId: payload.step_id || 0,
    })
  }
  return calls
}

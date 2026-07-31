/**
 * Browser-side MCP tool definitions and execution.
 *
 * Each tool has:
 * - A definition (JSON Schema) registered with the WASM agent
 * - A handler function that executes in the browser
 *
 * Tools are pure functions — no network, no I/O, just computation.
 */

// ============================================================================
// Tool Definitions (MCP-compatible JSON Schema)
// ============================================================================

export interface ToolDefinition {
  name: string
  title?: string
  description: string
  parameters: ToolParameterSchema[]
  input_schema?: Record<string, unknown>
}

interface ToolParameterSchema {
  name: string
  param_type: string
  description: string
  required: boolean
}

const GET_CURRENT_TIME: ToolDefinition = {
  name: 'get_current_time',
  title: 'Current Time',
  description: 'Get the current date and time. Use this when the user asks about time, date, or timezone.',
  parameters: [],
  input_schema: {
    type: 'object',
    properties: {},
    required: [],
  },
}

/** All registered browser-side tools. */
export const BROWSER_TOOLS: ToolDefinition[] = [
  GET_CURRENT_TIME,
]

// ============================================================================
// Tool Execution
// ============================================================================

export interface ToolResult {
  tool_name: string
  success: boolean
  output_json: string
  error_message: string
  step_id: number
}

/**
 * Execute a browser-side tool by name.
 *
 * Returns a ToolResult compatible with the WASM agent's
 * `process_tool_result_for_session` input format.
 */
export function executeBrowserTool(
  toolName: string,
  _args: Record<string, unknown>,
  stepId: number,
): ToolResult {
  try {
    const output = dispatchTool(toolName)
    return {
      tool_name: toolName,
      success: true,
      output_json: JSON.stringify(output),
      error_message: '',
      step_id: stepId,
    }
  } catch (e) {
    return {
      tool_name: toolName,
      success: false,
      output_json: '{}',
      error_message: e instanceof Error ? e.message : String(e),
      step_id: stepId,
    }
  }
}

function dispatchTool(name: string): Record<string, unknown> {
  switch (name) {
    case 'get_current_time':
      return handleGetCurrentTime()
    default:
      throw new Error(`Unknown browser tool: ${name}`)
  }
}

// ============================================================================
// Tool Handlers
// ============================================================================

function handleGetCurrentTime(): Record<string, unknown> {
  const now = new Date()
  return {
    datetime: now.toISOString(),
    date: now.toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    }),
    time: now.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false,
    }),
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    unix_timestamp: Math.floor(now.getTime() / 1000),
  }
}

// ============================================================================
// Tool Registration (JSON for WASM)
// ============================================================================

/**
 * Serialize browser tools to JSON for `register_tools()`.
 */
export function toolsDefinitionJson(): string {
  return JSON.stringify(BROWSER_TOOLS)
}

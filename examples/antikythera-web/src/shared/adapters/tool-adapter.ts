/**
 * Client-owned tool declarations.
 *
 * Each entry is `{ definition, handler }` and is registered with the host
 * runtime through the `createAgentRuntime` `tools` option. There is no
 * dispatch switch: the runtime routes a `call_tool` action to the matching
 * handler by name. Handlers are pure local computation — no network, no I/O.
 */

import type { ToolDefinition, ToolEntry } from 'antikythera-agent/runtime'

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

/** All client-owned tools: definition + handler, one entry per tool. */
export const CLIENT_TOOL_ENTRIES: ToolEntry[] = [
  { definition: GET_CURRENT_TIME, handler: handleGetCurrentTime },
]

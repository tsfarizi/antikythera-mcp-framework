interface ToolCallEvent {
  tool_name?: string
  'tool-name'?: string
  step_id?: number
}

interface ToolResult {
  tool_name: string
  success: boolean
  output_json: string
  error_message: string
  step_id: number
}

export async function emitToolCall(event: ToolCallEvent): Promise<ToolResult> {
  console.warn('Tool execution not implemented in MVP:', event)
  return {
    tool_name: event.tool_name || event['tool-name'] || '',
    success: false,
    output_json: '{}',
    error_message: 'Tool execution not available in browser MVP',
    step_id: event.step_id || 0,
  }
}

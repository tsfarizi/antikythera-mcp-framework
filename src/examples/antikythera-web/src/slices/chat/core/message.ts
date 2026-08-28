export type MessageRole = 'user' | 'assistant' | 'system'

export interface ToolCallRecord {
  tool: string
  input: Record<string, unknown>
  output: Record<string, unknown> | null
  success: boolean
  error?: string
}

export interface Message {
  id: string
  role: MessageRole
  content: string
  timestamp: number
  streaming?: boolean
  /** Tool calls executed during this message's turn (collapsible in UI). */
  toolCalls?: ToolCallRecord[]
}

export function createMessage(
  role: Message['role'],
  content: string,
  toolCalls?: ToolCallRecord[],
): Message {
  return {
    id: crypto.randomUUID(),
    role,
    content,
    timestamp: Date.now(),
    ...(toolCalls && toolCalls.length > 0 ? { toolCalls } : {}),
  }
}

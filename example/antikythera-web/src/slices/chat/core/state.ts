import type { Message } from './message'

export type ChatStatus = 'idle' | 'streaming' | 'error'

export interface ChatState {
  sessionId: string | null
  status: ChatStatus
  messages: Message[]
  error: string | null
  streamingContent: string
}

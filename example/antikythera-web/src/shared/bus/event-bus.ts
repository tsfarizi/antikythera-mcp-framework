interface EventMap {
  'chat:message-sent': { sessionId: string; content: string; timestamp: number }
  'chat:token-received': { sessionId: string; token: string }
  'chat:response-completed': { sessionId: string; content: string }
  'chat:error-occurred': { sessionId: string; error: string }
  'chat:streaming-started': { sessionId: string }
  'session:created': { sessionId: string; title: string }
  'session:switched': { sessionId: string }
}

type EventHandler<T = unknown> = (payload: T) => void

class EventBus {
  private handlers = new Map<string, Set<EventHandler>>()

  on<K extends keyof EventMap>(event: K, handler: EventHandler<EventMap[K]>): () => void {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, new Set())
    }
    this.handlers.get(event)!.add(handler as EventHandler)
    return () => {
      this.handlers.get(event)?.delete(handler as EventHandler)
    }
  }

  emit<K extends keyof EventMap>(event: K, payload: EventMap[K]): void {
    this.handlers.get(event)?.forEach((handler) => handler(payload))
  }
}

export const eventBus = new EventBus()

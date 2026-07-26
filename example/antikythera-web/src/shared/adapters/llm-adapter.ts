export interface LlmRequest {
  model?: string
  messages: Array<{ role: string; content: string }>
  stream?: boolean
}

export interface LlmChunk {
  model: string
  message: { role: string; content: string }
  done: boolean
  total_duration?: number
  eval_count?: number
}

const OLLAMA_BASE = 'http://localhost:11434'

export async function* streamOllama(
  model: string,
  messages: Array<{ role: string; content: string }>,
): AsyncGenerator<string> {
  console.log('[Ollama] streamOllama called:', { model, messageCount: messages.length })
  console.log('[Ollama] Request URL:', `${OLLAMA_BASE}/api/chat`)

  const requestBody = {
    model,
    messages,
    stream: true,
  }
  console.log('[Ollama] Request body:', JSON.stringify(requestBody).substring(0, 500) + '...')

  let response: Response
  try {
    response = await fetch(`${OLLAMA_BASE}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(requestBody),
    })
    console.log('[Ollama] Response status:', response.status, response.statusText)
  } catch (fetchErr) {
    console.error('[Ollama] Fetch FAILED:', fetchErr)
    throw new Error(`Failed to connect to Ollama at ${OLLAMA_BASE}: ${fetchErr}`)
  }

  if (!response.ok) {
    const err = await response.text()
    console.error('[Ollama] API error response:', err)
    throw new Error(`Ollama API error: ${response.status} ${err}`)
  }

  if (!response.body) {
    console.error('[Ollama] Response body is null')
    throw new Error('Ollama response body is null')
  }

  const reader = response.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  let chunkCount = 0

  console.log('[Ollama] Starting to read stream...')

  while (true) {
    const { done, value } = await reader.read()
    if (done) {
      console.log('[Ollama] Stream ended, total chunks:', chunkCount)
      break
    }

    buffer += decoder.decode(value, { stream: true })
    const lines = buffer.split('\n')
    buffer = lines.pop() || ''

    for (const line of lines) {
      if (line.trim()) {
        try {
          const chunk: LlmChunk = JSON.parse(line)
          chunkCount++
          if (chunk.message?.content) {
            if (chunkCount % 10 === 0) {
              console.log(`[Ollama] Chunk ${chunkCount}, content length: ${chunk.message.content.length}`)
            }
            yield chunk.message.content
          }
          if (chunk.done) {
            console.log('[Ollama] Chunk indicates done, duration:', chunk.total_duration, 'eval_count:', chunk.eval_count)
          }
        } catch (parseErr) {
          console.error('[Ollama] Failed to parse chunk:', line, parseErr)
        }
      }
    }
  }

  console.log('[Ollama] streamOllama complete, total chunks yielded:', chunkCount)
}

export async function callOllama(
  model: string,
  messages: Array<{ role: string; content: string }>,
): Promise<string> {
  console.log('[Ollama] callOllama called:', { model, messageCount: messages.length })

  const response = await fetch(`${OLLAMA_BASE}/api/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model,
      messages,
      stream: false,
    }),
  })

  if (!response.ok) {
    const err = await response.text()
    throw new Error(`Ollama API error: ${response.status} ${err}`)
  }

  const data = await response.json()
  return data.message?.content || ''
}

export function getOllamaModel(): string {
  const model = localStorage.getItem('ollama_model') || 'llama3.2'
  console.log('[Ollama] getOllamaModel:', model)
  return model
}

export function setOllamaModel(model: string): void {
  console.log('[Ollama] setOllamaModel:', model)
  localStorage.setItem('ollama_model', model)
}

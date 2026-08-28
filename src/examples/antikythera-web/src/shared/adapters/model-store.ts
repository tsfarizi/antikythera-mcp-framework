/**
 * Local persistence for the LLM model selection.
 *
 * Pure storage — no fetch, no provider knowledge. The model name is passed to
 * the runtime `llm` option; the actual LLM call is proxied by the Antikythera
 * server (R6: no direct provider calls from the client).
 */

const MODEL_STORAGE_KEY = 'antikythera_model'
const DEFAULT_MODEL = 'gpt-oss:120b-cloud'

export function getModel(): string {
  const stored = localStorage.getItem(MODEL_STORAGE_KEY)
  return stored || DEFAULT_MODEL
}

export function setModel(model: string): void {
  localStorage.setItem(MODEL_STORAGE_KEY, model)
}

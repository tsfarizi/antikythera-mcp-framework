/**
 * Wire vocabulary and runner-mapping helpers for the Antikythera runtime
 * bridge. Shapes follow `documentation/WIRE_PROTOCOL.md` and
 * `contracts/shared/wire_protocol.golden.json`.
 */

/** Wire protocol constants. */
export const WIRE: {
  LLM_CALL: string;
  TOOLS_EXECUTE: string;
  TOOLS_LIST: string;
  EVENTS: string;
  OWNER_CLIENT: 'client';
  OWNER_SERVER: 'server';
  OWNER_MCP: 'mcp';
  HOOK_PREPARE_TURN: 'prepare-turn';
  HOOK_DECIDE_ACTION: 'decide-action';
  HOOK_HANDLE_TOOL_RESULT: 'handle-tool-result';
  PASSTHROUGH: string;
};

/** Join a base URL and a path, tolerating a trailing slash on the base. */
export function joinUrl(serverUrl: string, path: string): string;

/** Opaque id generator for client_id and correlation defaults. */
export function randomId(prefix: string): string;

export interface LlmRequestInput {
  provider?: string | null;
  model?: string | null;
  sessionId?: string | null;
  messagesJson?: string;
  forceJson?: boolean;
  temperature?: number | null;
  maxTokens?: number | null;
  schemaName?: string | null;
  metadataJson?: string | null;
}

/** Wire `llm-request` body (snake_case, golden `llm_call_request`). */
export interface LlmRequest {
  provider: string | null;
  model: string | null;
  session_id: string | null;
  messages_json: string;
  force_json: boolean;
  temperature: number | null;
  max_tokens: number | null;
  schema_name: string | null;
  metadata_json: string | null;
}

export function buildLlmRequest(input: LlmRequestInput): LlmRequest;

/** Wire `llm-response` body (golden `llm_call_response`). */
export interface LlmResponse {
  content: string;
  model: string | null;
  session_id: string | null;
  message_json: string | null;
  tokens_used: number | null;
  finish_reason: string | null;
  raw_response_json: string | null;
}

export function parseLlmResponse(body: unknown): LlmResponse;

export interface ToolCallEventInput {
  toolName: string;
  argumentsJson: string;
  sessionId?: string | null;
  stepId?: number;
}

/** Wire `tool-call-event` body (kebab-case, golden `tool_execute_request`). */
export interface ToolCallEvent {
  'tool-name': string;
  'arguments-json': string;
  'session-id': string | null;
  'step-id': number;
}

export function buildToolCallEvent(input: ToolCallEventInput): ToolCallEvent;

/** Wire `tool-execution-result` body (kebab-case, golden `tool_execute_response`). */
export interface ToolExecutionResultWire {
  'tool-name': string;
  success: boolean;
  'output-json': string;
  'error-message': string | null;
  'step-id': number;
}

export function parseToolExecutionResult(body: unknown): ToolExecutionResultWire;

/** Runner `ToolResultInput` (WIRE_PROTOCOL §6 mapping; step_id dropped). */
export interface ToolResultInput {
  tool_name: string;
  success: boolean;
  output_json: string;
  error_message: string | null;
  correlation_id: string | null;
}

export function wireToRunnerToolResult(
  wireResult: ToolExecutionResultWire,
  correlationId?: string | null,
): ToolResultInput;

/** Wire SSE event envelope (golden `*_event` shapes). */
export interface WireEventEnvelope {
  type: string;
  correlation_id: string | null;
  session_id: string | null;
  client_id: string | null;
  payload: unknown;
}

export function parseEventEnvelope(data: unknown): WireEventEnvelope;

export interface PostbackInput {
  correlationId: string;
  ok: boolean;
  payload?: unknown;
  error?: string | null;
}

/** POST-back body (golden `postback_response` / `postback_gate_denial`). */
export interface Postback {
  correlation_id: string;
  ok: boolean;
  payload: unknown;
  error: string | null;
}

export function buildPostback(input: PostbackInput): Postback;

/** Client-owned tool entry: definition plus a callable handler. */
export interface ToolEntry {
  definition: ToolDefinition;
  handler: ToolHandler;
}

export interface ToolDefinition {
  name: string;
  title?: string;
  description: string;
  parameters?: ToolParameterSchema[];
  input_schema?: Record<string, unknown> | null;
  output_schema?: Record<string, unknown> | null;
}

export interface ToolParameterSchema {
  name: string;
  param_type: string;
  description: string;
  required: boolean;
}

export type ToolHandlerResult =
  | { success: boolean; output?: unknown; error?: string }
  | unknown;

export type ToolHandler = (
  args: Record<string, unknown>,
) => ToolHandlerResult | Promise<ToolHandlerResult>;

export function normalizeLocalTools(
  tools?: ToolEntry[],
): Array<{ definition: ToolDefinition; handler: ToolHandler }>;

export function runToolHandler(
  handler: ToolHandler,
  args: unknown,
): Promise<{ success: boolean; output_json: string; error_message: string | null }>;

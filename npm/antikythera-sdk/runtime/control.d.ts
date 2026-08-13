/**
 * Control-channel event semantics shared by both core modes (tool-execution
 * requests, hook requests, LLM tokens, event forwarding) and the runtime
 * hooks provider wiring.
 */

/** Wrap a user hook into the runtime-hooks `(a, b) => string` contract. */
export function wrapHookFunction(hook: (a: string, b: string) => unknown): (a: string, b: string) => string;

/** Inject the client hooks provider into `globalThis.__ANTIKYTHERA_RUNTIME_HOOKS_PROVIDER__`. */
export function installRuntimeHooksProvider(
  hooks: RuntimeHooks | null,
): void;

export interface RuntimeHooks {
  prepareTurn?: (requestJson: string, sessionStateJson: string) => unknown;
  decideAction?: (sessionStateJson: string, llmResponseJson: string) => unknown;
  handleToolResult?: (sessionStateJson: string, toolResultJson: string) => unknown;
}

/** Resolve the hook decision for a wire `hook-request` payload. */
export function invokeHook(
  hook: 'prepare-turn' | 'decide-action' | 'handle-tool-result' | string,
  inputJson: string | null,
  sessionStateJson: string | null,
  hooks?: RuntimeHooks | null,
): string;

export interface PermissionGate {
  check(toolName: string): void;
  allows?(toolName: string): boolean;
}

export interface ControlHandlerOptions {
  transport: import('./transport').Transport;
  localEntries: Array<{ definition: import('./types').ToolDefinition; handler: import('./types').ToolHandler }>;
  hooks: RuntimeHooks | null;
  gate: PermissionGate;
  emit: (event: object) => void;
  onLlmToken?: (payload: Record<string, unknown>) => void;
  onRegistrySync?: (definitions: object[]) => void;
}

export interface ControlHandler {
  handle(envelope: import('./types').WireEventEnvelope): Promise<void>;
}

export function createControlHandler(options: ControlHandlerOptions): ControlHandler;

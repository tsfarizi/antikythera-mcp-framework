'use strict';

const {
  WIRE,
  buildPostback,
  runToolHandler,
} = require('./types.js');

/**
 * Control-channel event semantics shared by both core modes. Given a wire
 * event envelope it decides what the runtime must do: execute client-owned
 * tools and POST-back, answer hook requests with the client hook decision,
 * feed LLM tokens to the runner (client core) or the UI (server core), and
 * forward progress/lifecycle events.
 */

const HOOK_FN_BY_WIRE_NAME = {
  [WIRE.HOOK_PREPARE_TURN]: 'prepareTurn',
  [WIRE.HOOK_DECIDE_ACTION]: 'decideAction',
  [WIRE.HOOK_HANDLE_TOOL_RESULT]: 'handleToolResult',
};

/**
 * Wrap a user hook into the runtime-hooks contract:
 * `(a: string, b: string) => string`. Non-string returns are JSON-encoded;
 * thrown `Error` objects become plain strings (BUILD.md gate-error rule).
 * @param {Function} hook
 * @returns {Function}
 */
function wrapHookFunction(hook) {
  return (a, b) => {
    try {
      const result = hook(a, b);
      return typeof result === 'string' ? result : JSON.stringify(result);
    } catch (err) {
      if (typeof err === 'string') throw err;
      throw err instanceof Error ? err.message : String(err);
    }
  };
}

const RUNTIME_HOOKS_PROVIDER_SLOT = '__ANTIKYTHERA_RUNTIME_HOOKS_PROVIDER__';
const RUNTIME_HOOK_FN_NAMES = ['prepareTurn', 'decideAction', 'handleToolResult'];

/**
 * Hooks provider ownership (R3): several runtimes may coexist in one process,
 * so the global provider is owned by tokens, not by "whoever installed last".
 * Owners live in an insertion-ordered ledger; the most recent owner's
 * provider is what sits in the global slot, and releasing an owner restores
 * the previous one until zero owners remain.
 */

/** Insertion-ordered owners: opaque token -> wrapped provider (null = no hook fn given). */
const runtimeHooksProviderOwners = new Map();

/** Last value this module wrote to the slot; divergence means an external writer reset it. */
let publishedRuntimeHooksProviderValue;

/** Token of the legacy installRuntimeHooksProvider() install, if any. */
let legacyHooksOwnerToken = null;

/**
 * Wrap the caller-provided hooks into a global-slot provider. Returns null
 * when nothing callable was provided (parity with the legacy install rule:
 * absence of any hook function removes the provider).
 */
function buildRuntimeHooksProvider(hooks) {
  if (!hooks || typeof hooks !== 'object') return null;
  const provider = {};
  let hasHook = false;
  for (const name of RUNTIME_HOOK_FN_NAMES) {
    if (typeof hooks[name] === 'function') {
      provider[name] = wrapHookFunction(hooks[name]);
      hasHook = true;
    }
  }
  return hasHook ? provider : null;
}

/**
 * Reconcile the ledger with the shared slot: an external writer that deleted
 * or replaced the global invalidates ALL outstanding owners — restoration
 * state can no longer be trusted, and keeping it would resurrect a stale
 * provider.
 */
function reconcileRuntimeHooksProviders() {
  if (globalThis[RUNTIME_HOOKS_PROVIDER_SLOT] === publishedRuntimeHooksProviderValue) return;
  runtimeHooksProviderOwners.clear();
  publishedRuntimeHooksProviderValue = undefined;
}

/**
 * Publish the most recently acquired owner's provider into the global slot,
 * or clear the slot when no owner remains.
 */
function publishRuntimeHooksProvider() {
  let active = null;
  for (const provider of runtimeHooksProviderOwners.values()) {
    active = provider;
  }
  if (active) {
    globalThis[RUNTIME_HOOKS_PROVIDER_SLOT] = active;
    publishedRuntimeHooksProviderValue = active;
  } else {
    delete globalThis[RUNTIME_HOOKS_PROVIDER_SLOT];
    publishedRuntimeHooksProviderValue = undefined;
  }
}

/**
 * Register a hooks owner and publish its provider as the active one.
 * @param {{ prepareTurn?: Function, decideAction?: Function, handleToolResult?: Function }|null} hooks
 * @returns {object} opaque ownership token; pass it to releaseRuntimeHooksProvider()
 */
function acquireRuntimeHooksProvider(hooks) {
  reconcileRuntimeHooksProviders();
  const token = {};
  runtimeHooksProviderOwners.set(token, buildRuntimeHooksProvider(hooks));
  publishRuntimeHooksProvider();
  return token;
}

/**
 * Drop a hooks owner. Unknown/double-released tokens are a safe no-op. When
 * the released owner was active, the previous surviving owner's provider is
 * restored; the global slot is removed only when zero owners remain.
 * @param {object} token - token returned by acquireRuntimeHooksProvider()
 * @returns {boolean} true when a live owner was released
 */
function releaseRuntimeHooksProvider(token) {
  reconcileRuntimeHooksProviders();
  if (!runtimeHooksProviderOwners.delete(token)) return false;
  publishRuntimeHooksProvider();
  return true;
}

/**
 * Legacy single-runtime entry point, re-implemented on top of the same
 * ownership ledger so both APIs share one mechanism. The install owns its
 * token; installing again replaces it; installing null/empty releases only
 * this legacy owner instead of clobbering other runtimes' providers.
 * @param {{ prepareTurn?: Function, decideAction?: Function, handleToolResult?: Function }|null} hooks
 * @returns {void}
 */
function installRuntimeHooksProvider(hooks) {
  reconcileRuntimeHooksProviders();
  if (legacyHooksOwnerToken) {
    const staleToken = legacyHooksOwnerToken;
    legacyHooksOwnerToken = null;
    releaseRuntimeHooksProvider(staleToken);
  }
  if (!hooks || typeof hooks !== 'object' || !buildRuntimeHooksProvider(hooks)) {
    return;
  }
  legacyHooksOwnerToken = acquireRuntimeHooksProvider(hooks);
}

/**
 * Resolve the hook decision for a wire `hook-request` payload. Argument order
 * follows the runtime-hooks WIT interface: prepareTurn(request, state);
 * decideAction(state, response); handleToolResult(state, result).
 * @param {string} hook - 'prepare-turn' | 'decide-action' | 'handle-tool-result'
 * @param {string|null} inputJson
 * @param {string|null} sessionStateJson
 * @param {object} [hooks]
 * @returns {string} - JSON decision string
 */
function invokeHook(hook, inputJson, sessionStateJson, hooks) {
  const fnName = HOOK_FN_BY_WIRE_NAME[hook];
  const fn = hooks && typeof hooks[fnName] === 'function' ? hooks[fnName] : null;
  if (!fn) return WIRE.PASSTHROUGH;
  const call = wrapHookFunction(fn);
  if (hook === WIRE.HOOK_PREPARE_TURN) {
    return call(inputJson ?? '', sessionStateJson ?? '');
  }
  return call(sessionStateJson ?? '', inputJson ?? '');
}

/**
 * @param {object} options
 * @param {object} options.transport - createTransport() instance
 * @param {Array<{definition: object, handler: Function}>} options.localEntries
 * @param {object|null} options.hooks - client hooks provider
 * @param {{ check: (name: string) => void, allows?: (name: string) => boolean }} options.gate
 * @param {(event: object) => void} options.emit
 * @param {(payload: object) => void} [options.onLlmToken]
 * @param {() => string|null} [options.getSessionId]
 * @param {(definitions: Array<object>) => void} [options.onRegistrySync]
 * @returns {{ handle: (envelope: object) => Promise<void> }}
 */
function createControlHandler(options) {
  const transport = options.transport;
  const localByHandler = new Map(
    (options.localEntries ?? []).map((entry) => [entry.definition.name, entry]),
  );
  const emit = options.emit;
  const onLlmToken = options.onLlmToken ?? ((payload) => emit({ type: 'llm-token', payload }));
  const onRegistrySync = options.onRegistrySync ?? ((defs) => emit({ type: 'registry-sync', definitions: defs }));

  async function postback(correlationId, body) {
    if (!correlationId) {
      emit({ type: 'error', error: 'control: postback skipped (no correlation_id)' });
      return;
    }
    try {
      await transport.postback(correlationId, body);
    } catch (err) {
      emit({
        type: 'error',
        error: `control: postback ${correlationId} failed: ${err instanceof Error ? err.message : String(err)}`,
      });
    }
  }

  async function executeClientTool(toolName, args) {
    options.gate.check(toolName);
    const entry = localByHandler.get(toolName);
    if (!entry) {
      throw new Error(`permission: tool '${toolName}' is not owned by this client`);
    }
    return runToolHandler(entry.handler, args);
  }

  async function handleToolExecutionRequest(envelope) {
    const correlationId = envelope.correlation_id;
    const call = envelope.payload ?? {};
    const toolName = call['tool-name'];
    const args = parseArgumentsJson(call['arguments-json']);
    if (typeof toolName !== 'string' || !toolName) {
      await postback(correlationId, buildPostback({
        correlationId,
        ok: false,
        error: 'permission: tool-execution-request missing tool-name',
      }));
      return;
    }
    try {
      const result = await executeClientTool(toolName, args);
      const payload = {
        'tool-name': toolName,
        success: result.success,
        'output-json': result.output_json,
        'error-message': result.error_message,
        'step-id': typeof call['step-id'] === 'number' ? call['step-id'] : 0,
      };
      await postback(correlationId, buildPostback({ correlationId, ok: true, payload }));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      await postback(correlationId, buildPostback({
        correlationId,
        ok: false,
        error: message.startsWith('permission:') ? message : `permission: ${message}`,
      }));
    }
  }

  async function handleHookRequest(envelope) {
    const correlationId = envelope.correlation_id;
    const payload = envelope.payload ?? {};
    try {
      const decision = invokeHook(payload.hook, payload.input_json, payload.session_state_json, options.hooks);
      await postback(correlationId, buildPostback({ correlationId, ok: true, payload: decision }));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      await postback(correlationId, buildPostback({ correlationId, ok: false, error: message }));
    }
  }

  async function handle(envelope) {
    try {
      switch (envelope.type) {
        case 'tool-execution-request':
          await handleToolExecutionRequest(envelope);
          return;
        case 'hook-request':
          await handleHookRequest(envelope);
          return;
        case 'llm-token':
          onLlmToken(envelope.payload ?? {});
          return;
        case 'event-forward':
          emit({ type: 'event-forward', sessionId: envelope.session_id, payload: envelope.payload });
          return;
        case 'registry-sync':
          onRegistrySync(Array.isArray(envelope.payload) ? envelope.payload : []);
          return;
        case 'lifecycle':
          emit({ type: 'lifecycle', payload: envelope.payload ?? {} });
          return;
        case 'error':
          emit({ type: 'error', payload: envelope.payload ?? {} });
          return;
        default:
          emit({ type: envelope.type, payload: envelope.payload });
      }
    } catch (err) {
      emit({ type: 'error', error: err instanceof Error ? err.message : String(err) });
    }
  }

  return { handle };
}

function parseArgumentsJson(argumentsJson) {
  if (typeof argumentsJson !== 'string' || !argumentsJson) return {};
  try {
    const parsed = JSON.parse(argumentsJson);
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

module.exports = {
  createControlHandler,
  installRuntimeHooksProvider,
  acquireRuntimeHooksProvider,
  releaseRuntimeHooksProvider,
  wrapHookFunction,
  invokeHook,
};

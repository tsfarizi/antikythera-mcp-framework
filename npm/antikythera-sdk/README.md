# antikythera-agent

General-purpose agent runtime with multi-agent orchestration, MCP tool integration, and WebAssembly support.

## Installation

```bash
npm install antikythera-agent
```

## Quick Start

### Browser Usage

```javascript
import { PromptManager } from 'antikythera-agent';

const prompts = new PromptManager();

prompts.register({
  id: 'assistant',
  name: 'Assistant',
  content: 'You are a helpful assistant.',
  tags: ['general']
});

const content = prompts.getContent('assistant');
```

### WASM via jco component (recommended)

The browser WASM path is a **composite** WASI component (`wasm32-wasip1`) transpiled with `@bytecodealliance/jco`: the `antikythera-sdk` runner component is composed with the embedded `antikythera-toolrunner` component and the `antikythera-default-hooks` passthrough provider (`wasm-tools compose`), so builtin tools execute inside the module without host round-trip and the pipeline defaults match the SDK alone. Import the `runner` namespace from `antikythera-agent/component`:

```javascript
import { runner } from 'antikythera-agent/component';

// Initialize the agent runner — returns the raw session id (plain string, not JSON)
const sessionId = runner.init(JSON.stringify({ session_id: 's1' }));

// Get the agent state for a session
const state = runner.getState(sessionId);
```

The `runner` namespace exposes 16 camelCase functions (all payloads are JSON strings): `init`, `prepareUserTurn`, `commitLlmResponse`, `commitLlmStream`, `processLlmResponseForSession`, `processToolResultForSession`, `appendLlmChunk`, `drainEvents`, `getState`, `resetSession`, `sweepIdleSessions`, `registerTools`, `getToolsPrompt`, `setContextPolicy`, `getTelemetrySnapshot`, `getSloSnapshot`.

> Note: the transpiled module uses top-level await — set your bundler build target to ES2022 (e.g. Vite `build.target: 'es2022'`).

### WASM Initialization (legacy wasm-bindgen, deprecated)

The `antikythera_wasm_bindgen` export is the **legacy** wasm-bindgen browser path (`wasm32-unknown-unknown`). It is deprecated and kept only for compatibility during the transition to the WASI component + jco path above.

```javascript
import init from 'antikythera-agent/antikythera_wasm_bindgen';

// Initialize WASM with browser binary
await init();
```

### Multi-Agent Orchestration

```javascript
const { PromptManager, Orchestrator } = require('antikythera-agent');

const prompts = new PromptManager();
prompts.register({ id: 'coder', name: 'Coder', content: 'You are a software engineer.' });
prompts.register({ id: 'reviewer', name: 'Reviewer', content: 'You are a code reviewer.' });

const orchestrator = new Orchestrator({ executionMode: 'auto' });

orchestrator.registerAgent({
  id: 'coder',
  name: 'Coder',
  role: 'developer',
  systemPrompt: prompts.getContent('coder')
});

orchestrator.registerAgent({
  id: 'reviewer',
  name: 'Reviewer',
  role: 'reviewer',
  systemPrompt: prompts.getContent('reviewer')
});

const result = await orchestrator.dispatch('Write and review code');
```

### Load Prompts from JSON

```javascript
const { PromptManager } = require('antikythera-agent');

// Load from file
const prompts = PromptManager.fromFile('prompts.json');

// Or load from string
const prompts = PromptManager.fromJSON('[{"id":"agent","name":"Agent","content":"You are helpful."}]');
```

## Package Contents

| File | Description |
|:-----|:------------|
| `index.js` | Main exports (PromptManager, SessionManager) |
| `component/` | Composite WASI component (SDK runner + embedded toolrunner + default-hooks) transpiled with jco — ESM bindings, namespace `runner` (camelCase), WASI stubs under `wasi-stubs/` |
| `antikythera_wasm_bindgen.js` | **Legacy** wasm-bindgen WASM glue code for browser (deprecated) |
| `antikythera_wasm_bindgen_bg.wasm` | **Legacy** browser WASM binary (deprecated) |

## Requirements

- Node.js with ESM support

## License

MIT

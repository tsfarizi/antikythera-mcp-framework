# antikythera-agent

General-purpose agent runtime with multi-agent orchestration, MCP tool integration, and WebAssembly support.

## Installation

```bash
npm install antikythera-agent
```

## Quick Start

### Create an Agent

```javascript
const { PromptManager } = require('antikythera-agent');

const prompts = new PromptManager();

prompts.register({
  id: 'assistant',
  name: 'Assistant',
  content: 'You are a helpful assistant.',
  tags: ['general']
});

const content = prompts.getContent('assistant');
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

## Requirements

- Node.js >= 18.0.0

## License

MIT

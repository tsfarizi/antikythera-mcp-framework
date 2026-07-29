# @antikythera/sdk

Agent runtime with multi-agent orchestration, session management, and MCP tool integration. Powered by WebAssembly.

## Installation

```bash
npm install @antikythera/sdk
```

## Quick Start

### Setup with PromptManager

```typescript
import { PromptManager, Agent } from '@antikythera/sdk';

// Initialize with built-in prompts
const prompts = new PromptManager();

// Use a built-in prompt
const agent = new Agent({
  provider: 'openai',
  model: 'gpt-4o',
  systemPrompt: prompts.getContent('coder')
});

const result = await agent.run('Write a sorting algorithm');
console.log(result.output);
```

### Custom Prompts

```typescript
import { PromptManager, Agent } from '@antikythera/sdk';

const prompts = new PromptManager();

// Register custom prompts
prompts.register({
  id: 'my-reviewer',
  name: 'Security Reviewer',
  content: 'You are a security-focused code reviewer. Identify vulnerabilities and security issues.',
  tags: ['reviewer', 'security']
});

// Use custom prompt
const agent = new Agent({
  provider: 'openai',
  model: 'gpt-4o',
  systemPrompt: prompts.getContent('my-reviewer')
});
```

### Multi-Agent with Centralized Prompts

```typescript
import { PromptManager, Orchestrator } from '@antikythera/sdk';

const prompts = new PromptManager();

// Register all agent prompts
prompts.register({
  id: 'backend-dev',
  name: 'Backend Developer',
  content: 'You are a backend developer specializing in APIs and databases.',
  tags: ['developer', 'backend']
});

prompts.register({
  id: 'frontend-dev',
  name: 'Frontend Developer',
  content: 'You are a frontend developer specializing in UI/UX.',
  tags: ['developer', 'frontend']
});

// Create orchestrator with prompts
const orchestrator = new Orchestrator();

orchestrator.registerAgent({
  id: 'backend',
  name: 'Backend Dev',
  role: 'developer',
  systemPrompt: prompts.getContent('backend-dev')
});

orchestrator.registerAgent({
  id: 'frontend',
  name: 'Frontend Dev',
  role: 'developer',
  systemPrompt: prompts.getContent('frontend-dev')
});

// Dispatch tasks
const result = await orchestrator.dispatch('Build a REST API');
```

### Export/Import Prompts

```typescript
import { PromptManager } from '@antikythera/sdk';

const prompts = new PromptManager();

// Export all prompts to JSON
const json = prompts.export();

// Save to file
fs.writeFileSync('prompts.json', json);

// Later, load from file
const loaded = PromptManager.fromFile('prompts.json');
```

## API Reference

### PromptManager

Centralized prompt management.

```typescript
class PromptManager {
  constructor(includeBuiltins?: boolean);
  register(config: PromptConfig): void;
  update(id: string, updates: Partial<PromptConfig>): void;
  get(id: string): PromptConfig | undefined;
  getByTag(tag: string): PromptConfig[];
  list(): PromptConfig[];
  has(id: string): boolean;
  remove(id: string): boolean;
  getContent(id: string): string | undefined;
  export(): string;
  import(json: string): void;
  static fromFile(filePath: string): PromptManager;
  static fromJSON(json: string): PromptManager;
}
```

### Built-in Prompts

| ID | Name | Tags |
|:---|:-----|:-----|
| `coder` | Code Writer | coder, engineering, default |
| `reviewer` | Code Reviewer | reviewer, quality, default |
| `analyst` | Data Analyst | analyst, data, default |
| `researcher` | Researcher | researcher, analysis, default |
| `architect` | Software Architect | architect, design, default |
| `debugger` | Debugger | debugger, troubleshooting, default |
| `documenter` | Technical Writer | documentation, writing, default |
| `security` | Security Analyst | security, audit, default |
| `optimizer` | Performance Optimizer | performance, optimization, default |
| `tester` | QA Engineer | testing, quality, default |

## Requirements

- Node.js >= 18.0.0

## License

MIT

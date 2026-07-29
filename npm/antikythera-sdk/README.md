# antikythera-agent

Multi-agent orchestration with MCP tool integration. Powered by WebAssembly.

## Installation

```bash
npm install antikythera-agent
```

## Quick Start

### Setup with PromptManager

```typescript
import { PromptManager, Agent } from 'antikythera-agent';

const prompts = new PromptManager();

const agent = new Agent({
  provider: 'openai',
  model: 'gpt-4o',
  systemPrompt: prompts.getContent('coder')
});

const result = await agent.run('Write a sorting algorithm');
console.log(result.output);
```

### Multi-Agent Orchestration

```typescript
import { PromptManager, Orchestrator } from 'antikythera-agent';

const prompts = new PromptManager();
const orchestrator = new Orchestrator({ executionMode: 'auto' });

orchestrator.registerAgent({
  id: 'coder',
  name: 'Code Writer',
  role: 'developer',
  systemPrompt: prompts.getContent('coder')
});

orchestrator.registerAgent({
  id: 'reviewer',
  name: 'Code Reviewer',
  role: 'reviewer',
  systemPrompt: prompts.getContent('reviewer')
});

const result = await orchestrator.dispatch('Write and review code');
```

## Built-in Prompts

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

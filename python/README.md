# Antikythera Agent SDK

General-purpose agent runtime with multi-agent orchestration, MCP tool integration, and WebAssembly support.

## Installation

```bash
pip install antikythera-agent
```

For WASM execution support:

```bash
pip install antikythera-agent[wasm]
```

## Quick Start

### Create an Agent

```python
from antikythera_agent import Agent, PromptManager, PromptConfig

prompts = PromptManager()
prompts.register(PromptConfig(
    id="assistant",
    name="Assistant",
    content="You are a helpful assistant.",
))

agent = Agent(
    provider="openai",
    model="gpt-4o",
    system_prompt=prompts.get_content("assistant")
)

result = agent.run("Hello, how can you help me?")
print(result.output)
```

### Multi-Agent Orchestration

```python
from antikythera_agent import Orchestrator, PromptManager, PromptConfig

prompts = PromptManager()
prompts.register(PromptConfig(
    id="coder",
    name="Coder",
    content="You are a software engineer.",
    tags=["engineering"]
))
prompts.register(PromptConfig(
    id="reviewer",
    name="Reviewer",
    content="You are a code reviewer.",
    tags=["quality"]
))

orchestrator = Orchestrator(execution_mode="auto", max_concurrent_tasks=4)

orchestrator.register_agent(AgentProfileConfig(
    id="coder",
    name="Coder",
    role="developer",
    system_prompt=prompts.get_content("coder")
))

orchestrator.register_agent(AgentProfileConfig(
    id="reviewer",
    name="Reviewer",
    role="reviewer",
    system_prompt=prompts.get_content("reviewer")
))

result = orchestrator.dispatch("Write and review code")
```

### WASM Runtime (Server-Side)

```python
from antikythera_agent import WasmRuntime

runtime = WasmRuntime()
result = runtime.call_checked("init", '{"max_steps": 10}')
print(result)
```

### Load Prompts from JSON

```python
from antikythera_agent import PromptManager

# Load from file
prompts = PromptManager.from_file("prompts.json")

# Or load from string
prompts = PromptManager.from_json('[{"id":"agent","name":"Agent","content":"You are helpful."}]')
```

## Requirements

- Python >= 3.9
- wasmtime >= 28.0 (optional, for WASM execution)

## License

MIT

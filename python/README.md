# Antikythera Agent Runtime

Multi-agent orchestration with MCP tool integration. Powered by WebAssembly.

## Installation

```bash
pip install antikythera-agent
```

## Quick Start

### Setup with PromptManager

```python
from antikythera_agent import Agent, PromptManager

# Initialize with built-in prompts
prompts = PromptManager()

# Use a built-in prompt
agent = Agent(
    provider="openai",
    model="gpt-4o",
    system_prompt=prompts.get_content("coder")
)

result = agent.run("Write a sorting algorithm")
print(result.output)
```

### Custom Prompts

```python
from antikythera_agent import Agent, PromptManager, PromptConfig

prompts = PromptManager()

# Register custom prompts
prompts.register(PromptConfig(
    id="my-reviewer",
    name="Security Reviewer",
    content="You are a security-focused code reviewer.",
    tags=["reviewer", "security"]
))

# Use custom prompt
agent = Agent(
    provider="openai",
    model="gpt-4o",
    system_prompt=prompts.get_content("my-reviewer")
)
```

### Multi-Agent with Centralized Prompts

```python
from antikythera_agent import Orchestrator, PromptManager, AgentProfileConfig

prompts = PromptManager()

orchestrator = Orchestrator(execution_mode="auto", max_concurrent_tasks=4)

orchestrator.register_agent(AgentProfileConfig(
    id="coder",
    name="Code Writer",
    role="developer",
    system_prompt=prompts.get_content("coder")
))

orchestrator.register_agent(AgentProfileConfig(
    id="reviewer",
    name="Code Reviewer",
    role="reviewer",
    system_prompt=prompts.get_content("reviewer")
))

result = orchestrator.dispatch("Write and review code")
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

- Python >= 3.9

## License

MIT

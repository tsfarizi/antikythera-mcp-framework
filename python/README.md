# Antikythera MCP Framework

Agent runtime with multi-agent orchestration, session management, and MCP tool integration. Powered by WebAssembly.

## Installation

```bash
pip install antikythera-mcp
```

## Quick Start

### Setup with PromptManager

```python
from antikythera import Agent, PromptManager

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
from antikythera import Agent, PromptManager, PromptConfig

prompts = PromptManager()

# Register custom prompts
prompts.register(PromptConfig(
    id="my-reviewer",
    name="Security Reviewer",
    content="You are a security-focused code reviewer. Identify vulnerabilities and security issues.",
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
from antikythera import Orchestrator, PromptManager, AgentProfileConfig

prompts = PromptManager()

# Register all agent prompts
prompts.register(PromptConfig(
    id="backend-dev",
    name="Backend Developer",
    content="You are a backend developer specializing in APIs and databases.",
    tags=["developer", "backend"]
))

prompts.register(PromptConfig(
    id="frontend-dev",
    name="Frontend Developer",
    content="You are a frontend developer specializing in UI/UX.",
    tags=["developer", "frontend"]
))

# Create orchestrator with prompts
orchestrator = Orchestrator()

orchestrator.register_agent(AgentProfileConfig(
    id="backend",
    name="Backend Dev",
    role="developer",
    system_prompt=prompts.get_content("backend-dev")
))

orchestrator.register_agent(AgentProfileConfig(
    id="frontend",
    name="Frontend Dev",
    role="developer",
    system_prompt=prompts.get_content("frontend-dev")
))

# Dispatch tasks
result = orchestrator.dispatch("Build a REST API")
```

### Export/Import Prompts

```python
from antikythera import PromptManager

prompts = PromptManager()

# Export all prompts to JSON
json_str = prompts.export()

# Save to file
with open("prompts.json", "w") as f:
    f.write(json_str)

# Later, load from file
loaded = PromptManager.from_file("prompts.json")
```

## Type Hints

All classes and functions are fully typed with Python type hints. IDE autocompletion and type checking work out of the box.

```python
from antikythera import Agent, AgentConfig, PromptManager

config: AgentConfig = AgentConfig(provider="openai", model="gpt-4o")
prompts: PromptManager = PromptManager()
agent: Agent = Agent(provider=config.provider, model=config.model, system_prompt=prompts.get_content("coder"))
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

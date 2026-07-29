"""
Antikythera Agent Runtime

Multi-agent orchestration with MCP tool integration.
Powered by WebAssembly.

Example:
    >>> from antikythera_agent import Agent, PromptManager
    >>> prompts = PromptManager()
    >>> agent = Agent(provider="openai", model="gpt-4o", system_prompt=prompts.get_content("coder"))
    >>> result = agent.run("Write a sorting algorithm")
    >>> print(result.output)
"""

from antikythera_agent.agent import Agent
from antikythera_agent.orchestrator import Orchestrator
from antikythera_agent.session import SessionManager
from antikythera_agent.prompts import PromptManager, PromptConfig
from antikythera_agent.types import (
    AgentConfig,
    AgentResult,
    AgentProfileConfig,
    TaskResult,
    PipelineResult,
    OrchestratorConfig,
    SessionInfo,
)
from antikythera_agent.utils import get_version

__version__ = "1.7.1"
__all__ = [
    # Core classes
    "Agent",
    "Orchestrator",
    "SessionManager",
    "PromptManager",
    # Types
    "AgentConfig",
    "AgentResult",
    "AgentProfileConfig",
    "TaskResult",
    "PipelineResult",
    "OrchestratorConfig",
    "SessionInfo",
    "PromptConfig",
    # Utilities
    "get_version",
]

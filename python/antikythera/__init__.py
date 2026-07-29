"""
Antikythera MCP Framework

Agent runtime with multi-agent orchestration, session management,
and MCP tool integration. Powered by WebAssembly.

Example:
    >>> from antikythera import Agent, PromptManager
    >>> prompts = PromptManager()
    >>> agent = Agent(provider="openai", model="gpt-4o", system_prompt=prompts.get_content("coder"))
    >>> result = agent.run("Write a sorting algorithm")
    >>> print(result.output)
"""

from antikythera.agent import Agent
from antikythera.orchestrator import Orchestrator
from antikythera.session import SessionManager
from antikythera.prompts import PromptManager, PromptConfig
from antikythera.types import (
    AgentConfig,
    AgentResult,
    AgentProfileConfig,
    TaskResult,
    PipelineResult,
    OrchestratorConfig,
    SessionInfo,
)
from antikythera.utils import get_version

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

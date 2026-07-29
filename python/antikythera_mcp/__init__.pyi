"""Type stubs for Antikythera MCP SDK."""

from antikythera_mcp.types import (
    AgentConfig as AgentConfig,
    AgentResult as AgentResult,
    AgentProfileConfig as AgentProfileConfig,
    TaskResult as TaskResult,
    PipelineResult as PipelineResult,
    OrchestratorConfig as OrchestratorConfig,
    SessionInfo as SessionInfo,
)
from antikythera_mcp.agent import Agent as Agent
from antikythera_mcp.orchestrator import Orchestrator as Orchestrator
from antikythera_mcp.session import SessionManager as SessionManager
from antikythera_mcp.prompts import PromptManager as PromptManager
from antikythera_mcp.prompts import PromptConfig as PromptConfig
from antikythera_mcp.utils import get_version as get_version

__version__: str

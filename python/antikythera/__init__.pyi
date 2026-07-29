"""Type stubs for Antikythera SDK."""

from antikythera.types import (
    AgentConfig as AgentConfig,
    AgentResult as AgentResult,
    AgentProfileConfig as AgentProfileConfig,
    TaskResult as TaskResult,
    PipelineResult as PipelineResult,
    OrchestratorConfig as OrchestratorConfig,
    SessionInfo as SessionInfo,
)
from antikythera.agent import Agent as Agent
from antikythera.orchestrator import Orchestrator as Orchestrator
from antikythera.session import SessionManager as SessionManager
from antikythera.prompts import PromptManager as PromptManager
from antikythera.prompts import PromptConfig as PromptConfig
from antikythera.utils import get_version as get_version

__version__: str

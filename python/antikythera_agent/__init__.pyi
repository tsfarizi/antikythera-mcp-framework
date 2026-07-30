"""Type stubs for Antikythera Agent Runtime."""

from antikythera_agent.types import (
    AgentConfig as AgentConfig,
    AgentResult as AgentResult,
    AgentProfileConfig as AgentProfileConfig,
    TaskResult as TaskResult,
    PipelineResult as PipelineResult,
    OrchestratorConfig as OrchestratorConfig,
    SessionInfo as SessionInfo,
)
from antikythera_agent.agent import Agent as Agent
from antikythera_agent.orchestrator import Orchestrator as Orchestrator
from antikythera_agent.session import SessionManager as SessionManager
from antikythera_agent.prompts import PromptManager as PromptManager
from antikythera_agent.prompts import PromptConfig as PromptConfig
from antikythera_agent.runtime import WasmRuntime as WasmRuntime
from antikythera_agent.runtime import WasmRuntimeError as WasmRuntimeError
from antikythera_agent.utils import get_version as get_version

__version__: str

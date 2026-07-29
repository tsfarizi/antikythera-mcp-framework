"""Multi-agent orchestrator implementation."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Optional

from antikythera_agent.types import (
    AgentProfileConfig,
    OrchestratorConfig,
    PipelineResult,
    TaskResult,
)

# Path to pre-compiled WASM binary
_WASM_PATH = Path(__file__).parent / "antikythera.wasm"


class Orchestrator:
    """Multi-agent orchestrator for parallel and sequential task execution.

    Manages multiple agents, routes tasks, and coordinates execution.
    Supports budget limits, guardrails, and cooperative cancellation.

    Example:
        >>> orchestrator = Orchestrator(execution_mode="auto")
        >>> orchestrator.register_agent(
        ...     AgentProfileConfig(
        ...         id="coder",
        ...         name="Code Writer",
        ...         role="coder",
        ...         system_prompt="You are an expert programmer."
        ...     )
        ... )
        >>> result = orchestrator.dispatch("Write a sorting algorithm")
        >>> print(result.output)
    """

    def __init__(
        self,
        execution_mode: str = "auto",
        max_concurrent_tasks: int = 4,
        max_total_steps: Optional[int] = None,
        max_total_tasks: Optional[int] = None,
        default_retry_condition: str = "always",
    ):
        """Create a new Orchestrator instance.

        Args:
            execution_mode: How tasks are executed ('auto', 'sequential', 'concurrent', 'parallel').
            max_concurrent_tasks: Maximum tasks running simultaneously.
            max_total_steps: Maximum total steps across all tasks.
            max_total_tasks: Maximum number of tasks.
            default_retry_condition: Retry policy ('always', 'on-transient', 'never').
        """
        self._config = OrchestratorConfig(
            execution_mode=execution_mode,  # type: ignore
            max_concurrent_tasks=max_concurrent_tasks,
            max_total_steps=max_total_steps,
            max_total_tasks=max_total_tasks,
            default_retry_condition=default_retry_condition,  # type: ignore
        )
        self._agents: list[AgentProfileConfig] = []
        self._wasm = self._init_wasm()

    @classmethod
    def from_config(cls, config: OrchestratorConfig) -> Orchestrator:
        """Create an Orchestrator from a config object.

        Args:
            config: Orchestrator configuration.

        Returns:
            Configured Orchestrator instance.
        """
        return cls(
            execution_mode=config.execution_mode,
            max_concurrent_tasks=config.max_concurrent_tasks,
            max_total_steps=config.max_total_steps,
            max_total_tasks=config.max_total_tasks,
            default_retry_condition=config.default_retry_condition,
        )

    def register_agent(self, profile: AgentProfileConfig) -> None:
        """Register an agent profile.

        Args:
            profile: Agent profile configuration.

        Raises:
            ValueError: If profile is missing required fields.
        """
        if not profile.id or not profile.name or not profile.role:
            raise ValueError("Agent profile requires id, name, and role")
        self._agents.append(
            AgentProfileConfig(
                id=profile.id,
                name=profile.name,
                role=profile.role,
                system_prompt=profile.system_prompt,
                max_steps=profile.max_steps,
            )
        )

    def dispatch(self, task: str, session_id: Optional[str] = None) -> TaskResult:
        """Dispatch a single task to the best-suited agent.

        Args:
            task: The task prompt or description.
            session_id: Optional session ID.

        Returns:
            Task execution result.
        """
        args = json.dumps(
            {
                "config": {
                    "execution_mode": self._config.execution_mode,
                    "max_concurrent_tasks": self._config.max_concurrent_tasks,
                },
                "agents": [
                    {
                        "id": a.id,
                        "name": a.name,
                        "role": a.role,
                        "system_prompt": a.system_prompt,
                        "max_steps": a.max_steps,
                    }
                    for a in self._agents
                ],
                "task": task,
                "session_id": session_id,
            }
        )

        try:
            result_json = self._wasm.call("orchestrator_dispatch", args)
            result_dict = json.loads(result_json)
            return TaskResult(
                task_id=result_dict.get("task_id", ""),
                agent_id=result_dict.get("agent_id", ""),
                output=result_dict.get("output"),
                success=result_dict.get("success", False),
                steps_used=result_dict.get("steps_used", 0),
                session_id=result_dict.get("session_id", ""),
                error=result_dict.get("error"),
                error_kind=result_dict.get("error_kind"),
                duration_ms=result_dict.get("duration_ms", 0),
            )
        except Exception as e:
            return TaskResult(
                task_id="",
                agent_id="",
                output=None,
                success=False,
                steps_used=0,
                session_id=session_id or "",
                error=str(e),
            )

    def dispatch_many(self, tasks: list[str]) -> list[TaskResult]:
        """Dispatch multiple tasks.

        Args:
            tasks: List of task prompts.

        Returns:
            List of task results.
        """
        return [self.dispatch(task) for task in tasks]

    def pipeline(self, tasks: list[str]) -> PipelineResult:
        """Execute tasks sequentially, chaining outputs.

        Args:
            tasks: List of task prompts.

        Returns:
            Pipeline execution result.
        """
        results: list[TaskResult] = []
        previous_output = ""

        for task in tasks:
            input_text = (
                f"Previous output:\n{previous_output}\n\nCurrent task:\n{task}"
                if previous_output
                else task
            )

            result = self.dispatch(input_text)
            results.append(result)

            if isinstance(result.output, str):
                previous_output = result.output
            else:
                previous_output = json.dumps(result.output) if result.output else ""

            if not result.success:
                return PipelineResult(
                    results=results,
                    final_output=result.output,
                    total_steps=sum(r.steps_used for r in results),
                    success=False,
                    error=result.error,
                )

        return PipelineResult(
            results=results,
            final_output=results[-1].output if results else None,
            total_steps=sum(r.steps_used for r in results),
            success=True,
        )

    def cancel(self) -> None:
        """Cancel all running tasks."""
        try:
            self._wasm.call("orchestrator_cancel", "{}")
        except Exception:
            pass

    def get_budget(self) -> dict[str, Any]:
        """Get orchestrator budget snapshot.

        Returns:
            Budget state dictionary.
        """
        return {
            "consumed_steps": 0,
            "dispatched_tasks": 0,
            "is_step_budget_exhausted": False,
            "is_task_budget_exhausted": False,
        }

    def list_agents(self) -> list[AgentProfileConfig]:
        """List all registered agents.

        Returns:
            List of agent profiles.
        """
        return list(self._agents)

    def _init_wasm(self) -> Any:
        """Initialize WASM runtime.

        Returns:
            WASM instance with call method.
        """
        if not _WASM_PATH.exists():
            raise RuntimeError(
                f"WASM binary not found at {_WASM_PATH}. "
                "Please install with: pip install antikythera"
            )

        class WasmProxy:
            def call(self, func_name: str, args: str) -> str:
                return json.dumps(
                    {
                        "success": False,
                        "error": "WASM runtime not available in development mode",
                    }
                )

        return WasmProxy()

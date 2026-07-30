"""Agent implementation."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Optional

from antikythera_agent.types import AgentConfig, AgentResult
from antikythera_agent.runtime import WasmRuntime, WasmRuntimeError


class Agent:
    """Single agent execution runtime.

    Creates and runs an agent with a specific provider and model configuration.
    Handles prompt processing, tool calls, and response generation.

    Example:
        >>> agent = Agent(provider="openai", model="gpt-4o")
        >>> result = agent.run("Explain quantum computing")
        >>> print(result.output)
    """

    def __init__(
        self,
        provider: str,
        model: str,
        system_prompt: Optional[str] = None,
        max_steps: int = 8,
        timeout: int = 60000,
    ):
        """Create a new Agent instance.

        Args:
            provider: The LLM provider identifier.
            model: The model name to use.
            system_prompt: Optional system prompt override.
            max_steps: Maximum reasoning steps.
            timeout: Request timeout in milliseconds.
        """
        self._config = AgentConfig(
            provider=provider,
            model=model,
            system_prompt=system_prompt,
            max_steps=max_steps,
            timeout=timeout,
        )
        self._runtime = WasmRuntime()

    @classmethod
    def from_config(cls, config: AgentConfig) -> Agent:
        """Create an Agent from a config object.

        Args:
            config: Agent configuration.

        Returns:
            Configured Agent instance.
        """
        return cls(
            provider=config.provider,
            model=config.model,
            system_prompt=config.system_prompt,
            max_steps=config.max_steps,
            timeout=config.timeout,
        )

    def run(self, prompt: str, session_id: Optional[str] = None) -> AgentResult:
        """Run the agent with a user prompt.

        Args:
            prompt: The user's input prompt.
            session_id: Optional session ID for conversation continuity.

        Returns:
            Agent execution result.

        Raises:
            RuntimeError: If WASM execution fails.
        """
        args = json.dumps(
            {
                "config": {
                    "provider": self._config.provider,
                    "model": self._config.model,
                    "system_prompt": self._config.system_prompt,
                    "max_steps": self._config.max_steps,
                    "timeout": self._config.timeout,
                },
                "prompt": prompt,
                "session_id": session_id,
            }
        )

        try:
            result_dict = self._runtime.call_checked("agent_run", args)
            return AgentResult(
                output=result_dict.get("output", ""),
                success=result_dict.get("success", False),
                steps_used=result_dict.get("steps_used", 0),
                session_id=result_dict.get("session_id", ""),
                error=result_dict.get("error"),
            )
        except WasmRuntimeError as e:
            return AgentResult(
                output="",
                success=False,
                steps_used=0,
                session_id=session_id or "",
                error=str(e),
            )

    def get_config(self) -> AgentConfig:
        """Get the agent's configuration.

        Returns:
            The agent configuration.
        """
        return AgentConfig(
            provider=self._config.provider,
            model=self._config.model,
            system_prompt=self._config.system_prompt,
            max_steps=self._config.max_steps,
            timeout=self._config.timeout,
        )

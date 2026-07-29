"""Agent implementation."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Optional

from antikythera.types import AgentConfig, AgentResult

# Path to pre-compiled WASM binary
_WASM_PATH = Path(__file__).parent / "antikythera.wasm"


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
        self._wasm = self._init_wasm()

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

        # Call WASM export (placeholder - actual implementation depends on WASM ABI)
        try:
            result_json = self._wasm.call("agent_run", args)
            result_dict = json.loads(result_json)
            return AgentResult(
                output=result_dict.get("output", ""),
                success=result_dict.get("success", False),
                steps_used=result_dict.get("steps_used", 0),
                session_id=result_dict.get("session_id", ""),
                error=result_dict.get("error"),
            )
        except Exception as e:
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

        # Placeholder for actual WASM initialization
        # In production, this would use wasmtime or similar
        class WasmProxy:
            def call(self, func_name: str, args: str) -> str:
                # Placeholder implementation
                return json.dumps(
                    {
                        "output": "WASM runtime not available in development mode",
                        "success": False,
                        "error": "WASM binary not loaded",
                    }
                )

        return WasmProxy()

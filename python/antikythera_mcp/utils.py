"""Utility functions."""

from __future__ import annotations

from antikythera_mcp.agent import Agent
from antikythera_mcp.types import AgentConfig

# Built-in role templates
_ROLE_TEMPLATES = {
    "coder": AgentConfig(
        provider="openai",
        model="gpt-4o",
        system_prompt=(
            "You are an expert software engineer. "
            "Write clean, efficient, and well-documented code. "
            "Follow best practices and handle edge cases."
        ),
    ),
    "reviewer": AgentConfig(
        provider="openai",
        model="gpt-4o",
        system_prompt=(
            "You are an expert code reviewer. "
            "Analyze code for bugs, security issues, performance problems, "
            "and style violations. Provide constructive feedback with specific suggestions."
        ),
    ),
    "analyst": AgentConfig(
        provider="openai",
        model="gpt-4o",
        system_prompt=(
            "You are a data analyst. "
            "Analyze data, identify patterns, create visualizations, "
            "and provide actionable insights. Use statistical methods when appropriate."
        ),
    ),
    "researcher": AgentConfig(
        provider="openai",
        model="gpt-4o",
        system_prompt=(
            "You are a thorough researcher. "
            "Gather information from multiple sources, verify facts, "
            "synthesize findings, and present well-structured reports with citations."
        ),
    ),
}


def create_agent_from_role(
    role: str,
    provider: str = "openai",
    model: str = "gpt-4o",
    **kwargs,
) -> Agent:
    """Create an agent with a built-in role template.

    Args:
        role: Role template ('coder', 'reviewer', 'analyst', 'researcher').
        provider: LLM provider override.
        model: Model name override.
        **kwargs: Additional configuration overrides.

    Returns:
        Configured Agent instance.

    Raises:
        ValueError: If role is unknown.

    Example:
        >>> coder = create_agent_from_role("coder")
        >>> reviewer = create_agent_from_role("reviewer", model="gpt-4o")
    """
    if role not in _ROLE_TEMPLATES:
        available = ", ".join(_ROLE_TEMPLATES.keys())
        raise ValueError(f"Unknown role: {role}. Available: {available}")

    base = _ROLE_TEMPLATES[role]
    return Agent(
        provider=provider or base.provider,
        model=model or base.model,
        system_prompt=kwargs.get("system_prompt", base.system_prompt),
        max_steps=kwargs.get("max_steps", base.max_steps),
        timeout=kwargs.get("timeout", base.timeout),
    )


def get_version() -> str:
    """Get SDK version.

    Returns:
        Version string.
    """
    return "1.7.1"

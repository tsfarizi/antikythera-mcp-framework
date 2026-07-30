"""Utility functions."""

from __future__ import annotations

from antikythera_agent.agent import Agent
from antikythera_agent.types import AgentConfig


def get_version() -> str:
    """Get SDK version.

    Returns:
        Version string.
    """
    return "1.7.10"

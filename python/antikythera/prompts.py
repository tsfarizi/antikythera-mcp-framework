"""Centralized prompt management."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


@dataclass
class PromptConfig:
    """Prompt configuration.

    Attributes:
        id: Unique prompt identifier.
        name: Human-readable name.
        content: The prompt content.
        description: What this prompt does.
        tags: Tags for categorization.
    """

    id: str
    name: str
    content: str
    description: Optional[str] = None
    tags: list[str] = field(default_factory=list)


# Built-in prompt templates
_BUILTIN_PROMPTS: list[PromptConfig] = [
    PromptConfig(
        id="coder",
        name="Code Writer",
        content=(
            "You are an expert software engineer. "
            "Write clean, efficient, and well-documented code. "
            "Follow best practices and handle edge cases. "
            "Always consider error handling, performance implications, "
            "and code maintainability."
        ),
        description="General-purpose coding assistant",
        tags=["coder", "engineering", "default"],
    ),
    PromptConfig(
        id="reviewer",
        name="Code Reviewer",
        content=(
            "You are an expert code reviewer. "
            "Analyze code for bugs, security issues, performance problems, "
            "and style violations. Provide constructive feedback with specific "
            "suggestions for improvement. Consider edge cases, error handling, "
            "and maintainability."
        ),
        description="Code review specialist",
        tags=["reviewer", "quality", "default"],
    ),
    PromptConfig(
        id="analyst",
        name="Data Analyst",
        content=(
            "You are a data analyst. "
            "Analyze data, identify patterns, create visualizations, "
            "and provide actionable insights. Use statistical methods "
            "when appropriate. Present findings clearly with supporting evidence."
        ),
        description="Data analysis specialist",
        tags=["analyst", "data", "default"],
    ),
    PromptConfig(
        id="researcher",
        name="Researcher",
        content=(
            "You are a thorough researcher. "
            "Gather information from multiple sources, verify facts, "
            "synthesize findings, and present well-structured reports "
            "with citations. Always cross-reference information and "
            "note any uncertainties."
        ),
        description="Research and analysis specialist",
        tags=["researcher", "analysis", "default"],
    ),
    PromptConfig(
        id="architect",
        name="Software Architect",
        content=(
            "You are a software architect. "
            "Design scalable, maintainable, and robust systems. "
            "Consider trade-offs between complexity and simplicity, "
            "performance and readability. Provide clear diagrams and "
            "documentation for your designs."
        ),
        description="System design specialist",
        tags=["architect", "design", "default"],
    ),
    PromptConfig(
        id="debugger",
        name="Debugger",
        content=(
            "You are an expert debugger. "
            "Systematically identify root causes of issues. "
            "Use logical deduction, check assumptions, and verify hypotheses. "
            "Provide clear explanations of the problem and step-by-step solutions."
        ),
        description="Debugging and troubleshooting specialist",
        tags=["debugger", "troubleshooting", "default"],
    ),
    PromptConfig(
        id="documenter",
        name="Technical Writer",
        content=(
            "You are a technical writer. "
            "Create clear, concise, and well-organized documentation. "
            "Use appropriate formatting, include code examples where helpful, "
            "and ensure information is accurate and up-to-date."
        ),
        description="Documentation specialist",
        tags=["documentation", "writing", "default"],
    ),
    PromptConfig(
        id="security",
        name="Security Analyst",
        content=(
            "You are a security analyst. "
            "Identify vulnerabilities, assess risks, and recommend "
            "security improvements. Follow security best practices "
            "and standards. Consider both technical and operational "
            "security aspects."
        ),
        description="Security analysis specialist",
        tags=["security", "audit", "default"],
    ),
    PromptConfig(
        id="optimizer",
        name="Performance Optimizer",
        content=(
            "You are a performance optimization expert. "
            "Identify bottlenecks, suggest improvements, and measure impact. "
            "Consider both time and space complexity. Balance optimization "
            "with code readability."
        ),
        description="Performance optimization specialist",
        tags=["performance", "optimization", "default"],
    ),
    PromptConfig(
        id="tester",
        name="QA Engineer",
        content=(
            "You are a QA engineer. "
            "Write comprehensive tests, identify edge cases, and ensure "
            "software quality. Consider unit tests, integration tests, "
            "and end-to-end scenarios. Think about both happy paths "
            "and error cases."
        ),
        description="Quality assurance specialist",
        tags=["testing", "quality", "default"],
    ),
]


class PromptManager:
    """Centralized prompt management for all agents.

    Stores, organizes, and provides access to all prompts used by agents.
    Supports built-in prompts, custom prompts, and prompt inheritance.

    Example:
        >>> prompts = PromptManager()
        >>> coder_prompt = prompts.get("coder")
        >>> prompts.register(PromptConfig(
        ...     id="my-reviewer",
        ...     name="My Code Reviewer",
        ...     content="You are a code reviewer specializing in security.",
        ...     tags=["reviewer", "security"]
        ... ))
        >>> reviewer_prompts = prompts.get_by_tag("reviewer")
    """

    def __init__(self, include_builtins: bool = True):
        """Create a new PromptManager.

        Args:
            include_builtins: Include built-in prompt templates.
        """
        self._prompts: dict[str, PromptConfig] = {}
        if include_builtins:
            for prompt in _BUILTIN_PROMPTS:
                self._prompts[prompt.id] = PromptConfig(
                    id=prompt.id,
                    name=prompt.name,
                    content=prompt.content,
                    description=prompt.description,
                    tags=list(prompt.tags),
                )

    def register(self, config: PromptConfig) -> None:
        """Register a prompt configuration.

        Args:
            config: Prompt configuration.

        Raises:
            ValueError: If prompt ID already exists or config is invalid.
        """
        if not config.id or not config.content:
            raise ValueError("Prompt requires id and content")
        if config.id in self._prompts:
            raise ValueError(
                f"Prompt '{config.id}' already exists. Use update() to modify."
            )
        self._prompts[config.id] = PromptConfig(
            id=config.id,
            name=config.name,
            content=config.content,
            description=config.description,
            tags=list(config.tags),
        )

    def update(self, id: str, **kwargs) -> None:
        """Update an existing prompt.

        Args:
            id: Prompt ID.
            **kwargs: Fields to update.

        Raises:
            ValueError: If prompt not found.
        """
        if id not in self._prompts:
            raise ValueError(f"Prompt '{id}' not found")

        existing = self._prompts[id]
        self._prompts[id] = PromptConfig(
            id=id,
            name=kwargs.get("name", existing.name),
            content=kwargs.get("content", existing.content),
            description=kwargs.get("description", existing.description),
            tags=kwargs.get("tags", list(existing.tags)),
        )

    def get(self, id: str) -> Optional[PromptConfig]:
        """Get a prompt by ID.

        Args:
            id: Prompt ID.

        Returns:
            Prompt configuration or None.
        """
        return self._prompts.get(id)

    def get_by_tag(self, tag: str) -> list[PromptConfig]:
        """Get all prompts with a specific tag.

        Args:
            tag: Tag to filter by.

        Returns:
            Matching prompts.
        """
        return [p for p in self._prompts.values() if tag in p.tags]

    def list(self) -> list[PromptConfig]:
        """Get all registered prompts.

        Returns:
            All prompts.
        """
        return list(self._prompts.values())

    def has(self, id: str) -> bool:
        """Check if a prompt exists.

        Args:
            id: Prompt ID.

        Returns:
            Whether prompt exists.
        """
        return id in self._prompts

    def remove(self, id: str) -> bool:
        """Remove a prompt.

        Args:
            id: Prompt ID.

        Returns:
            Whether prompt was removed.
        """
        return self._prompts.pop(id, None) is not None

    def get_content(self, id: str) -> Optional[str]:
        """Get prompt content by ID.

        Args:
            id: Prompt ID.

        Returns:
            Prompt content or None.
        """
        prompt = self._prompts.get(id)
        return prompt.content if prompt else None

    def export(self) -> str:
        """Export all prompts as JSON.

        Returns:
            JSON string.
        """
        prompts = [
            {
                "id": p.id,
                "name": p.name,
                "content": p.content,
                "description": p.description,
                "tags": p.tags,
            }
            for p in self._prompts.values()
        ]
        return json.dumps(prompts, indent=2)

    def import_json(self, json_str: str) -> None:
        """Import prompts from JSON.

        Args:
            json_str: JSON string of prompts.
        """
        prompts = json.loads(json_str)
        for prompt_data in prompts:
            self._prompts[prompt_data["id"]] = PromptConfig(
                id=prompt_data["id"],
                name=prompt_data["name"],
                content=prompt_data["content"],
                description=prompt_data.get("description"),
                tags=prompt_data.get("tags", []),
            )

    @classmethod
    def from_file(cls, file_path: str | Path) -> PromptManager:
        """Create a PromptManager from a JSON file.

        Args:
            file_path: Path to JSON file.

        Returns:
            New PromptManager instance.
        """
        manager = cls(include_builtins=False)
        content = Path(file_path).read_text(encoding="utf-8")
        manager.import_json(content)
        return manager

    @classmethod
    def from_json(cls, json_str: str) -> PromptManager:
        """Create a PromptManager from a JSON string.

        Args:
            json_str: JSON string.

        Returns:
            New PromptManager instance.
        """
        manager = cls(include_builtins=False)
        manager.import_json(json_str)
        return manager

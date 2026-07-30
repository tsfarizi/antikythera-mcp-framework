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


class PromptManager:
    """Centralized prompt management for agents.

    Provides a registry for storing, organizing, and retrieving prompts.
    Start with an empty registry and register your own prompts.

    Example:
        >>> prompts = PromptManager()
        >>> prompts.register(PromptConfig(
        ...     id="my-agent",
        ...     name="My Agent",
        ...     content="You are a helpful assistant.",
        ... ))
        >>> agent = Agent(
        ...     provider="openai",
        ...     model="gpt-4o",
        ...     system_prompt=prompts.get_content("my-agent"),
        ... )
    """

    def __init__(self) -> None:
        self._prompts: dict[str, PromptConfig] = {}

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
        manager = cls()
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
        manager = cls()
        manager.import_json(json_str)
        return manager

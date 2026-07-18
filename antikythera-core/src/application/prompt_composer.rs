//! System prompt composition logic.
//!
//! Extracted from McpClient for single-responsibility.

use crate::application::config::{PromptsConfig, ToolConfig};

/// Composes system prompts from templates and tool guidance.
pub struct PromptComposer<'a> {
    prompts: &'a PromptsConfig,
    tools: &'a [ToolConfig],
}

impl<'a> PromptComposer<'a> {
    pub fn new(prompts: &'a PromptsConfig, tools: &'a [ToolConfig]) -> Self {
        Self { prompts, tools }
    }

    /// Compose the full system prompt with template substitution.
    pub fn compose(&self, override_prompt: Option<String>) -> String {
        let template = self.prompts.template().to_string();
        let custom_instruction = override_prompt.unwrap_or_default();
        if template.is_empty() {
            return custom_instruction.trim().to_string();
        }

        let tool_guidance = self.tool_guidance();

        let mut prompt = template
            .replace("{{language_guidance}}", "")
            .replace("{{tool_guidance}}", tool_guidance.trim())
            .replace("{{custom_instruction}}", custom_instruction.trim());
        prompt = prompt
            .replace("{{language_guidance}}", "")
            .replace("{{tool_guidance}}", "")
            .replace("{{custom_instruction}}", "");

        Self::clean_blank_lines(&prompt)
    }

    /// Build tool guidance string.
    fn tool_guidance(&self) -> String {
        if self.tools.is_empty() {
            return self.prompts.fallback_guidance().to_string();
        }
        let mut text = format!("{}\n", self.prompts.tool_guidance());
        for tool in self.tools {
            let description = tool
                .description
                .as_deref()
                .unwrap_or("No description available.");
            text.push_str(&format!("- {}: {}\n", tool.name, description));
        }
        text.push_str(self.prompts.fallback_guidance());
        text
    }

    /// Clean multiple blank lines.
    fn clean_blank_lines(prompt: &str) -> String {
        let mut cleaned = Vec::new();
        let mut previous_blank = false;
        for line in prompt.lines().map(|line| line.trim_end()) {
            let trimmed = line.trim();
            let is_blank = trimmed.is_empty();
            if is_blank {
                if !previous_blank {
                    cleaned.push(String::new());
                }
                previous_blank = true;
            } else {
                cleaned.push(trimmed.to_string());
                previous_blank = false;
            }
        }
        cleaned.join("\n").trim().to_string()
    }
}

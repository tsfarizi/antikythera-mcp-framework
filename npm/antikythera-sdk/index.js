/**
 * @antikythera/sdk - Antikythera MCP Framework
 *
 * Agent runtime with multi-agent orchestration, session management,
 * and MCP tool integration. Powered by WebAssembly.
 *
 * @packageDocumentation
 */

'use strict';

const fs = require('fs');
const path = require('path');

// ============================================================================
// Types
// ============================================================================

/**
 * @typedef {Object} AgentConfig
 * @property {string} provider - LLM provider identifier
 * @property {string} model - Model name
 * @property {string} [systemPrompt] - System prompt override
 * @property {number} [maxSteps] - Maximum reasoning steps
 * @property {number} [timeout] - Request timeout in ms
 */

/**
 * @typedef {Object} AgentResult
 * @property {string} output - Agent response text
 * @property {boolean} success - Execution success
 * @property {number} stepsUsed - Reasoning steps taken
 * @property {string} sessionId - Session identifier
 * @property {string} [error] - Error message if failed
 */

/**
 * @typedef {Object} AgentProfileConfig
 * @property {string} id - Unique agent ID
 * @property {string} name - Display name
 * @property {string} role - Semantic role
 * @property {string} [systemPrompt] - System prompt
 * @property {number} [maxSteps] - Max steps
 */

/**
 * @typedef {Object} TaskResult
 * @property {string} taskId - Task identifier
 * @property {string} agentId - Agent identifier
 * @property {*} output - Task output
 * @property {boolean} success - Task success
 * @property {number} stepsUsed - Steps taken
 * @property {string} sessionId - Session ID
 * @property {string} [error] - Error message
 * @property {string} [errorKind] - Error classification
 * @property {number} durationMs - Execution time
 */

/**
 * @typedef {Object} PipelineResult
 * @property {TaskResult[]} results - Individual results
 * @property {*} finalOutput - Final output
 * @property {number} totalSteps - Total steps
 * @property {boolean} success - Overall success
 * @property {string} [error] - Error message
 */

/**
 * @typedef {Object} OrchestratorConfig
 * @property {string} [executionMode] - Execution mode
 * @property {number} [maxConcurrentTasks] - Max concurrent tasks
 * @property {number} [maxTotalSteps] - Max total steps
 * @property {number} [maxTotalTasks] - Max total tasks
 * @property {string} [defaultRetryCondition] - Retry policy
 */

/**
 * @typedef {Object} SessionInfo
 * @property {string} sessionId - Session ID
 * @property {string} agentId - Agent ID
 * @property {number} createdAt - Creation timestamp
 * @property {number} lastActivity - Last activity timestamp
 * @property {number} messageCount - Message count
 */

/**
 * @typedef {Object} PromptConfig
 * @property {string} id - Unique prompt identifier
 * @property {string} name - Human-readable name
 * @property {string} content - The prompt content
 * @property {string} [description] - What this prompt does
 * @property {string[]} [tags] - Tags for categorization
 */

// ============================================================================
// PromptManager
// ============================================================================

/**
 * Centralized prompt management for all agents.
 *
 * Stores, organizes, and provides access to all prompts used by agents.
 * Supports built-in prompts, custom prompts, and prompt inheritance.
 *
 * @example
 * ```javascript
 * const prompts = new PromptManager();
 *
 * // Use built-in prompts
 * const coderPrompt = prompts.get('coder');
 *
 * // Register custom prompts
 * prompts.register({
 *   id: 'my-reviewer',
 *   name: 'My Code Reviewer',
 *   content: 'You are a code reviewer specializing in security.',
 *   tags: ['reviewer', 'security']
 * });
 *
 * // Get all prompts for a category
 * const reviewerPrompts = prompts.getByTag('reviewer');
 * ```
 */
class PromptManager {
  /** @type {Map<string, PromptConfig>} */
  #prompts = new Map();

  /**
   * Create a new PromptManager with optional built-in prompts.
   * @param {boolean} [includeBuiltins=true] - Include built-in prompt templates
   */
  constructor(includeBuiltins = true) {
    if (includeBuiltins) {
      this.#registerBuiltins();
    }
  }

  /**
   * Register a prompt configuration.
   * @param {PromptConfig} config - Prompt configuration
   * @throws {Error} If prompt ID already exists
   */
  register(config) {
    if (!config.id || !config.content) {
      throw new Error('Prompt requires id and content');
    }
    if (this.#prompts.has(config.id)) {
      throw new Error(`Prompt '${config.id}' already exists. Use update() to modify.`);
    }
    this.#prompts.set(config.id, { ...config });
  }

  /**
   * Update an existing prompt.
   * @param {string} id - Prompt ID
   * @param {Partial<PromptConfig>} updates - Fields to update
   * @throws {Error} If prompt not found
   */
  update(id, updates) {
    const existing = this.#prompts.get(id);
    if (!existing) {
      throw new Error(`Prompt '${id}' not found`);
    }
    this.#prompts.set(id, { ...existing, ...updates, id });
  }

  /**
   * Get a prompt by ID.
   * @param {string} id - Prompt ID
   * @returns {PromptConfig | undefined} Prompt configuration
   */
  get(id) {
    return this.#prompts.get(id);
  }

  /**
   * Get all prompts with a specific tag.
   * @param {string} tag - Tag to filter by
   * @returns {PromptConfig[]} Matching prompts
   */
  getByTag(tag) {
    return [...this.#prompts.values()].filter(p => p.tags?.includes(tag));
  }

  /**
   * Get all registered prompts.
   * @returns {PromptConfig[]} All prompts
   */
  list() {
    return [...this.#prompts.values()];
  }

  /**
   * Check if a prompt exists.
   * @param {string} id - Prompt ID
   * @returns {boolean} Whether prompt exists
   */
  has(id) {
    return this.#prompts.has(id);
  }

  /**
   * Remove a prompt.
   * @param {string} id - Prompt ID
   * @returns {boolean} Whether prompt was removed
   */
  remove(id) {
    return this.#prompts.delete(id);
  }

  /**
   * Get prompt content by ID.
   * @param {string} id - Prompt ID
   * @returns {string | undefined} Prompt content
   */
  getContent(id) {
    return this.#prompts.get(id)?.content;
  }

  /**
   * Export all prompts as JSON.
   * @returns {string} JSON string
   */
  export() {
    return JSON.stringify([...this.#prompts.values()], null, 2);
  }

  /**
   * Import prompts from JSON.
   * @param {string} json - JSON string of prompts
   */
  import(json) {
    const prompts = JSON.parse(json);
    for (const prompt of prompts) {
      this.#prompts.set(prompt.id, prompt);
    }
  }

  /**
   * Create a PromptManager from a JSON file.
   * @param {string} filePath - Path to JSON file
   * @returns {PromptManager} New PromptManager instance
   */
  static fromFile(filePath) {
    const manager = new PromptManager(false);
    const content = fs.readFileSync(filePath, 'utf-8');
    manager.import(content);
    return manager;
  }

  /**
   * Create a PromptManager from a JSON string.
   * @param {string} json - JSON string
   * @returns {PromptManager} New PromptManager instance
   */
  static fromJSON(json) {
    const manager = new PromptManager(false);
    manager.import(json);
    return manager;
  }

  /** @private */
  #registerBuiltins() {
    const builtins = [
      {
        id: 'coder',
        name: 'Code Writer',
        content: 'You are an expert software engineer. Write clean, efficient, and well-documented code. Follow best practices and handle edge cases. Always consider error handling, performance implications, and code maintainability.',
        description: 'General-purpose coding assistant',
        tags: ['coder', 'engineering', 'default']
      },
      {
        id: 'reviewer',
        name: 'Code Reviewer',
        content: 'You are an expert code reviewer. Analyze code for bugs, security issues, performance problems, and style violations. Provide constructive feedback with specific suggestions for improvement. Consider edge cases, error handling, and maintainability.',
        description: 'Code review specialist',
        tags: ['reviewer', 'quality', 'default']
      },
      {
        id: 'analyst',
        name: 'Data Analyst',
        content: 'You are a data analyst. Analyze data, identify patterns, create visualizations, and provide actionable insights. Use statistical methods when appropriate. Present findings clearly with supporting evidence.',
        description: 'Data analysis specialist',
        tags: ['analyst', 'data', 'default']
      },
      {
        id: 'researcher',
        name: 'Researcher',
        content: 'You are a thorough researcher. Gather information from multiple sources, verify facts, synthesize findings, and present well-structured reports with citations. Always cross-reference information and note any uncertainties.',
        description: 'Research and analysis specialist',
        tags: ['researcher', 'analysis', 'default']
      },
      {
        id: 'architect',
        name: 'Software Architect',
        content: 'You are a software architect. Design scalable, maintainable, and robust systems. Consider trade-offs between complexity and simplicity, performance and readability. Provide clear diagrams and documentation for your designs.',
        description: 'System design specialist',
        tags: ['architect', 'design', 'default']
      },
      {
        id: 'debugger',
        name: 'Debugger',
        content: 'You are an expert debugger. Systematically identify root causes of issues. Use logical deduction, check assumptions, and verify hypotheses. Provide clear explanations of the problem and step-by-step solutions.',
        description: 'Debugging and troubleshooting specialist',
        tags: ['debugger', 'troubleshooting', 'default']
      },
      {
        id: 'documenter',
        name: 'Technical Writer',
        content: 'You are a technical writer. Create clear, concise, and well-organized documentation. Use appropriate formatting, include code examples where helpful, and ensure information is accurate and up-to-date.',
        description: 'Documentation specialist',
        tags: ['documentation', 'writing', 'default']
      },
      {
        id: 'security',
        name: 'Security Analyst',
        content: 'You are a security analyst. Identify vulnerabilities, assess risks, and recommend security improvements. Follow security best practices and standards. Consider both technical and operational security aspects.',
        description: 'Security analysis specialist',
        tags: ['security', 'audit', 'default']
      },
      {
        id: 'optimizer',
        name: 'Performance Optimizer',
        content: 'You are a performance optimization expert. Identify bottlenecks, suggest improvements, and measure impact. Consider both time and space complexity. Balance optimization with code readability.',
        description: 'Performance optimization specialist',
        tags: ['performance', 'optimization', 'default']
      },
      {
        id: 'tester',
        name: 'QA Engineer',
        content: 'You are a QA engineer. Write comprehensive tests, identify edge cases, and ensure software quality. Consider unit tests, integration tests, and end-to-end scenarios. Think about both happy paths and error cases.',
        description: 'Quality assurance specialist',
        tags: ['testing', 'quality', 'default']
      }
    ];

    for (const builtin of builtins) {
      this.#prompts.set(builtin.id, builtin);
    }
  }
}

// ============================================================================
// SessionManager
// ============================================================================

/**
 * Session lifecycle management.
 */
class SessionManager {
  /** @type {Map<string, SessionInfo>} */
  #sessions = new Map();

  /** @type {number} */
  #maxSessions;

  /** @type {number} */
  #ttlMs;

  /**
   * Create a new SessionManager.
   * @param {{ maxSessions?: number, sessionTtlMs?: number }} [config] - Configuration
   */
  constructor(config = {}) {
    this.#maxSessions = config.maxSessions ?? 100;
    this.#ttlMs = config.sessionTtlMs ?? 3600000;
  }

  /**
   * Get or create a session.
   * @param {string} sessionId - Session ID
   * @param {string} agentId - Agent ID
   * @returns {SessionInfo} Session info
   */
  getOrCreate(sessionId, agentId) {
    const now = Date.now();
    let session = this.#sessions.get(sessionId);

    if (session) {
      session.lastActivity = now;
      return session;
    }

    if (this.#sessions.size >= this.#maxSessions) {
      this.#evictExpired();
      if (this.#sessions.size >= this.#maxSessions) {
        const oldest = [...this.#sessions.values()]
          .sort((a, b) => a.lastActivity - b.lastActivity)[0];
        if (oldest) {
          this.#sessions.delete(oldest.sessionId);
        }
      }
    }

    session = {
      sessionId,
      agentId,
      createdAt: now,
      lastActivity: now,
      messageCount: 0
    };

    this.#sessions.set(sessionId, session);
    return session;
  }

  /**
   * Get a session by ID.
   * @param {string} sessionId - Session ID
   * @returns {SessionInfo | null} Session info or null
   */
  get(sessionId) {
    return this.#sessions.get(sessionId) ?? null;
  }

  /**
   * List sessions for an agent.
   * @param {string} agentId - Agent ID
   * @returns {SessionInfo[]} Sessions
   */
  listByAgent(agentId) {
    return [...this.#sessions.values()].filter(s => s.agentId === agentId);
  }

  /**
   * List all sessions.
   * @returns {SessionInfo[]} All sessions
   */
  listAll() {
    return [...this.#sessions.values()];
  }

  /**
   * Remove a session.
   * @param {string} sessionId - Session ID
   * @returns {SessionInfo | null} Removed session
   */
  remove(sessionId) {
    return this.#sessions.get(sessionId) ?? null;
  }

  /**
   * Get session count.
   * @returns {number} Count
   */
  count() {
    return this.#sessions.size;
  }

  /** @private */
  #evictExpired() {
    const now = Date.now();
    for (const [id, session] of this.#sessions) {
      if (now - session.lastActivity > this.#ttlMs) {
        this.#sessions.delete(id);
      }
    }
  }
}

// ============================================================================
// Exports
// ============================================================================

module.exports = {
  PromptManager,
  SessionManager
};

/**
 * Union registry (R5): client + server + MCP tool definitions with explicit
 * cross-side collision detection.
 */

export type ToolOwner = 'client' | 'server' | 'mcp';

export interface UnionRegistry {
  /** Merged ToolDefinition array for a single `register-tools` call. */
  toDefinitions(): object[];
  ownerOf(name: string): ToolOwner | undefined;
  has(name: string): boolean;
  size(): number;
}

export function createUnionRegistry(options?: {
  localEntries?: Array<{ definition: import('./types').ToolDefinition; handler: import('./types').ToolHandler }>;
  serverDefinitions?: object[];
  mcpDefinitions?: object[];
}): UnionRegistry;

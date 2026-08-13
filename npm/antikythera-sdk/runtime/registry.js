'use strict';

const { WIRE } = require('./types.js');

/**
 * Union registry (R5): the merged set of tool definitions from the client
 * side (local), the server side, and MCP servers. Each tool has exactly one
 * owner in {server, client, mcp}; a cross-side name collision is an explicit
 * error. The merged list is pushed to the runner in a single `register-tools`
 * call (register-tools replaces the whole registry).
 */

/**
 * @param {object} options
 * @param {Array<{definition: object, handler: Function}>} [options.localEntries]
 * @param {Array<object>} [options.serverDefinitions]
 * @param {Array<object>} [options.mcpDefinitions]
 * @returns {{
 *   toDefinitions: () => Array<object>,
 *   ownerOf: (name: string) => string|undefined,
 *   has: (name: string) => boolean,
 *   size: () => number,
 * }}
 */
function createUnionRegistry(options = {}) {
  const byName = new Map();

  function add(definition, owner) {
    if (!definition || typeof definition.name !== 'string' || !definition.name) {
      throw new Error('registry: tool definition requires a name');
    }
    const existing = byName.get(definition.name);
    if (existing && existing.owner !== owner) {
      throw new Error(
        `tool registry collision: tool '${definition.name}' owned by '${existing.owner}' and '${owner}'`,
      );
    }
    byName.set(definition.name, { owner, definition });
  }

  for (const entry of options.localEntries ?? []) {
    add(entry.definition, WIRE.OWNER_CLIENT);
  }
  for (const definition of options.serverDefinitions ?? []) {
    add(definition, WIRE.OWNER_SERVER);
  }
  for (const definition of options.mcpDefinitions ?? []) {
    add(definition, WIRE.OWNER_MCP);
  }

  return {
    toDefinitions() {
      return [...byName.values()].map((entry) => entry.definition);
    },
    ownerOf(name) {
      return byName.get(name)?.owner;
    },
    has(name) {
      return byName.has(name);
    },
    size() {
      return byName.size;
    },
  };
}

module.exports = { createUnionRegistry };

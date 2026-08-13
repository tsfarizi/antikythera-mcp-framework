'use strict';

const { WIRE, joinUrl } = require('./types.js');

/**
 * HTTP transport for the wire protocol. Every request targets paths under
 * `${serverUrl}/antikythera/v1/*` — there is deliberately no other host
 * constant in this module (invariant: the client never fetches an LLM
 * provider directly; all LLM traffic proxies through the server).
 */

/**
 * @param {{ serverUrl: string }} options
 * @returns {{
 *   llmCall: (llmRequest: object, options?: {stream?: boolean}) => Promise<object>,
 *   executeServerTool: (toolCallEvent: object) => Promise<object>,
 *   pullTools: () => Promise<Array<object>>,
 *   postback: (correlationId: string, postbackBody: object) => Promise<null>,
 * }}
 */
function createTransport({ serverUrl }) {
  if (typeof serverUrl !== 'string' || !serverUrl) {
    throw new Error('transport: serverUrl is required');
  }

  /**
   * @param {string} path
   * @param {{ method?: string, body?: object }} [opts]
   * @returns {Promise<object|null>}
   */
  async function request(path, opts = {}) {
    const method = opts.method ?? 'GET';
    const hasBody = opts.body !== undefined;
    let response;
    try {
      response = await fetch(joinUrl(serverUrl, path), {
        method,
        headers: hasBody ? { 'Content-Type': 'application/json' } : {},
        body: hasBody ? JSON.stringify(opts.body) : undefined,
      });
    } catch (err) {
      throw new Error(`transport: ${method} ${path} failed: ${err instanceof Error ? err.message : String(err)}`);
    }
    if (response.status === 204) {
      return null;
    }
    const text = await response.text();
    let json = null;
    if (text) {
      try {
        json = JSON.parse(text);
      } catch {
        json = null;
      }
    }
    if (!response.ok) {
      const message =
        (json && (typeof json.error === 'string' ? json.error : json.message)) ||
        text ||
        `HTTP ${response.status}`;
      throw new Error(String(message));
    }
    return json;
  }

  return {
    /**
     * POST /llm/call — the wire body never carries the stream flag; streaming
     * is signaled by the `?stream=true` query parameter (WIRE_PROTOCOL §2.1).
     * @param {object} llmRequest
     * @param {{stream?: boolean}} [options]
     * @returns {Promise<object>}
     */
    llmCall(llmRequest, options = {}) {
      const path = options.stream ? `${WIRE.LLM_CALL}?stream=true` : WIRE.LLM_CALL;
      return request(path, { method: 'POST', body: llmRequest });
    },
    executeServerTool(toolCallEvent) {
      return request(WIRE.TOOLS_EXECUTE, { method: 'POST', body: toolCallEvent });
    },
    pullTools() {
      return request(WIRE.TOOLS_LIST);
    },
    async postback(correlationId, postbackBody) {
      return request(
        `${WIRE.EVENTS}/${encodeURIComponent(correlationId)}/response`,
        { method: 'POST', body: postbackBody },
      );
    },
  };
}

module.exports = { createTransport };

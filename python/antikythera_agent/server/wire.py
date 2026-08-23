"""Wire-shape builders and parsers for the runtime bridge (E03).

Sumber kebenaran: contracts/shared/wire_protocol.golden.json (shape kanonik)
dan npm/antikythera-sdk/runtime/types.js (perilaku referensi JS yang sudah ada:
nullish-default `??`, strict boolean `=== true`, streaming lewat query param
`?stream=true`, bukan body).
"""

from __future__ import annotations

from typing import Any, Dict, Optional

WIRE = {
    "LLM_CALL": "/antikythera/v1/llm/call",
    "TOOLS_EXECUTE": "/antikythera/v1/tools/execute",
    "TOOLS_LIST": "/antikythera/v1/tools",
    "EVENTS": "/antikythera/v1/events",
    "OWNER_CLIENT": "client",
    "OWNER_SERVER": "server",
    "OWNER_MCP": "mcp",
    "HOOK_PREPARE_TURN": "prepare-turn",
    "HOOK_DECIDE_ACTION": "decide-action",
    "HOOK_HANDLE_TOOL_RESULT": "handle-tool-result",
    "PASSTHROUGH": '{"passthrough": true}',
}


def _nullish_default(value: Any, default: Any) -> Any:
    """Mirror JS `??`: hanya None yang diganti default; falsy lain dipertahankan."""
    return default if value is None else value


def build_llm_request(input_: Dict[str, Any]) -> Dict[str, Any]:
    """Build body request llm/call (snake_case, golden `llm_call_request`).

    Streaming disinyalkan lewat query param `?stream=true` (WIRE_PROTOCOL
    §2.1), bukan body; `metadata_json` hanya untuk metadata provider.
    """
    return {
        "provider": input_.get("provider"),
        "model": input_.get("model"),
        "session_id": input_.get("session_id"),
        "messages_json": _nullish_default(input_.get("messages_json"), ""),
        "force_json": _nullish_default(input_.get("force_json"), False),
        "temperature": input_.get("temperature"),
        "max_tokens": input_.get("max_tokens"),
        "schema_name": input_.get("schema_name"),
        "metadata_json": input_.get("metadata_json"),
    }


def parse_llm_response(body: Any) -> Dict[str, Any]:
    """Parse body response llm/call (golden `llm_call_response`).

    Input non-object memicu exception; `content` yang bukan string dikosongkan
    menjadi '' (mirror `typeof body.content === 'string'`).
    """
    if not isinstance(body, dict):
        raise ValueError("llm/call response is not an object")
    content = body.get("content")
    return {
        "content": content if isinstance(content, str) else "",
        "model": body.get("model"),
        "session_id": body.get("session_id"),
        "message_json": body.get("message_json"),
        "tokens_used": body.get("tokens_used"),
        "finish_reason": body.get("finish_reason"),
        "raw_response_json": body.get("raw_response_json"),
    }


def build_tool_call_event(input_: Dict[str, Any]) -> Dict[str, Any]:
    """Build body tool-call-event (kebab-case, golden `tool_execute_request`)."""
    return {
        "tool-name": input_.get("tool_name"),
        "arguments-json": _nullish_default(input_.get("arguments_json"), "{}"),
        "session-id": input_.get("session_id"),
        "step-id": _nullish_default(input_.get("step_id"), 0),
    }


def parse_tool_execution_result(body: Any) -> Dict[str, Any]:
    """Parse body tool-execution-result (kebab-case, golden `tool_execute_response`).

    `success` hanya True untuk boolean True sejati (mirror `=== true`).
    """
    if not isinstance(body, dict):
        raise ValueError("tools/execute response is not an object")
    return {
        "tool-name": _nullish_default(body.get("tool-name"), ""),
        "success": body.get("success") is True,
        "output-json": _nullish_default(body.get("output-json"), "{}"),
        "error-message": body.get("error-message"),
        "step-id": _nullish_default(body.get("step-id"), 0),
    }


def parse_event_envelope(data: Any) -> Dict[str, Any]:
    """Parse envelope event SSE (snake_case, golden `*_event`).

    `type` diteruskan apa adanya (None bila tidak ada).
    """
    if not isinstance(data, dict):
        raise ValueError("SSE event data is not an object")
    return {
        "type": data.get("type"),
        "correlation_id": data.get("correlation_id"),
        "session_id": data.get("session_id"),
        "client_id": data.get("client_id"),
        "payload": data.get("payload"),
    }


def build_postback(input_: Dict[str, Any]) -> Dict[str, Any]:
    """Build body POST-back (golden `postback_response` / `postback_gate_denial`).

    `ok` hanya True untuk boolean True sejati (mirror `=== true`).
    """
    return {
        "correlation_id": input_.get("correlation_id"),
        "ok": input_.get("ok") is True,
        "payload": input_.get("payload"),
        "error": input_.get("error"),
    }


def wire_to_runner_tool_result(
    wire_result: Dict[str, Any], correlation_id: Optional[str] = None
) -> Dict[str, Any]:
    """Map wire tool-execution-result ke runner ToolResultInput (WIRE_PROTOCOL §6).

    `step_id` di-drop (runner menurunkannya dari session state); `output_json`
    selalu ada; `correlation_id` diteruskan dari pending call bila ada.
    """
    return {
        "tool_name": wire_result.get("tool-name"),
        "success": wire_result.get("success") is True,
        "output_json": _nullish_default(wire_result.get("output-json"), "{}"),
        "error_message": wire_result.get("error-message"),
        "correlation_id": correlation_id,
    }

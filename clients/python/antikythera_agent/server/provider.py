"""LLM provider proxy layer (unit U14): provider abstraction, stub, ollama.

Providers accept a wire `llm-request` (golden `llm_call_request`, built by
`wire.build_llm_request`) and MUST return a wire `llm-response` (golden
`llm_call_response`) with exactly the seven golden keys — no extra fields,
no missing fields (WIRE_PROTOCOL §7).

Implementations (decision D-2 default: stub + ollama minimal):
- `StubProvider`: configured static response (JSON string) for tests/parity.
- `OllamaProvider`: minimal HTTP proxy to Ollama (`POST {base_url}/api/chat`)
  using only stdlib `urllib.request` (decision D3/A-3: zero-dependency base);
  the transport is injectable for tests.

`provider_registry` + `resolve_provider` give downstream units (U21 control,
U23 loop owner) a single lookup point. Default provider: `stub`.

Error contract:
- `ValueError` — invalid wire input (non-object request, undecodable
  `messages_json`, invalid stub configuration).
- `LlmError` — LLM/provider failure (transport/network, invalid provider
  response, missing model). Raw HTTP exceptions are never surfaced as-is.

Streaming is NOT a provider concern (WIRE_PROTOCOL §2.1): the only stream
signal is the transport-level query param `?stream=true` (unit U31), and
`metadata_json` is never read as a stream signal. `metadata_json` is not
forwarded to Ollama because the `/api/chat` API has no metadata field; when
it is forwarded (future providers), it must remain verbatim.
"""

from __future__ import annotations

import json
import urllib.request
from abc import ABC, abstractmethod
from typing import Any, Callable, Dict, Optional

from . import wire

#: Provider name used when no explicit provider is given.
DEFAULT_PROVIDER = "stub"

#: Static body returned by the registry-default stub instance.
_DEFAULT_STUB_RESPONSE = '{"content": "stub response", "finish_reason": "stop"}'

#: HTTP transport signature: (url, body, timeout) -> parsed JSON object.
HttpTransport = Callable[[str, Dict[str, Any], float], Dict[str, Any]]


class LlmError(Exception):
    """Domain error for LLM/provider failures (never a raw HTTP exception)."""


class LlmProvider(ABC):
    """Base contract: `call(request)` -> golden `llm_call_response` shape."""

    def call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        if not isinstance(request, dict):
            raise ValueError("llm provider request must be an object")
        return self._call(request)

    @abstractmethod
    def _call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Provider-specific request handling."""


class StubProvider(LlmProvider):
    """Return a configured static response for tests/parity."""

    def __init__(self, response_json: str = _DEFAULT_STUB_RESPONSE):
        try:
            body = json.loads(response_json)
        except json.JSONDecodeError as exc:
            raise ValueError(f"stub response_json is not valid JSON: {exc}") from exc
        if not isinstance(body, dict):
            raise ValueError("stub response_json must be a JSON object")
        self.response_json = response_json

    def _call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        # Mirror `StubLlmProvider::call` (parity U61): `content` is the whole
        # configured string verbatim — never the value of a `content` field in
        # the configured JSON; model/session_id come from the request.
        parsed = {
            "content": self.response_json,
            "model": request.get("model"),
            "session_id": request.get("session_id"),
            "message_json": None,
            "tokens_used": 4,
            "finish_reason": "stop",
            "raw_response_json": None,
        }
        # Final normalization through the golden parser guarantees the
        # seven-key invariant (no extra fields) while preserving the content.
        return wire.parse_llm_response(parsed)


def _urlopen_json_transport(url: str, body: Dict[str, Any], timeout: float) -> Dict[str, Any]:
    """Default transport: stdlib `urllib.request` POST returning parsed JSON."""
    payload = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


class OllamaProvider(LlmProvider):
    """Minimal HTTP proxy to Ollama's `/api/chat` (zero-dependency base)."""

    def __init__(
        self,
        base_url: str = "http://127.0.0.1:11434",
        model: Optional[str] = None,
        timeout: float = 60.0,
        transport: Optional[HttpTransport] = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.model = model
        self.timeout = timeout
        self._transport = transport or _urlopen_json_transport

    def _call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        url = self.base_url + "/api/chat"
        body = self._build_ollama_body(request)
        try:
            raw_response = self._transport(url, body, self.timeout)
        except Exception as exc:
            # Every failure at the HTTP boundary is a domain error; raw
            # URLError/HTTPError must never reach the caller.
            raise LlmError(f"ollama request to {url} failed: {exc}") from exc
        return self._parse_ollama_response(raw_response, request)

    def _build_ollama_body(self, request: Dict[str, Any]) -> Dict[str, Any]:
        raw_messages = request.get("messages_json") or "[]"
        try:
            messages = json.loads(raw_messages)
        except (TypeError, json.JSONDecodeError) as exc:
            raise ValueError(f"llm-request messages_json is not valid JSON: {exc}") from exc
        if not isinstance(messages, list):
            raise ValueError("llm-request messages_json must decode to a JSON array")
        model = request.get("model") or self.model
        if not model:
            raise LlmError(
                "ollama provider requires a model (request.model or provider model)"
            )
        body = {"model": model, "messages": messages, "stream": False}
        if request.get("force_json") is True:
            body["format"] = "json"
        options = {}
        temperature = request.get("temperature")
        if temperature is not None:
            options["temperature"] = temperature
        max_tokens = request.get("max_tokens")
        if max_tokens is not None:
            options["num_predict"] = max_tokens
        if options:
            body["options"] = options
        return body

    def _parse_ollama_response(self, raw: Any, request: Dict[str, Any]) -> Dict[str, Any]:
        if not isinstance(raw, dict):
            raise LlmError(f"ollama response is not a JSON object: {type(raw).__name__}")
        message = raw.get("message")
        content = ""
        message_json = None
        if isinstance(message, dict):
            message_content = message.get("content")
            content = message_content if isinstance(message_content, str) else ""
            message_json = json.dumps(message)
        eval_count = raw.get("eval_count") or 0
        prompt_eval_count = raw.get("prompt_eval_count") or 0
        tokens_used = None
        if eval_count or prompt_eval_count:
            tokens_used = prompt_eval_count + eval_count
        finish_reason = raw.get("done_reason")
        if finish_reason is None and raw.get("done") is True:
            finish_reason = "stop"
        parsed = {
            "content": content,
            "model": raw.get("model"),
            "session_id": request.get("session_id"),
            "message_json": message_json,
            "tokens_used": tokens_used,
            "finish_reason": finish_reason,
            "raw_response_json": json.dumps(raw),
        }
        # Final normalization through the golden parser guarantees the
        # seven-key invariant even if a future field mapping drifts.
        return wire.parse_llm_response(parsed)


provider_registry: Dict[str, LlmProvider] = {
    "stub": StubProvider(),
    "ollama": OllamaProvider(),
}


def resolve_provider(name: Optional[str] = None) -> LlmProvider:
    """Resolve a registered provider; `None`/omitted -> default `stub`."""
    key = DEFAULT_PROVIDER if name is None else name
    if key not in provider_registry:
        raise KeyError(f"unknown LLM provider: {key!r}; known: {sorted(provider_registry)}")
    return provider_registry[key]

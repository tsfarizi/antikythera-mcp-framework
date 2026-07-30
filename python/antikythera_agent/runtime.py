"""WASM runtime for executing antikythera components."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Callable, Optional

_WASM_PATH = Path(__file__).parent / "antikythera.wasm"


class WasmRuntimeError(Exception):
    """Error from WASM runtime."""


class WasmRuntime:
    """WASM runtime for executing antikythera components.

    Uses wasmtime to run WASI components with host-imports bridge.

    Example:
        >>> runtime = WasmRuntime()
        >>> result = runtime.call("init", '{"max_steps": 10}')
        >>> print(result)
    """

    def __init__(
        self,
        wasi_stdout: Optional[Callable[[str], None]] = None,
        wasi_stderr: Optional[Callable[[str], None]] = None,
    ) -> None:
        """Initialize WASM runtime.

        Args:
            wasi_stdout: Callback for WASI stdout output.
            wasi_stderr: Callback for WASI stderr output.
        """
        self._wasm_path = _WASM_PATH
        self._wasi_stdout = wasi_stdout or (lambda s: None)
        self._wasi_stderr = wasi_stderr or (lambda s: None)
        self._store: Any = None
        self._instance: Any = None
        self._module: Any = None
        self._linker: Any = None
        self._initialized = False

    def _ensure_initialized(self) -> None:
        """Lazy-initialize wasmtime components."""
        if self._initialized:
            return

        if not self._wasm_path.exists():
            raise WasmRuntimeError(
                f"WASM binary not found at {self._wasm_path}. "
                "Install with: pip install antikythera-agent"
            )

        try:
            import wasmtime
        except ImportError:
            raise WasmRuntimeError(
                "wasmtime is required for WASM execution. "
                "Install with: pip install wasmtime"
            )

        self._wasmtime = wasmtime

        engine = wasmtime.Engine()
        self._store = wasmtime.Store(engine)
        self._module = wasmtime.Module.from_file(engine, str(self._wasm_path))

        self._linker = wasmtime.Linker(engine)
        self._linker.define_wasi()

        self._instance = self._linker.instantiate(self._store, self._module)
        self._initialized = True

    def call(self, func_name: str, args: str) -> str:
        """Call an exported WASM function.

        Args:
            func_name: Name of the exported function.
            args: JSON string argument.

        Returns:
            JSON string result.

        Raises:
            WasmRuntimeError: If call fails.
        """
        self._ensure_initialized()

        try:
            memory = self._instance.exports(self._store)["memory"]
            alloc = self._instance.exports(self._store)["cabi_realloc"]

            args_bytes = args.encode("utf-8")
            ptr = alloc(self._store, 0, 0, 1, len(args_bytes))

            memory.data_ptr(self._store)[ptr:ptr + len(args_bytes)] = args_bytes

            func = self._instance.exports(self._store).get(func_name)
            if func is None:
                raise WasmRuntimeError(f"Export '{func_name}' not found in WASM")

            result_ptr = func(self._store, ptr, len(args_bytes))

            result_len = 4096
            result_bytes = bytes(memory.data_ptr(self._store)[result_ptr:result_ptr + result_len])
            result_bytes = result_bytes.split(b"\x00")[0]

            return result_bytes.decode("utf-8")

        except WasmRuntimeError:
            raise
        except Exception as e:
            raise WasmRuntimeError(f"WASM call '{func_name}' failed: {e}") from e

    def call_checked(self, func_name: str, args: str) -> dict[str, Any]:
        """Call WASM function and parse JSON result.

        Args:
            func_name: Name of the exported function.
            args: JSON string argument.

        Returns:
            Parsed result dictionary.

        Raises:
            WasmRuntimeError: If call or parsing fails.
        """
        result_json = self.call(func_name, args)
        try:
            return json.loads(result_json)
        except json.JSONDecodeError as e:
            raise WasmRuntimeError(
                f"Invalid JSON response from '{func_name}': {e}"
            ) from e

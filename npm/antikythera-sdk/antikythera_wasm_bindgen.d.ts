/* tslint:disable */
/* eslint-disable */

export function append_llm_chunk(session_id: string, chunk: string, correlation_id?: string | null): boolean;

export function commit_llm_response(prepared_turn_json: string, llm_response_json: string): string;

export function commit_llm_stream(prepared_turn_json: string): string;

export function drain_events(session_id: string): string;

export function get_slo_snapshot(session_id: string): string;

export function get_state(session_id: string): string;

export function get_telemetry_snapshot(session_id: string): string;

export function get_tools_prompt(): string;

export function init(config_json: string): string;

export function prepare_user_turn(request_json: string): string;

export function process_llm_response_for_session(session_id: string, llm_response_json: string): string;

export function process_tool_result_for_session(session_id: string, tool_result_json: string): string;

export function register_tools(tools_json: string): number;

export function reset_session(session_id: string): boolean;

export function set_context_policy(policy_json: string): boolean;

export function sweep_idle_sessions(now_unix_ms?: bigint | null): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly init: (a: number, b: number) => [number, number, number, number];
    readonly prepare_user_turn: (a: number, b: number) => [number, number, number, number];
    readonly commit_llm_response: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly commit_llm_stream: (a: number, b: number) => [number, number, number, number];
    readonly process_llm_response_for_session: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly process_tool_result_for_session: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly append_llm_chunk: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly drain_events: (a: number, b: number) => [number, number, number, number];
    readonly get_state: (a: number, b: number) => [number, number, number, number];
    readonly reset_session: (a: number, b: number) => [number, number, number];
    readonly sweep_idle_sessions: (a: number, b: bigint) => [number, number, number];
    readonly register_tools: (a: number, b: number) => [number, number, number];
    readonly get_tools_prompt: () => [number, number, number, number];
    readonly set_context_policy: (a: number, b: number) => [number, number, number];
    readonly get_telemetry_snapshot: (a: number, b: number) => [number, number, number, number];
    readonly get_slo_snapshot: (a: number, b: number) => [number, number, number, number];
    readonly mcp_free_string: (a: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

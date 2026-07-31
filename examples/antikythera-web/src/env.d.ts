/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<object, object, unknown>;
  export default component;
}

declare module "antikythera-agent/antikythera_wasm_bindgen" {
  export function append_llm_chunk(session_id: string, chunk: string, correlation_id?: string | null): boolean;
  export function commit_llm_response(prepared_turn_json: string, llm_response_json: string): string;
  export function commit_llm_stream(prepared_turn_json: string): string;
  export function drain_events(session_id: string): string;
  export function get_state(session_id: string): string;
  export function get_tools_prompt(): string;
  export function init(config_json: string): string;
  export function prepare_user_turn(request_json: string): string;
  export function process_llm_response_for_session(session_id: string, llm_response_json: string): string;
  export function process_tool_result_for_session(session_id: string, tool_result_json: string): string;
  export function register_tools(tools_json: string): number;
  export function reset_session(session_id: string): boolean;
  export function set_context_policy(policy_json: string): boolean;
  export default function __wbg_init(module_or_path?: unknown): Promise<unknown>;
}

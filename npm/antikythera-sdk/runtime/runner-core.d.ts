/**
 * Client-core runtime: loads the jco-transpiled component and owns the tool
 * loop (K1).
 */

import {
  AgentRuntimeOptionsBase,
  ClientCoreRuntime,
  LlmOptions,
  RunnerNamespace,
} from './index';

export interface ClientCoreOptions extends AgentRuntimeOptionsBase {
  core?: 'client';
  llm?: LlmOptions;
  /**
   * Absolute URL of the jco bundle directory; the entry file is resolved from
   * the server manifest (`GET /antikythera/v1/component/manifest`, WIRE_PROTOCOL
   * §2.6). Omit to keep the bundled component (default, decision D5).
   */
  componentBase?: string;
  /**
   * Directly injected runner namespace; bypasses the component import
   * entirely (decision D5). Takes precedence over `componentBase`.
   */
  runner?: RunnerNamespace;
}

export function createClientCoreRuntime(options: ClientCoreOptions): Promise<ClientCoreRuntime>;

/** Options for `loadRunnerModule` (decision D5). */
export interface LoadRunnerOptions {
  /** Server base URL used for the manifest fetch; required when `componentBase` is set. */
  serverUrl?: string;
  /** Absolute URL of the jco bundle directory; the entry file is resolved from the server manifest. */
  componentBase?: string;
  /** Directly injected runner namespace; bypasses the component import. */
  runner?: RunnerNamespace;
}

/**
 * Load the runner namespace: injected `runner` > `componentBase`
 * (manifest-resolved entry) > bundled path (default). Cached per URL.
 */
export function loadRunnerModule(options?: LoadRunnerOptions): Promise<RunnerNamespace>;

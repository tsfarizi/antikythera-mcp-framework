/**
 * Client-side permission gate (default-deny). Denials surface as errors whose
 * message starts with `permission:`.
 */

export interface PermissionPolicy {
  allow?: string[];
}

export interface PolicyGate {
  /** Throw `permission: tool '<name>' not in allowlist` when not allowed. */
  check(toolName: string): void;
  allows(toolName: string): boolean;
}

export function createPolicyGate(policy?: PermissionPolicy): PolicyGate;

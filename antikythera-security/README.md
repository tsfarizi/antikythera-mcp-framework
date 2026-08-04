# antikythera-security

Security implementations for the Antikythera Agent SDK: input validation, rate limiting, and secrets management.

## Features

- **InputValidator** — size, message length, keyword blocking, HTML sanitization, JSON structure validation, URL pattern matching
- **RateLimiter** — sliding-window rate limiting per minute/hour/day with concurrent session tracking and background cleanup
- **SecretManager** — versioned secret storage with rotation support and in-memory backend
- **SecurityFacade** — single entry point that constructs all subsystems from one `SecurityConfig`

## Feature Flags

- `validation` — enables `InputValidator` (requires `regex`)
- `rate-limit` — enables `RateLimiter`
- `memory` — enables `SecretManager` with in-memory backend
- `full` — enables all of the above

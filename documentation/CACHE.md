# Cache

This document describes the active cache model for configuration data.

## Cache Lifecycle

```mermaid
flowchart TD
    Source[Configuration source] --> Encode[TOML encode]
    Encode --> File[Text cache artifact]
    File --> Decode[TOML decode]
    Decode --> Runtime[Runtime configuration use]
```

## Current Behavior

- Configuration artifacts are stored as TOML text for human readability.
- Cache decoding is tied to active schema compatibility checks.
- Runtime uses cache artifacts only when integrity checks pass.

## Operational Guidance

- Rebuild cache after schema-affecting updates.
- Keep import/export procedures aligned with cache format expectations.

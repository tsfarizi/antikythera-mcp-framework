# Config

This document describes the TOML-based configuration flow used by the current documentation set and related SDK surfaces.

## Overview

The repository documents two configuration stories:

1. The lightweight CLI config flow built around `app.toml`
2. Broader SDK and core configuration helpers that use serialized configuration data

## Configuration model

```mermaid
flowchart TD
    INPUT[Config source] --> SERIALIZE[TOML serialization]
    SERIALIZE --> FILE[Text file on disk]
    FILE --> LOAD[Load config]
    LOAD --> USE[CLI, SDK, or runtime usage]
```

## Why TOML

| Property | Benefit |
|:---------|:--------|
| Text format | Human-readable and editable in any text editor |
| Standard format | Widely supported across languages and tools |
| Typed serialization | Matches Rust data structures directly |
| Easy export/import | Works well for backup and transfer flows |

## Main points

- Configuration is treated as structured data first, not hand-edited prose.
- Export and inspection can still be done through JSON-based helper commands or APIs.
- Secrets should remain outside the config file when a dedicated secret mechanism is available.

## Related documents

- [`CLI.md`](CLI.md) for the current CLI config workflow
- [`IMPORT_EXPORT.md`](IMPORT_EXPORT.md) for backup and restore flows
- [`CACHE.md`](CACHE.md) for cache-specific notes

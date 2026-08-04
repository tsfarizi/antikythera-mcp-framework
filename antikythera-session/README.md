# antikythera-session

Session management and conversational data models for the Antikythera Agent SDK.

## Features

- `Message`, `MessageRole`, `MessagePart` — typed chat message model with custom serde
- `Session` — full session entity with messages, tokens, tools, steps
- `SessionManager` — thread-safe session manager (`Arc<RwLock<HashMap>>`) supporting concurrent operations
- `SessionExport` / `BatchExport` — versioned session import/export with JSON format

## Session Lifecycle

```
new() → create_session() → add_message() → get_chat_history() → delete_session()
       → search_sessions() → export / import
```

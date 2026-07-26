# Antikythera CLI Learnings

## 2026-07-16: ChatApp God Object Refactoring

### Summary
Extracted logical field groups from the 30+ field `ChatApp` struct into three dedicated sub-structs: `StreamingState`, `HistoryState`, and `ScrollState`.

### Changes Made
- **`app.rs`**: Added `StreamingState`, `HistoryState`, `ScrollState` structs. Replaced individual fields with `streaming: StreamingState`, `history: HistoryState`, `scroll: ScrollState`.
- **`event_loop.rs`**: Updated `stream_rx` and `streaming_content` accesses to use `app.streaming.*`.
- **`handlers/commands.rs`**: Updated `history_store` to `app.history.store`, `current_history_session` to `app.history.current_session`, `history.open_and_refresh_with` to `app.history.browser.open_and_refresh_with`.
- **`handlers/history_handler.rs`**: Updated all `app.history.*` to `app.history.browser.*` and `app.history_store.*` to `app.history.store.*`.
- **`handlers/submit.rs`**: Updated `conversation_scroll` to `app.scroll.conversation`, `current_history_session` to `app.history.current_session`, streaming fields to `app.streaming.*`.
- **`event_loop/result_handler.rs`**: Updated `current_history_session` to `app.history.current_session`, `history_store` to `app.history.store`, `conversation_scroll` to `app.scroll.conversation`.
- **`event_loop/key_dispatch.rs`**: Updated `history.open` to `app.history.browser.open`, `history_store` to `app.history.store`, scroll fields to `app.scroll.*`.
- **`render/conversation.rs`**: Updated `streaming_content` to `app.streaming.content`, `conversation_scroll` to `app.scroll.conversation`.
- **`render/history_overlay.rs`**: Updated all `app.history.*` to `app.history.browser.*`.
- **`render/mod.rs`**: Updated `log_scroll` to `app.scroll.log`, `history.open` to `app.history.browser.open`.

### Key Observations
- The `HistoryBrowser` type was already a separate struct in `types.rs`, so `HistoryState` wraps it as `browser` field plus `store` and `current_session`.
- Pre-existing compilation error in `antikythera-core` (`AgentOptions` missing `event_sender` field) was fixed as part of this change.
- No changes to public API - all methods on `ChatApp` remain the same.
- All 88 field access sites were updated successfully.

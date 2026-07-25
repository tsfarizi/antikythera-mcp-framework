//! History browser keyboard handler.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::presentation::tui::app::ChatApp;
use crate::presentation::tui::event_loop::KeyAction;

pub(crate) fn handle_history_key(key: KeyEvent, app: &mut ChatApp) -> KeyAction {
    // Rename mode intercepts all printable input.
    if app.history.browser.rename_mode {
        match key.code {
            KeyCode::Esc => {
                app.history.browser.rename_mode = false;
                app.history.browser.rename_buffer.clear();
            }
            KeyCode::Enter => {
                let new_title = app.history.browser.rename_buffer.trim().to_string();
                if !new_title.is_empty()
                    && let Some(id) = app
                        .history
                        .browser
                        .sessions
                        .get(app.history.browser.cursor)
                        .map(|s| s.id.clone())
                    && app.history.store.rename_session(&id, new_title).is_ok()
                {
                    app.history.browser.sessions = app.history.store.list_sessions();
                }
                app.history.browser.rename_mode = false;
                app.history.browser.rename_buffer.clear();
            }
            KeyCode::Backspace => {
                app.history.browser.rename_buffer.pop();
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.history.browser.rename_buffer.push(ch);
            }
            _ => {}
        }
        return KeyAction::None;
    }

    // Detail view — show full conversation, allow scrolling.
    if app.history.browser.detail.is_some() {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => {
                app.history.browser.detail = None;
                app.history.browser.detail_scroll = 0;
            }
            KeyCode::Up => {
                app.history.browser.detail_scroll = app.history.browser.detail_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                app.history.browser.detail_scroll += 1;
            }
            _ => {}
        }
        return KeyAction::None;
    }

    // List view — navigate, open, delete, rename.
    match key.code {
        KeyCode::Esc | KeyCode::F(3) => {
            app.history.browser.open = false;
            app.status = "Siap.".to_string();
        }
        KeyCode::Up => {
            app.history.browser.cursor = app.history.browser.cursor.saturating_sub(1);
        }
        KeyCode::Down => {
            let max = app.history.browser.sessions.len().saturating_sub(1);
            if app.history.browser.cursor < max {
                app.history.browser.cursor += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(id) = app
                .history
                .browser
                .sessions
                .get(app.history.browser.cursor)
                .map(|s| s.id.clone())
            {
                app.history.browser.detail = app.history.store.load_session(&id);
                app.history.browser.detail_scroll = 0;
            }
        }
        KeyCode::Char('d') => {
            if let Some(id) = app
                .history
                .browser
                .sessions
                .get(app.history.browser.cursor)
                .map(|s| s.id.clone())
            {
                let _ = app.history.store.delete_session(&id);
                app.history.browser.sessions = app.history.store.list_sessions();
                let max = app.history.browser.sessions.len().saturating_sub(1);
                app.history.browser.cursor = app.history.browser.cursor.min(max);
            }
        }
        KeyCode::Char('r') => {
            let buf = app
                .history
                .browser
                .sessions
                .get(app.history.browser.cursor)
                .map(|s| s.title.clone())
                .unwrap_or_default();
            app.history.browser.rename_buffer = buf;
            app.history.browser.rename_mode = true;
        }
        _ => {}
    }
    KeyAction::None
}

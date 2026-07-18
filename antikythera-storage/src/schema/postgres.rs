//! PostgreSQL schema definitions for the sessions table.

/// Returns the SQL for creating the sessions table.
pub fn sessions_schema() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS sessions (
        id VARCHAR(36) PRIMARY KEY,
        data JSONB NOT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NOW()
    );
    "#
}

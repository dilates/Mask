use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use directories::ProjectDirs;

pub fn data_dir() -> PathBuf {
    ProjectDirs::from("com", "mask", "Mask")
        .map(|p| p.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn open() -> Result<Connection> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).ok();
    let conn = Connection::open(dir.join("mask.db"))?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS personas (
            id             TEXT PRIMARY KEY,
            name           TEXT NOT NULL,
            description    TEXT NOT NULL DEFAULT '',
            color          TEXT NOT NULL DEFAULT '#6366f1',
            proxy_type     TEXT,
            proxy_host     TEXT,
            proxy_port     INTEGER,
            proxy_user     TEXT,
            proxy_pass     TEXT,
            vpn_config     TEXT,
            timezone       TEXT NOT NULL DEFAULT 'UTC',
            locale         TEXT NOT NULL DEFAULT 'en-US',
            user_agent     TEXT NOT NULL DEFAULT '',
            notes          TEXT NOT NULL DEFAULT '',
            created_at     TEXT NOT NULL,
            updated_at     TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS audit_log (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            persona_id   TEXT,
            persona_name TEXT,
            event_type   TEXT NOT NULL,
            detail       TEXT,
            result       TEXT,
            ts           TEXT NOT NULL
        );
    ")?;

    // Additive migrations — safe to re-run (ALTER TABLE IF NOT EXISTS equivalent via IGNORE)
    for stmt in [
        "ALTER TABLE personas ADD COLUMN browser_type TEXT NOT NULL DEFAULT 'firefox'",
        "ALTER TABLE personas ADD COLUMN container_mode TEXT NOT NULL DEFAULT 'none'",
        "ALTER TABLE personas ADD COLUMN wg_interface TEXT",
    ] {
        // SQLite returns error 1 ("duplicate column") when column already exists — ignore it
        let _ = conn.execute_batch(stmt);
    }

    Ok(())
}

pub fn log_event(
    conn: &Connection,
    persona_id: Option<&str>,
    persona_name: Option<&str>,
    event_type: &str,
    detail: Option<&str>,
    result: Option<&str>,
) -> Result<()> {
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO audit_log (persona_id, persona_name, event_type, detail, result, ts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![persona_id, persona_name, event_type, detail, result, ts],
    )?;
    Ok(())
}

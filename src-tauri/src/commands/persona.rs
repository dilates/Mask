use crate::{db, models::*};
use rusqlite::params;
use uuid::Uuid;
use chrono::Utc;

#[tauri::command]
pub fn list_personas() -> Result<Vec<Persona>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id,name,description,color,proxy_type,proxy_host,proxy_port,proxy_user,proxy_pass,vpn_config,timezone,locale,user_agent,notes,created_at,updated_at FROM personas ORDER BY created_at")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(Persona {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            color: row.get(3)?,
            proxy_type: row.get(4)?,
            proxy_host: row.get(5)?,
            proxy_port: row.get(6)?,
            proxy_user: row.get(7)?,
            proxy_pass: row.get(8)?,
            vpn_config: row.get(9)?,
            timezone: row.get(10)?,
            locale: row.get(11)?,
            user_agent: row.get(12)?,
            notes: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_persona(input: CreatePersona) -> Result<Persona, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let p = Persona {
        id: id.clone(),
        name: input.name,
        description: input.description.unwrap_or_default(),
        color: input.color.unwrap_or_else(|| "#6366f1".into()),
        proxy_type: input.proxy_type,
        proxy_host: input.proxy_host,
        proxy_port: input.proxy_port,
        proxy_user: input.proxy_user,
        proxy_pass: input.proxy_pass,
        vpn_config: input.vpn_config,
        timezone: input.timezone.unwrap_or_else(|| "UTC".into()),
        locale: input.locale.unwrap_or_else(|| "en-US".into()),
        user_agent: input.user_agent.unwrap_or_default(),
        notes: input.notes.unwrap_or_default(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    conn.execute(
        "INSERT INTO personas VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        params![
            p.id, p.name, p.description, p.color,
            p.proxy_type, p.proxy_host, p.proxy_port,
            p.proxy_user, p.proxy_pass, p.vpn_config,
            p.timezone, p.locale, p.user_agent, p.notes,
            p.created_at, p.updated_at
        ],
    ).map_err(|e| e.to_string())?;

    db::log_event(&conn, Some(&id), Some(&p.name), "persona_created", None, Some("ok")).ok();
    Ok(p)
}

#[tauri::command]
pub fn update_persona(input: UpdatePersona) -> Result<Persona, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    // Load current
    let mut stmt = conn.prepare(
        "SELECT id,name,description,color,proxy_type,proxy_host,proxy_port,proxy_user,proxy_pass,vpn_config,timezone,locale,user_agent,notes,created_at,updated_at FROM personas WHERE id=?1"
    ).map_err(|e| e.to_string())?;

    let mut p: Persona = stmt.query_row([&input.id], |row| {
        Ok(Persona {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            color: row.get(3)?,
            proxy_type: row.get(4)?,
            proxy_host: row.get(5)?,
            proxy_port: row.get(6)?,
            proxy_user: row.get(7)?,
            proxy_pass: row.get(8)?,
            vpn_config: row.get(9)?,
            timezone: row.get(10)?,
            locale: row.get(11)?,
            user_agent: row.get(12)?,
            notes: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
        })
    }).map_err(|e| format!("Persona not found: {e}"))?;

    if let Some(v) = input.name { p.name = v; }
    if let Some(v) = input.description { p.description = v; }
    if let Some(v) = input.color { p.color = v; }
    if let Some(v) = input.proxy_type { p.proxy_type = Some(v); }
    if let Some(v) = input.proxy_host { p.proxy_host = Some(v); }
    if let Some(v) = input.proxy_port { p.proxy_port = Some(v); }
    if let Some(v) = input.proxy_user { p.proxy_user = Some(v); }
    if let Some(v) = input.proxy_pass { p.proxy_pass = Some(v); }
    if let Some(v) = input.vpn_config { p.vpn_config = Some(v); }
    if let Some(v) = input.timezone { p.timezone = v; }
    if let Some(v) = input.locale { p.locale = v; }
    if let Some(v) = input.user_agent { p.user_agent = v; }
    if let Some(v) = input.notes { p.notes = v; }
    p.updated_at = now;

    conn.execute(
        "UPDATE personas SET name=?2,description=?3,color=?4,proxy_type=?5,proxy_host=?6,proxy_port=?7,proxy_user=?8,proxy_pass=?9,vpn_config=?10,timezone=?11,locale=?12,user_agent=?13,notes=?14,updated_at=?15 WHERE id=?1",
        params![
            p.id, p.name, p.description, p.color,
            p.proxy_type, p.proxy_host, p.proxy_port,
            p.proxy_user, p.proxy_pass, p.vpn_config,
            p.timezone, p.locale, p.user_agent, p.notes,
            p.updated_at
        ],
    ).map_err(|e| e.to_string())?;

    db::log_event(&conn, Some(&p.id), Some(&p.name), "persona_updated", None, Some("ok")).ok();
    Ok(p)
}

#[tauri::command]
pub fn delete_persona(id: String) -> Result<(), String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    // Get name for log before deleting
    let name: Option<String> = conn.query_row(
        "SELECT name FROM personas WHERE id=?1", [&id],
        |row| row.get(0)
    ).ok();

    conn.execute("DELETE FROM personas WHERE id=?1", [&id])
        .map_err(|e| e.to_string())?;

    db::log_event(&conn, Some(&id), name.as_deref(), "persona_deleted", None, Some("ok")).ok();
    Ok(())
}

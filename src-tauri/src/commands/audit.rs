use crate::{db, models::AuditEntry};

#[tauri::command]
pub fn get_audit_log(limit: Option<i64>) -> Result<Vec<AuditEntry>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let n = limit.unwrap_or(200);

    let mut stmt = conn.prepare(
        "SELECT id,persona_id,persona_name,event_type,detail,result,ts FROM audit_log ORDER BY ts DESC LIMIT ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([n], |row| {
        Ok(AuditEntry {
            id: row.get(0)?,
            persona_id: row.get(1)?,
            persona_name: row.get(2)?,
            event_type: row.get(3)?,
            detail: row.get(4)?,
            result: row.get(5)?,
            ts: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_data_dir() -> String {
    db::data_dir().to_string_lossy().into_owned()
}

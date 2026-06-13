use crate::{db, models::*};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use chrono::Utc;

const PBKDF2_ITER: u32 = 200_000;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITER, &mut key);
    key
}

/// Export one or all personas to an encrypted JSON blob (base64-encoded).
/// Format: base64( salt[16] || nonce[12] || aes256gcm_ciphertext )
#[tauri::command]
pub fn export_personas(
    persona_ids: Vec<String>,
    password: String,
) -> Result<String, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id,name,description,color,proxy_type,proxy_host,proxy_port,\
         proxy_user,proxy_pass,vpn_config,timezone,locale,user_agent,notes,\
         browser_type,container_mode,wg_interface,created_at,updated_at FROM personas"
    ).map_err(|e| e.to_string())?;

    let all: Vec<Persona> = stmt.query_map([], |row| {
        Ok(Persona {
            id:             row.get(0)?,
            name:           row.get(1)?,
            description:    row.get(2)?,
            color:          row.get(3)?,
            proxy_type:     row.get(4)?,
            proxy_host:     row.get(5)?,
            proxy_port:     row.get(6)?,
            proxy_user:     row.get(7)?,
            proxy_pass:     row.get(8)?,
            vpn_config:     row.get(9)?,
            timezone:       row.get(10)?,
            locale:         row.get(11)?,
            user_agent:     row.get(12)?,
            notes:          row.get(13)?,
            browser_type:   row.get::<_, Option<String>>(14)?.unwrap_or_else(|| "firefox".into()),
            container_mode: row.get::<_, Option<String>>(15)?.unwrap_or_else(|| "none".into()),
            wg_interface:   row.get(16)?,
            created_at:     row.get(17)?,
            updated_at:     row.get(18)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let personas: Vec<Persona> = if persona_ids.is_empty() {
        all
    } else {
        all.into_iter().filter(|p| persona_ids.contains(&p.id)).collect()
    };

    if personas.is_empty() {
        return Err("No personas matched the given IDs".into());
    }

    let payload = ExportPayload {
        version: 1,
        exported_at: Utc::now().to_rfc3339(),
        personas,
    };
    let plaintext = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;

    // Generate random salt and nonce
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let key_bytes = derive_key(&password, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_slice())
        .map_err(|e| format!("Encryption failed: {e}"))?;

    let mut blob = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(&salt);
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ciphertext);

    db::log_event(&conn, None, None, "export", Some(&format!("{} personas", payload.personas.len())), Some("ok")).ok();
    Ok(B64.encode(&blob))
}

/// Decrypt an exported blob and import the personas (skipping duplicates by ID).
#[tauri::command]
pub fn import_personas(blob: String, password: String) -> Result<usize, String> {
    let raw = B64.decode(blob.trim()).map_err(|e| format!("Invalid base64: {e}"))?;

    if raw.len() < SALT_LEN + NONCE_LEN + 16 {
        return Err("Data too short".into());
    }

    let salt = &raw[..SALT_LEN];
    let nonce_bytes = &raw[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &raw[SALT_LEN + NONCE_LEN..];

    let key_bytes = derive_key(&password, salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed — wrong password or corrupted data")?;

    let payload: ExportPayload = serde_json::from_slice(&plaintext)
        .map_err(|e| format!("Invalid payload: {e}"))?;

    if payload.version != 1 {
        return Err(format!("Unsupported export version {}", payload.version));
    }

    let conn = db::open().map_err(|e| e.to_string())?;
    let mut imported = 0;

    for p in &payload.personas {
        // INSERT OR IGNORE skips existing IDs
        let result = conn.execute(
            "INSERT OR IGNORE INTO personas VALUES \
             (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
            rusqlite::params![
                p.id, p.name, p.description, p.color,
                p.proxy_type, p.proxy_host, p.proxy_port,
                p.proxy_user, p.proxy_pass, p.vpn_config,
                p.timezone, p.locale, p.user_agent, p.notes,
                p.browser_type, p.container_mode, p.wg_interface,
                p.created_at, p.updated_at
            ],
        );
        if result.map(|n| n > 0).unwrap_or(false) {
            imported += 1;
        }
    }

    db::log_event(&conn, None, None, "import",
        Some(&format!("{}/{} personas imported", imported, payload.personas.len())),
        Some("ok")).ok();
    Ok(imported)
}

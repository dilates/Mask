use crate::{db, models::Persona};
use std::path::PathBuf;
use std::process::Command;
use rusqlite::params;

fn firefox_profile_dir(persona_id: &str) -> PathBuf {
    db::data_dir().join("profiles").join(persona_id)
}

fn user_js_content(persona: &Persona) -> String {
    let ua = if persona.user_agent.is_empty() {
        "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0".to_string()
    } else {
        persona.user_agent.clone()
    };

    format!(
        r#"// Mask-generated user.js — do not edit manually
user_pref("general.useragent.override", "{ua}");
user_pref("privacy.resistFingerprinting", true);
user_pref("privacy.resistFingerprinting.block_mozAddonManager", true);
user_pref("media.peerconnection.enabled", false);
user_pref("media.peerconnection.ice.default_address_only", true);
user_pref("media.peerconnection.ice.no_host", true);
user_pref("webgl.disabled", true);
user_pref("dom.battery.enabled", false);
user_pref("geo.enabled", false);
user_pref("network.dns.disablePrefetch", true);
user_pref("network.prefetch-next", false);
user_pref("browser.cache.disk.enable", false);
user_pref("browser.sessionstore.privacy_level", 2);
user_pref("privacy.clearOnShutdown.cookies", false);
user_pref("intl.accept_languages", "{locale}");
"#,
        ua = ua.replace('"', "\\\""),
        locale = persona.locale.replace('"', "\\\""),
    )
}

fn proxy_prefs(persona: &Persona) -> String {
    match persona.proxy_type.as_deref() {
        Some("socks5") => {
            let host = persona.proxy_host.as_deref().unwrap_or("127.0.0.1");
            let port = persona.proxy_port.unwrap_or(1080);
            format!(
                r#"user_pref("network.proxy.type", 1);
user_pref("network.proxy.socks", "{host}");
user_pref("network.proxy.socks_port", {port});
user_pref("network.proxy.socks_version", 5);
user_pref("network.proxy.socks_remote_dns", true);
"#
            )
        }
        Some("http") => {
            let host = persona.proxy_host.as_deref().unwrap_or("127.0.0.1");
            let port = persona.proxy_port.unwrap_or(8080);
            format!(
                r#"user_pref("network.proxy.type", 1);
user_pref("network.proxy.http", "{host}");
user_pref("network.proxy.http_port", {port});
user_pref("network.proxy.ssl", "{host}");
user_pref("network.proxy.ssl_port", {port});
"#
            )
        }
        _ => String::from(r#"user_pref("network.proxy.type", 0);"#) + "\n",
    }
}

fn ensure_profile(persona: &Persona) -> Result<PathBuf, String> {
    let dir = firefox_profile_dir(&persona.id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Write combined user.js
    let content = user_js_content(persona) + &proxy_prefs(persona);
    std::fs::write(dir.join("user.js"), content).map_err(|e| e.to_string())?;

    Ok(dir)
}

#[tauri::command]
pub fn launch_browser(persona_id: String) -> Result<String, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let persona: Persona = conn.query_row(
        "SELECT id,name,description,color,proxy_type,proxy_host,proxy_port,proxy_user,proxy_pass,vpn_config,timezone,locale,user_agent,notes,created_at,updated_at FROM personas WHERE id=?1",
        [&persona_id],
        |row| Ok(Persona {
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
        }),
    ).map_err(|e| format!("Persona not found: {e}"))?;

    let profile_dir = ensure_profile(&persona)?;

    // Find firefox binary
    let firefox = ["firefox", "firefox-esr", "firefox-bin"]
        .iter()
        .find(|&&bin| which(bin).is_some())
        .copied()
        .ok_or("Firefox not found in PATH")?;

    let mut cmd = Command::new(firefox);
    cmd.arg("--profile")
       .arg(profile_dir.to_str().unwrap())
       .arg("--new-instance");

    // Timezone via env
    if persona.timezone != "UTC" {
        cmd.env("TZ", &persona.timezone);
    }

    cmd.spawn().map_err(|e| format!("Failed to launch Firefox: {e}"))?;

    db::log_event(&conn, Some(&persona.id), Some(&persona.name), "browser_launched", None, Some("ok")).ok();
    Ok(format!("Launched Firefox for persona '{}'", persona.name))
}

fn which(bin: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let full = dir.join(bin);
            if full.is_file() { Some(full) } else { None }
        })
    })
}

#[tauri::command]
pub fn get_profile_path(persona_id: String) -> String {
    firefox_profile_dir(&persona_id)
        .to_string_lossy()
        .into_owned()
}

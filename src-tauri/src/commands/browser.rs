use crate::{db, models::Persona};
use std::path::PathBuf;
use std::process::Command;

// ── Profile directories ────────────────────────────────────────────────────

fn profile_dir(persona_id: &str) -> PathBuf {
    db::data_dir().join("profiles").join(persona_id)
}

fn extension_src_dir() -> PathBuf {
    db::data_dir().join("extension")
}

// ── Firefox ────────────────────────────────────────────────────────────────

fn firefox_user_js(persona: &Persona) -> String {
    let ua = if persona.user_agent.is_empty() {
        "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0".to_string()
    } else {
        persona.user_agent.clone()
    };

    let proxy_prefs = match persona.proxy_type.as_deref() {
        Some("socks5") => {
            let h = persona.proxy_host.as_deref().unwrap_or("127.0.0.1");
            let p = persona.proxy_port.unwrap_or(1080);
            format!(
                "user_pref(\"network.proxy.type\", 1);\n\
                 user_pref(\"network.proxy.socks\", \"{h}\");\n\
                 user_pref(\"network.proxy.socks_port\", {p});\n\
                 user_pref(\"network.proxy.socks_version\", 5);\n\
                 user_pref(\"network.proxy.socks_remote_dns\", true);\n"
            )
        }
        Some("http") => {
            let h = persona.proxy_host.as_deref().unwrap_or("127.0.0.1");
            let p = persona.proxy_port.unwrap_or(8080);
            format!(
                "user_pref(\"network.proxy.type\", 1);\n\
                 user_pref(\"network.proxy.http\", \"{h}\");\n\
                 user_pref(\"network.proxy.http_port\", {p});\n\
                 user_pref(\"network.proxy.ssl\", \"{h}\");\n\
                 user_pref(\"network.proxy.ssl_port\", {p});\n"
            )
        }
        _ => "user_pref(\"network.proxy.type\", 0);\n".to_string(),
    };

    format!(
        "// Mask-generated user.js — regenerated on each launch\n\
         user_pref(\"general.useragent.override\", \"{ua}\");\n\
         user_pref(\"privacy.resistFingerprinting\", true);\n\
         user_pref(\"privacy.resistFingerprinting.block_mozAddonManager\", true);\n\
         user_pref(\"media.peerconnection.enabled\", false);\n\
         user_pref(\"media.peerconnection.ice.default_address_only\", true);\n\
         user_pref(\"media.peerconnection.ice.no_host\", true);\n\
         user_pref(\"webgl.disabled\", true);\n\
         user_pref(\"dom.battery.enabled\", false);\n\
         user_pref(\"geo.enabled\", false);\n\
         user_pref(\"network.dns.disablePrefetch\", true);\n\
         user_pref(\"network.prefetch-next\", false);\n\
         user_pref(\"browser.cache.disk.enable\", false);\n\
         user_pref(\"browser.sessionstore.privacy_level\", 2);\n\
         user_pref(\"intl.accept_languages\", \"{locale}\");\n\
         user_pref(\"xpinstall.signatures.required\", false);\n\
         {proxy_prefs}",
        ua = ua.replace('"', "\\\""),
        locale = persona.locale.replace('"', "\\\""),
        proxy_prefs = proxy_prefs,
    )
}

fn ensure_mask_extension(ext_dir: &PathBuf, persona: &Persona) {
    std::fs::create_dir_all(ext_dir).ok();

    let name_escaped = persona.name.replace('"', "\\\"");
    let color = &persona.color;

    let manifest = format!(r#"{{
  "manifest_version": 2,
  "name": "Mask Indicator",
  "version": "1.0",
  "description": "Shows the active Mask persona in the toolbar.",
  "browser_action": {{
    "default_title": "{name_escaped}",
    "default_popup": "popup.html"
  }},
  "permissions": ["storage"],
  "background": {{ "scripts": ["background.js"], "persistent": false }}
}}"#);

    let popup_html = format!(r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8">
<style>
  body {{ margin:0; padding:10px 14px; background:#0f1117; color:#fff;
          font-family:system-ui,sans-serif; font-size:13px; min-width:160px; }}
  .badge {{ display:inline-block; width:10px; height:10px; border-radius:50%;
            margin-right:6px; background:{color}; vertical-align:middle; }}
  .name  {{ font-weight:600; vertical-align:middle; }}
  .sub   {{ color:#9ca3af; font-size:11px; margin-top:4px; }}
</style>
</head>
<body>
  <span class="badge"></span><span class="name">{name_escaped}</span>
  <div class="sub">Active Mask persona</div>
  <div class="sub" id="ip">Checking IP…</div>
  <script src="popup.js"></script>
</body>
</html>"#);

    let popup_js = r#"
document.addEventListener('DOMContentLoaded', () => {
  browser.storage.local.get('egress_ip').then(data => {
    document.getElementById('ip').textContent =
      data.egress_ip ? 'IP: ' + data.egress_ip : 'IP unknown';
  });
});
"#;

    let background_js = r#"
// Placeholder — receives messages from Mask native app via native messaging
// or reads from storage set by the host app.
"#;

    std::fs::write(ext_dir.join("manifest.json"), manifest).ok();
    std::fs::write(ext_dir.join("popup.html"), popup_html).ok();
    std::fs::write(ext_dir.join("popup.js"), popup_js).ok();
    std::fs::write(ext_dir.join("background.js"), background_js).ok();
}

fn setup_firefox_profile(persona: &Persona) -> Result<PathBuf, String> {
    let dir = profile_dir(&persona.id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    std::fs::write(dir.join("user.js"), firefox_user_js(persona))
        .map_err(|e| e.to_string())?;

    // Write active persona JSON for the extension
    let active_json = serde_json::json!({
        "id": persona.id,
        "name": persona.name,
        "color": persona.color,
    });
    std::fs::write(dir.join("mask_persona.json"), active_json.to_string()).ok();

    // Install the Mask indicator extension into the profile
    let ext_guid = "mask-indicator@mask.local";
    let ext_dir = dir.join("extensions").join(ext_guid);
    ensure_mask_extension(&ext_dir, persona);

    Ok(dir)
}

// ── Chromium / Brave ───────────────────────────────────────────────────────

fn chromium_prefs(_persona: &Persona) -> serde_json::Value {
    serde_json::json!({
        "profile": {
            "content_settings": {},
            "default_content_setting_values": {
                "geolocation": 2,
                "media_stream_camera": 2,
                "media_stream_mic": 2,
            }
        }
    })
}

fn setup_chromium_profile(persona: &Persona) -> Result<PathBuf, String> {
    let dir = profile_dir(&persona.id);
    let default_dir = dir.join("Default");
    std::fs::create_dir_all(&default_dir).map_err(|e| e.to_string())?;

    // Write Preferences file
    let prefs = chromium_prefs(persona);
    std::fs::write(
        default_dir.join("Preferences"),
        serde_json::to_string_pretty(&prefs).unwrap(),
    ).map_err(|e| e.to_string())?;

    // Write active persona for extension
    std::fs::write(
        dir.join("mask_persona.json"),
        serde_json::json!({"id": persona.id, "name": persona.name, "color": persona.color}).to_string(),
    ).ok();

    // Write the extension source directory
    let ext_dir = extension_src_dir();
    ensure_mask_extension(&ext_dir, persona);

    Ok(dir)
}

fn chromium_launch_args(persona: &Persona, profile_dir: &PathBuf) -> Vec<String> {
    let mut args = vec![
        format!("--user-data-dir={}", profile_dir.display()),
        "--new-window".into(),
        "--no-first-run".into(),
        "--disable-sync".into(),
        "--disable-background-networking".into(),
        "--disable-client-side-phishing-detection".into(),
        "--disable-default-apps".into(),
        "--disable-hang-monitor".into(),
        "--disable-prompt-on-repost".into(),
        "--disable-translate".into(),
        "--metrics-recording-only".into(),
        "--safebrowsing-disable-auto-update".into(),
        // WebRTC disable
        "--disable-webrtc-hw-encoding".into(),
        "--disable-webrtc-hw-decoding".into(),
        "--enforce-webrtc-ip-permission-check".into(),
        "--webrtc-ip-handling-policy=disable_non_proxied_udp".into(),
        // Fingerprint hardening
        "--disable-reading-from-canvas".into(),
        format!("--lang={}", persona.locale),
        // Load extension
        format!("--load-extension={}", extension_src_dir().display()),
    ];

    if !persona.user_agent.is_empty() {
        args.push(format!("--user-agent={}", persona.user_agent));
    }

    match persona.proxy_type.as_deref() {
        Some("socks5") => {
            let h = persona.proxy_host.as_deref().unwrap_or("127.0.0.1");
            let p = persona.proxy_port.unwrap_or(1080);
            args.push(format!("--proxy-server=socks5://{h}:{p}"));
            args.push("--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost".into());
        }
        Some("http") => {
            let h = persona.proxy_host.as_deref().unwrap_or("127.0.0.1");
            let p = persona.proxy_port.unwrap_or(8080);
            args.push(format!("--proxy-server=http://{h}:{p}"));
        }
        _ => {}
    }

    args
}

// ── find_binary helper ─────────────────────────────────────────────────────

fn find_binary(candidates: &[&str]) -> Option<String> {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for bin in candidates {
        for dir in std::env::split_paths(&path_var) {
            if dir.join(bin).is_file() {
                return Some(bin.to_string());
            }
        }
    }
    None
}

// ── Public commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn launch_browser(persona_id: String) -> Result<String, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id,name,description,color,proxy_type,proxy_host,proxy_port,\
         proxy_user,proxy_pass,vpn_config,timezone,locale,user_agent,notes,\
         browser_type,container_mode,wg_interface,created_at,updated_at FROM personas WHERE id=?1"
    ).map_err(|e| e.to_string())?;

    let persona = stmt.query_row([&persona_id], |row| {
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
    }).map_err(|e| format!("Persona not found: {e}"))?;

    let msg = match persona.browser_type.as_str() {
        "chromium" | "brave" => launch_chromium(&persona)?,
        _ => launch_firefox(&persona)?,
    };

    db::log_event(&conn, Some(&persona.id), Some(&persona.name), "browser_launched",
                  Some(&persona.browser_type), Some("ok")).ok();
    Ok(msg)
}

fn launch_firefox(persona: &Persona) -> Result<String, String> {
    let profile_dir = setup_firefox_profile(persona)?;

    let bin = find_binary(&["firefox", "firefox-esr", "firefox-bin"])
        .ok_or("Firefox not found in PATH")?;

    let mut cmd = Command::new(&bin);
    cmd.arg("--profile").arg(&profile_dir)
       .arg("--new-instance");

    if persona.timezone != "UTC" {
        cmd.env("TZ", &persona.timezone);
    }

    cmd.spawn().map_err(|e| format!("Failed to launch Firefox: {e}"))?;
    Ok(format!("Launched Firefox for '{}'", persona.name))
}

fn launch_chromium(persona: &Persona) -> Result<String, String> {
    let profile_dir = setup_chromium_profile(persona)?;

    let candidates: &[&str] = match persona.browser_type.as_str() {
        "brave" => &["brave", "brave-browser", "brave-browser-stable"],
        _       => &["chromium", "chromium-browser", "google-chrome", "google-chrome-stable"],
    };
    let bin = find_binary(candidates)
        .ok_or_else(|| format!("{} not found in PATH", persona.browser_type))?;

    let args = chromium_launch_args(persona, &profile_dir);

    let mut cmd = Command::new(&bin);
    for arg in &args { cmd.arg(arg); }

    if persona.timezone != "UTC" {
        cmd.env("TZ", &persona.timezone);
    }

    cmd.spawn().map_err(|e| format!("Failed to launch {}: {e}", persona.browser_type))?;
    Ok(format!("Launched {} for '{}'", persona.browser_type, persona.name))
}

#[tauri::command]
pub fn get_profile_path(persona_id: String) -> String {
    profile_dir(&persona_id).to_string_lossy().into_owned()
}

#[tauri::command]
pub fn detect_browsers() -> Vec<String> {
    let mut found = Vec::new();
    let pairs: &[(&str, &[&str])] = &[
        ("firefox",  &["firefox", "firefox-esr"]),
        ("chromium", &["chromium", "chromium-browser"]),
        ("brave",    &["brave", "brave-browser", "brave-browser-stable"]),
    ];
    for (name, bins) in pairs {
        if find_binary(bins).is_some() {
            found.push(name.to_string());
        }
    }
    found
}

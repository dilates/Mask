use crate::{db, models::Persona};
use std::process::Command;

fn runtime(mode: &str) -> &'static str {
    match mode {
        "docker" => "docker",
        _ => "podman",
    }
}

fn container_name(persona_id: &str) -> String {
    format!("mask-persona-{}", &persona_id[..8])
}

/// Launch a browser inside an isolated container with its own network namespace.
/// The container mounts the persona's profile directory and routes through the
/// configured proxy via environment variables.
#[tauri::command]
pub fn launch_container_browser(persona_id: String) -> Result<String, String> {
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

    if persona.container_mode == "none" {
        return Err("Container mode is not enabled for this persona".into());
    }

    let rt = runtime(&persona.container_mode);
    let name = container_name(&persona.id);
    let profile_dir = db::data_dir().join("profiles").join(&persona.id);
    std::fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;

    // Determine browser binary inside container
    let browser_bin = match persona.browser_type.as_str() {
        "chromium" => "chromium",
        "brave"    => "brave-browser",
        _          => "firefox",
    };

    // Container image — uses a public image with the browser pre-installed
    let image = match persona.browser_type.as_str() {
        "chromium" | "brave" => "ghcr.io/jlesage/chromium:latest",
        _ => "jlesage/firefox:latest",
    };

    let display = std::env::var("DISPLAY").unwrap_or(":0".into());
    let xauth = std::env::var("XAUTHORITY")
        .unwrap_or_else(|_| format!("{}/.Xauthority", std::env::var("HOME").unwrap_or_default()));

    let mut args: Vec<String> = vec![
        "run".into(),
        "--rm".into(),
        "--name".into(), name.clone(),
        "--network=host".into(),            // shares host network; proxy routing applies
        "--ipc=host".into(),
        // Mount X11 socket and profile volume
        "-v".into(), "/tmp/.X11-unix:/tmp/.X11-unix:rw".into(),
        "-v".into(), format!("{}:/home/user/.mozilla:rw", profile_dir.display()),
        "-v".into(), format!("{xauth}:/root/.Xauthority:ro"),
        // Environment
        "-e".into(), format!("DISPLAY={display}"),
        "-e".into(), format!("TZ={}", persona.timezone),
    ];

    // Proxy env vars inside container
    if let (Some(pt), Some(ph), Some(pp)) = (
        persona.proxy_type.as_deref(),
        persona.proxy_host.as_deref(),
        persona.proxy_port,
    ) {
        let proxy_url = if pt == "socks5" {
            format!("socks5://{ph}:{pp}")
        } else {
            format!("http://{ph}:{pp}")
        };
        args.push("-e".into());
        args.push(format!("http_proxy={proxy_url}"));
        args.push("-e".into());
        args.push(format!("https_proxy={proxy_url}"));
        args.push("-e".into());
        args.push(format!("HTTP_PROXY={proxy_url}"));
        args.push("-e".into());
        args.push(format!("HTTPS_PROXY={proxy_url}"));
    }

    args.push(image.into());
    args.push(browser_bin.into());

    Command::new(rt)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to start {rt}: {e}"))?;

    db::log_event(
        &conn, Some(&persona.id), Some(&persona.name),
        "container_launched", Some(rt), Some("ok"),
    ).ok();

    Ok(format!(
        "Launched {} in {} container '{}' for '{}'",
        persona.browser_type, rt, name, persona.name
    ))
}

/// Stop a running persona container.
#[tauri::command]
pub fn stop_container(persona_id: String) -> Result<String, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let (name_str, mode): (String, String) = conn.query_row(
        "SELECT name, container_mode FROM personas WHERE id=?1",
        [&persona_id],
        |row| Ok((row.get(0)?, row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "podman".into()))),
    ).map_err(|e| e.to_string())?;

    let rt = runtime(&mode);
    let cname = container_name(&persona_id);

    let out = Command::new(rt)
        .args(["stop", &cname])
        .output()
        .map_err(|e| format!("Failed to stop container: {e}"))?;

    if out.status.success() {
        db::log_event(&conn, Some(&persona_id), Some(&name_str), "container_stopped", None, Some("ok")).ok();
        Ok(format!("Stopped container '{cname}'"))
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        Err(format!("Stop failed: {stderr}"))
    }
}

/// Check if podman or docker is available.
#[tauri::command]
pub fn detect_container_runtimes() -> Vec<String> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    ["podman", "docker"]
        .iter()
        .filter(|&&rt| {
            std::env::split_paths(&path).any(|d| d.join(rt).is_file())
        })
        .map(|s| s.to_string())
        .collect()
}

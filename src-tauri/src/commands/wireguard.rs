use crate::{db, models::WgStatus};
use std::process::Command;

/// Run a privileged command via pkexec (GUI sudo), falling back to sudo.
fn privileged(args: &[&str]) -> std::io::Result<std::process::Output> {
    // Try pkexec first (shows a polkit dialog — no password prompt in terminal)
    let pkexec = Command::new("pkexec")
        .args(args)
        .output();

    match pkexec {
        Ok(out) => Ok(out),
        Err(_) => Command::new("sudo").args(args).output(),
    }
}

fn wg_quick(action: &str, config_or_iface: &str) -> Result<String, String> {
    let out = privileged(&["wg-quick", action, config_or_iface])
        .map_err(|e| format!("wg-quick not found or failed to execute: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    if out.status.success() {
        Ok(stdout + &stderr)
    } else {
        Err(format!("wg-quick {action} failed:\n{stderr}"))
    }
}

#[tauri::command]
pub fn wg_up(persona_id: String) -> Result<WgStatus, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let (name, vpn_config, wg_iface): (String, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT name, vpn_config, wg_interface FROM personas WHERE id=?1",
            [&persona_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Persona not found: {e}"))?;

    // Prefer explicit interface name, then vpn_config path, then error
    let target = wg_iface
        .clone()
        .or_else(|| vpn_config.clone())
        .ok_or("No WireGuard config or interface set on this persona")?;

    let detail = wg_quick("up", &target)?;

    db::log_event(
        &conn, Some(&persona_id), Some(&name),
        "wg_up", Some(&target), Some("ok"),
    ).ok();

    Ok(WgStatus {
        interface: target,
        up: true,
        detail,
    })
}

#[tauri::command]
pub fn wg_down(persona_id: String) -> Result<WgStatus, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let (name, vpn_config, wg_iface): (String, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT name, vpn_config, wg_interface FROM personas WHERE id=?1",
            [&persona_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Persona not found: {e}"))?;

    let target = wg_iface
        .clone()
        .or_else(|| vpn_config.clone())
        .ok_or("No WireGuard config or interface set on this persona")?;

    let detail = wg_quick("down", &target)?;

    db::log_event(
        &conn, Some(&persona_id), Some(&name),
        "wg_down", Some(&target), Some("ok"),
    ).ok();

    Ok(WgStatus {
        interface: target,
        up: false,
        detail,
    })
}

/// Check if a WireGuard interface is currently up by reading /proc/net/if_inet6
/// or using `wg show`. Returns true if the interface appears in `wg show`.
#[tauri::command]
pub fn wg_status(persona_id: String) -> Result<WgStatus, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let (vpn_config, wg_iface): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT vpn_config, wg_interface FROM personas WHERE id=?1",
            [&persona_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Persona not found: {e}"))?;

    let target = wg_iface
        .clone()
        .or_else(|| vpn_config.as_deref().and_then(|p| {
            std::path::Path::new(p).file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        }))
        .ok_or("No WireGuard interface configured")?;

    // Run `wg show <iface>` — succeeds if the interface is up
    let out = Command::new("wg")
        .args(["show", &target])
        .output()
        .map_err(|e| format!("wg not found: {e}"))?;

    let up = out.status.success();
    let detail = String::from_utf8_lossy(&out.stdout).trim().to_string();

    Ok(WgStatus {
        interface: target,
        up,
        detail: if up { detail } else { "Interface is down".into() },
    })
}

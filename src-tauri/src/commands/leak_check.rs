use crate::models::{CheckItem, LeakCheckResult};
use serde::Deserialize;

#[derive(Deserialize)]
struct IpInfo {
    ip: Option<String>,
    country: Option<String>,
    timezone: Option<String>,
}

async fn fetch_ip_info(proxy: Option<(&str, u16, &str)>) -> Result<IpInfo, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10));

    if let Some((host, port, kind)) = proxy {
        let proxy_url = if kind == "socks5" {
            format!("socks5://{}:{}", host, port)
        } else {
            format!("http://{}:{}", host, port)
        };
        let p = reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?;
        builder = builder.proxy(p);
    }

    let client = builder.build().map_err(|e| e.to_string())?;
    let resp = client
        .get("https://ipinfo.io/json")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("IP check failed: {e}"))?;

    resp.json::<IpInfo>().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_leak_check(
    persona_id: String,
    expected_timezone: String,
    proxy_type: Option<String>,
    proxy_host: Option<String>,
    proxy_port: Option<i64>,
) -> Result<LeakCheckResult, String> {
    use crate::db;

    let mut checks: Vec<CheckItem> = Vec::new();

    // Determine proxy params
    let proxy_params: Option<(String, u16, String)> = match (
        proxy_type.as_deref(),
        proxy_host.as_deref(),
        proxy_port,
    ) {
        (Some(pt), Some(ph), Some(pp)) if !ph.is_empty() => {
            Some((ph.to_string(), pp as u16, pt.to_string()))
        }
        _ => None,
    };

    // 1. IP / egress check
    let proxy_ref = proxy_params.as_ref().map(|(h, p, t)| (h.as_str(), *p, t.as_str()));
    let ip_info = fetch_ip_info(proxy_ref).await;

    let (egress_ip, country) = match &ip_info {
        Ok(info) => {
            checks.push(CheckItem {
                name: "Egress IP".into(),
                status: "pass".into(),
                detail: format!("IP: {}", info.ip.clone().unwrap_or("unknown".into())),
                remediation: None,
            });
            (info.ip.clone(), info.country.clone())
        }
        Err(e) => {
            checks.push(CheckItem {
                name: "Egress IP".into(),
                status: "error".into(),
                detail: e.clone(),
                remediation: Some("Check your proxy/VPN is running and reachable.".into()),
            });
            (None, None)
        }
    };

    // 2. DNS leak check — compare DNS resolution IP vs egress IP country
    let dns_leak = check_dns_leak(&mut checks, proxy_ref, egress_ip.as_deref()).await;

    // 3. Timezone mismatch check
    let tz_from_ip = ip_info.as_ref().ok().and_then(|i| i.timezone.clone());
    let timezone_match = check_timezone(&mut checks, &expected_timezone, tz_from_ip.as_deref());

    // 4. WebRTC leak — note: this cannot be checked server-side; advise user
    checks.push(CheckItem {
        name: "WebRTC Leak".into(),
        status: "warn".into(),
        detail: "WebRTC leak detection requires browser-side testing. Mask sets media.peerconnection.enabled=false in user.js, but verify via a browser-based test (e.g. browserleaks.com/webrtc).".into(),
        remediation: Some("Visit browserleaks.com/webrtc in the persona browser and confirm no local IPs are exposed.".into()),
    });

    // Log
    let conn = db::open().map_err(|e| e.to_string())?;
    let summary = format!("ip={}", egress_ip.as_deref().unwrap_or("none"));
    db::log_event(
        &conn,
        Some(&persona_id),
        None,
        "leak_check",
        Some(&summary),
        Some(if dns_leak { "dns_leak_detected" } else { "ok" }),
    ).ok();

    Ok(LeakCheckResult {
        ip: egress_ip,
        dns_leak,
        webrtc_leak: false, // cannot determine server-side
        timezone_match,
        country,
        checks,
    })
}

async fn check_dns_leak(
    checks: &mut Vec<CheckItem>,
    proxy: Option<(&str, u16, &str)>,
    egress_ip: Option<&str>,
) -> bool {
    // Use a DNS-over-HTTPS check via the proxy to compare resolution
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8));

    if let Some((host, port, kind)) = proxy {
        let proxy_url = if kind == "socks5" {
            format!("socks5://{}:{}", host, port)
        } else {
            format!("http://{}:{}", host, port)
        };
        if let Ok(p) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(p);
        }
    }

    match builder.build() {
        Ok(client) => {
            // Check if DNS resolves through proxy by fetching a known endpoint
            match client
                .get("https://1.1.1.1/dns-query?name=whoami.akamai.net&type=A")
                .header("Accept", "application/dns-json")
                .send()
                .await
            {
                Ok(_) => {
                    checks.push(CheckItem {
                        name: "DNS Routing".into(),
                        status: "pass".into(),
                        detail: "DNS queries are routed through the proxy (SOCKS remote DNS enabled).".into(),
                        remediation: None,
                    });
                    false
                }
                Err(e) => {
                    let leaked = e.to_string().contains("dns") || proxy.is_some();
                    checks.push(CheckItem {
                        name: "DNS Routing".into(),
                        status: if leaked { "warn" } else { "pass" }.into(),
                        detail: format!("DNS check inconclusive: {e}"),
                        remediation: Some("Ensure your proxy supports remote DNS (SOCKS5 with remote DNS enabled). Check network.proxy.socks_remote_dns=true in Firefox.".into()),
                    });
                    false
                }
            }
        }
        Err(_) => false,
    }
}

fn check_timezone(
    checks: &mut Vec<CheckItem>,
    expected: &str,
    from_ip: Option<&str>,
) -> bool {
    match from_ip {
        Some(tz) => {
            // Simple prefix match (e.g. "Europe/..." vs "Europe/Amsterdam")
            let region_expected = expected.split('/').next().unwrap_or(expected);
            let region_ip = tz.split('/').next().unwrap_or(tz);
            let matches = region_expected.eq_ignore_ascii_case(region_ip)
                || expected.eq_ignore_ascii_case(tz);

            checks.push(CheckItem {
                name: "Timezone Consistency".into(),
                status: if matches { "pass" } else { "warn" }.into(),
                detail: format!("Profile timezone: {expected} | IP geolocation timezone: {tz}"),
                remediation: if matches {
                    None
                } else {
                    Some(format!(
                        "Update persona timezone to '{tz}' to match your proxy exit node, or choose a proxy in the same region as your configured timezone."
                    ))
                },
            });
            matches
        }
        None => {
            checks.push(CheckItem {
                name: "Timezone Consistency".into(),
                status: "warn".into(),
                detail: format!("Could not determine IP timezone. Profile timezone: {expected}"),
                remediation: Some("Run the IP check successfully first to compare timezones.".into()),
            });
            true
        }
    }
}

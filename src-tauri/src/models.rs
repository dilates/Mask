use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: String,
    pub proxy_type: Option<String>,  // "socks5" | "http" | null
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i64>,
    pub proxy_user: Option<String>,
    pub proxy_pass: Option<String>,
    pub vpn_config: Option<String>,  // path to .conf file
    pub timezone: String,
    pub locale: String,
    pub user_agent: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePersona {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i64>,
    pub proxy_user: Option<String>,
    pub proxy_pass: Option<String>,
    pub vpn_config: Option<String>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub user_agent: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePersona {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i64>,
    pub proxy_user: Option<String>,
    pub proxy_pass: Option<String>,
    pub vpn_config: Option<String>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub user_agent: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub persona_id: Option<String>,
    pub persona_name: Option<String>,
    pub event_type: String,
    pub detail: Option<String>,
    pub result: Option<String>,
    pub ts: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakCheckResult {
    pub ip: Option<String>,
    pub dns_leak: bool,
    pub webrtc_leak: bool,
    pub timezone_match: bool,
    pub country: Option<String>,
    pub checks: Vec<CheckItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub name: String,
    pub status: String,   // "pass" | "fail" | "warn" | "error"
    pub detail: String,
    pub remediation: Option<String>,
}

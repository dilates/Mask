export interface Persona {
  id: string;
  name: string;
  description: string;
  color: string;
  proxy_type: "socks5" | "http" | null;
  proxy_host: string | null;
  proxy_port: number | null;
  proxy_user: string | null;
  proxy_pass: string | null;
  vpn_config: string | null;
  timezone: string;
  locale: string;
  user_agent: string;
  notes: string;
  created_at: string;
  updated_at: string;
}

export interface CreatePersona {
  name: string;
  description?: string;
  color?: string;
  proxy_type?: string;
  proxy_host?: string;
  proxy_port?: number;
  proxy_user?: string;
  proxy_pass?: string;
  vpn_config?: string;
  timezone?: string;
  locale?: string;
  user_agent?: string;
  notes?: string;
}

export interface UpdatePersona extends Partial<CreatePersona> {
  id: string;
}

export interface AuditEntry {
  id: number;
  persona_id: string | null;
  persona_name: string | null;
  event_type: string;
  detail: string | null;
  result: string | null;
  ts: string;
}

export interface CheckItem {
  name: string;
  status: "pass" | "fail" | "warn" | "error";
  detail: string;
  remediation: string | null;
}

export interface LeakCheckResult {
  ip: string | null;
  dns_leak: boolean;
  webrtc_leak: boolean;
  timezone_match: boolean;
  country: string | null;
  checks: CheckItem[];
}

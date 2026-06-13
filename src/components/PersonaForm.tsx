import React, { useState, useEffect } from "react";
import { X } from "lucide-react";
import type { Persona, CreatePersona } from "../types";

const COLORS = [
  "#6366f1", "#8b5cf6", "#ec4899", "#ef4444",
  "#f97316", "#eab308", "#22c55e", "#06b6d4",
];

const TIMEZONES = [
  "UTC", "America/New_York", "America/Chicago", "America/Denver",
  "America/Los_Angeles", "Europe/London", "Europe/Paris", "Europe/Berlin",
  "Europe/Amsterdam", "Asia/Tokyo", "Asia/Shanghai", "Asia/Kolkata",
  "Australia/Sydney",
];

const LOCALES = [
  "en-US", "en-GB", "de-DE", "fr-FR", "es-ES", "nl-NL",
  "ja-JP", "zh-CN", "ru-RU", "pt-BR",
];

interface Props {
  persona?: Persona;
  onSave: (data: CreatePersona) => void;
  onClose: () => void;
}

export function PersonaForm({ persona, onSave, onClose }: Props) {
  const [form, setForm] = useState<CreatePersona>({
    name: "",
    description: "",
    color: COLORS[0],
    proxy_type: undefined,
    proxy_host: "",
    proxy_port: undefined,
    proxy_user: "",
    proxy_pass: "",
    vpn_config: "",
    timezone: "UTC",
    locale: "en-US",
    user_agent: "",
    notes: "",
  });

  useEffect(() => {
    if (persona) {
      setForm({
        name: persona.name,
        description: persona.description,
        color: persona.color,
        proxy_type: persona.proxy_type ?? undefined,
        proxy_host: persona.proxy_host ?? "",
        proxy_port: persona.proxy_port ?? undefined,
        proxy_user: persona.proxy_user ?? "",
        proxy_pass: persona.proxy_pass ?? "",
        vpn_config: persona.vpn_config ?? "",
        timezone: persona.timezone,
        locale: persona.locale,
        user_agent: persona.user_agent,
        notes: persona.notes,
      });
    }
  }, [persona]);

  const set = (k: keyof CreatePersona, v: unknown) =>
    setForm((f) => ({ ...f, [k]: v }));

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(form);
  };

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-panel border border-border rounded-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between p-6 border-b border-border">
          <h2 className="text-lg font-semibold text-white">
            {persona ? "Edit Persona" : "New Persona"}
          </h2>
          <button onClick={onClose} className="text-gray-400 hover:text-white">
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-5">
          {/* Name + Color */}
          <div className="flex gap-4">
            <div className="flex-1">
              <label className="label">Name</label>
              <input
                className="input"
                required
                value={form.name}
                onChange={(e) => set("name", e.target.value)}
                placeholder="e.g. Researcher Alpha"
              />
            </div>
            <div>
              <label className="label">Color</label>
              <div className="flex gap-2 mt-1">
                {COLORS.map((c) => (
                  <button
                    key={c}
                    type="button"
                    onClick={() => set("color", c)}
                    className="w-7 h-7 rounded-full border-2 transition-transform hover:scale-110"
                    style={{
                      backgroundColor: c,
                      borderColor: form.color === c ? "white" : "transparent",
                    }}
                  />
                ))}
              </div>
            </div>
          </div>

          {/* Description */}
          <div>
            <label className="label">Description</label>
            <input
              className="input"
              value={form.description}
              onChange={(e) => set("description", e.target.value)}
              placeholder="Purpose or context for this persona"
            />
          </div>

          {/* Proxy */}
          <fieldset className="border border-border rounded-lg p-4 space-y-3">
            <legend className="text-xs text-gray-400 px-1">Network Routing</legend>
            <div className="flex gap-3">
              <div className="w-36">
                <label className="label">Proxy Type</label>
                <select
                  className="input"
                  value={form.proxy_type ?? ""}
                  onChange={(e) =>
                    set("proxy_type", e.target.value || undefined)
                  }
                >
                  <option value="">None</option>
                  <option value="socks5">SOCKS5</option>
                  <option value="http">HTTP</option>
                </select>
              </div>
              {form.proxy_type && (
                <>
                  <div className="flex-1">
                    <label className="label">Host</label>
                    <input
                      className="input"
                      value={form.proxy_host ?? ""}
                      onChange={(e) => set("proxy_host", e.target.value)}
                      placeholder="127.0.0.1"
                    />
                  </div>
                  <div className="w-24">
                    <label className="label">Port</label>
                    <input
                      className="input"
                      type="number"
                      value={form.proxy_port ?? ""}
                      onChange={(e) =>
                        set("proxy_port", e.target.value ? parseInt(e.target.value) : undefined)
                      }
                      placeholder="1080"
                    />
                  </div>
                </>
              )}
            </div>
            {form.proxy_type && (
              <div className="flex gap-3">
                <div className="flex-1">
                  <label className="label">Username (optional)</label>
                  <input
                    className="input"
                    value={form.proxy_user ?? ""}
                    onChange={(e) => set("proxy_user", e.target.value)}
                  />
                </div>
                <div className="flex-1">
                  <label className="label">Password (optional)</label>
                  <input
                    className="input"
                    type="password"
                    value={form.proxy_pass ?? ""}
                    onChange={(e) => set("proxy_pass", e.target.value)}
                  />
                </div>
              </div>
            )}
            <div>
              <label className="label">VPN Config Path (optional)</label>
              <input
                className="input"
                value={form.vpn_config ?? ""}
                onChange={(e) => set("vpn_config", e.target.value)}
                placeholder="/etc/wireguard/persona-alpha.conf"
              />
            </div>
          </fieldset>

          {/* Fingerprint */}
          <fieldset className="border border-border rounded-lg p-4 space-y-3">
            <legend className="text-xs text-gray-400 px-1">Fingerprint</legend>
            <div className="flex gap-3">
              <div className="flex-1">
                <label className="label">Timezone</label>
                <select
                  className="input"
                  value={form.timezone}
                  onChange={(e) => set("timezone", e.target.value)}
                >
                  {TIMEZONES.map((tz) => (
                    <option key={tz} value={tz}>{tz}</option>
                  ))}
                </select>
              </div>
              <div className="flex-1">
                <label className="label">Locale</label>
                <select
                  className="input"
                  value={form.locale}
                  onChange={(e) => set("locale", e.target.value)}
                >
                  {LOCALES.map((l) => (
                    <option key={l} value={l}>{l}</option>
                  ))}
                </select>
              </div>
            </div>
            <div>
              <label className="label">User-Agent (leave blank for default)</label>
              <input
                className="input font-mono text-sm"
                value={form.user_agent ?? ""}
                onChange={(e) => set("user_agent", e.target.value)}
                placeholder="Mozilla/5.0 (X11; Linux x86_64; rv:128.0)..."
              />
            </div>
          </fieldset>

          {/* Notes */}
          <div>
            <label className="label">Notes</label>
            <textarea
              className="input resize-none"
              rows={3}
              value={form.notes}
              onChange={(e) => set("notes", e.target.value)}
              placeholder="Accounts associated, use case, reminders..."
            />
          </div>

          <div className="flex gap-3 justify-end pt-2">
            <button type="button" onClick={onClose} className="btn-secondary">
              Cancel
            </button>
            <button type="submit" className="btn-primary">
              {persona ? "Save Changes" : "Create Persona"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

import { useState } from "react";
import { CheckCircle, XCircle, AlertTriangle, Loader2, X } from "lucide-react";
import { api } from "../api";
import type { Persona, LeakCheckResult, CheckItem } from "../types";

interface Props {
  persona: Persona;
  onClose: () => void;
}

export function LeakCheckPanel({ persona, onClose }: Props) {
  const [result, setResult] = useState<LeakCheckResult | null>(null);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = async () => {
    setRunning(true);
    setError(null);
    try {
      const r = await api.runLeakCheck({
        personaId: persona.id,
        expectedTimezone: persona.timezone,
        proxyType: persona.proxy_type ?? undefined,
        proxyHost: persona.proxy_host ?? undefined,
        proxyPort: persona.proxy_port ?? undefined,
      });
      setResult(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  };

  const overallStatus = result
    ? result.checks.every((c) => c.status === "pass")
      ? "pass"
      : result.checks.some((c) => c.status === "fail" || c.status === "error")
      ? "fail"
      : "warn"
    : null;

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-panel border border-border rounded-xl w-full max-w-lg max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between p-5 border-b border-border">
          <div>
            <h2 className="text-base font-semibold text-white">Mask Check</h2>
            <p className="text-sm text-gray-400">{persona.name}</p>
          </div>
          <button onClick={onClose} className="text-gray-400 hover:text-white">
            <X size={20} />
          </button>
        </div>

        <div className="p-5 space-y-4">
          {!result && !running && (
            <div className="text-center space-y-3 py-4">
              <p className="text-gray-400 text-sm">
                Checks egress IP, DNS routing, timezone consistency, and WebRTC leak status for this persona.
              </p>
              {!persona.proxy_type && !persona.vpn_config && (
                <div className="bg-yellow-900/30 border border-yellow-700/50 rounded-lg p-3 text-yellow-300 text-sm">
                  No proxy configured — this persona will use your direct connection.
                </div>
              )}
              <button onClick={run} className="btn-primary mx-auto flex items-center gap-2">
                Run Mask Check
              </button>
            </div>
          )}

          {running && (
            <div className="flex items-center justify-center gap-3 py-8 text-gray-400">
              <Loader2 size={20} className="animate-spin" />
              <span>Running checks...</span>
            </div>
          )}

          {error && (
            <div className="bg-red-900/30 border border-red-700/50 rounded-lg p-4 text-red-300 text-sm">
              {error}
            </div>
          )}

          {result && (
            <div className="space-y-4">
              {/* Summary */}
              <div className={`rounded-lg p-4 border ${
                overallStatus === "pass"
                  ? "bg-green-900/20 border-green-700/50 text-green-300"
                  : overallStatus === "fail"
                  ? "bg-red-900/20 border-red-700/50 text-red-300"
                  : "bg-yellow-900/20 border-yellow-700/50 text-yellow-300"
              }`}>
                <div className="font-semibold text-sm mb-1">
                  {overallStatus === "pass" ? "All checks passed" : overallStatus === "fail" ? "Issues detected" : "Warnings present"}
                </div>
                {result.ip && (
                  <div className="text-xs opacity-80">
                    Egress IP: {result.ip} {result.country ? `(${result.country})` : ""}
                  </div>
                )}
              </div>

              {/* Check items */}
              <div className="space-y-2">
                {result.checks.map((check) => (
                  <CheckRow key={check.name} check={check} />
                ))}
              </div>

              <button onClick={run} className="btn-secondary w-full text-sm">
                Re-run Check
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function CheckRow({ check }: { check: CheckItem }) {
  const [expanded, setExpanded] = useState(false);

  const Icon =
    check.status === "pass"
      ? CheckCircle
      : check.status === "fail" || check.status === "error"
      ? XCircle
      : AlertTriangle;

  const color =
    check.status === "pass"
      ? "text-green-400"
      : check.status === "fail" || check.status === "error"
      ? "text-red-400"
      : "text-yellow-400";

  return (
    <div
      className="border border-border rounded-lg p-3 cursor-pointer hover:border-gray-600 transition-colors"
      onClick={() => setExpanded((e) => !e)}
    >
      <div className="flex items-center gap-2">
        <Icon size={16} className={color} />
        <span className="text-sm font-medium text-white flex-1">{check.name}</span>
        <span className={`text-xs font-mono uppercase ${color}`}>{check.status}</span>
      </div>
      {expanded && (
        <div className="mt-2 space-y-1 pl-6">
          <p className="text-xs text-gray-400">{check.detail}</p>
          {check.remediation && (
            <p className="text-xs text-yellow-400 italic">{check.remediation}</p>
          )}
        </div>
      )}
    </div>
  );
}

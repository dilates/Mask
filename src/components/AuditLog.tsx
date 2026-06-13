import { useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";
import { api } from "../api";
import type { AuditEntry } from "../types";

const EVENT_LABELS: Record<string, string> = {
  persona_created: "Created",
  persona_updated: "Updated",
  persona_deleted: "Deleted",
  browser_launched: "Browser launched",
  leak_check: "Mask Check",
};

const RESULT_COLOR: Record<string, string> = {
  ok: "text-green-400",
  dns_leak_detected: "text-red-400",
};

export function AuditLog() {
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [dataDir, setDataDir] = useState("");

  const load = async () => {
    setLoading(true);
    const [log, dir] = await Promise.all([
      api.getAuditLog(100),
      api.getDataDir(),
    ]);
    setEntries(log);
    setDataDir(dir);
    setLoading(false);
  };

  useEffect(() => { load(); }, []);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-white">Audit Log</h2>
          <p className="text-xs text-gray-500 mt-0.5">Data stored at: <code className="text-gray-400">{dataDir}</code></p>
        </div>
        <button onClick={load} className="icon-btn" title="Refresh">
          <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
        </button>
      </div>

      {entries.length === 0 && !loading && (
        <div className="text-center text-gray-500 py-12">No audit events yet.</div>
      )}

      <div className="space-y-1">
        {entries.map((e) => (
          <div key={e.id} className="flex items-start gap-3 bg-panel border border-border rounded-lg px-4 py-2.5 text-sm">
            <span className="text-gray-500 font-mono text-xs w-36 flex-shrink-0 mt-0.5">
              {new Date(e.ts).toLocaleString()}
            </span>
            <span className="text-indigo-400 w-36 flex-shrink-0 truncate">
              {e.persona_name ?? "—"}
            </span>
            <span className="text-gray-300 flex-1">
              {EVENT_LABELS[e.event_type] ?? e.event_type}
              {e.detail ? `: ${e.detail}` : ""}
            </span>
            {e.result && (
              <span className={`text-xs font-mono ${RESULT_COLOR[e.result] ?? "text-gray-400"}`}>
                {e.result}
              </span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

import { Edit2, Trash2, Play, Shield } from "lucide-react";
import type { Persona } from "../types";

interface Props {
  persona: Persona;
  onEdit: () => void;
  onDelete: () => void;
  onLaunch: () => void;
  onCheck: () => void;
}

export function PersonaCard({ persona, onEdit, onDelete, onLaunch, onCheck }: Props) {
  const proxyLabel = persona.proxy_type
    ? `${persona.proxy_type.toUpperCase()} ${persona.proxy_host}:${persona.proxy_port}`
    : persona.vpn_config
    ? "VPN"
    : "No proxy";

  return (
    <div className="bg-panel border border-border rounded-xl p-5 flex flex-col gap-4 hover:border-gray-600 transition-colors">
      {/* Header */}
      <div className="flex items-start gap-3">
        <div
          className="w-10 h-10 rounded-full flex items-center justify-center text-white font-bold text-sm flex-shrink-0"
          style={{ backgroundColor: persona.color }}
        >
          {persona.name.slice(0, 2).toUpperCase()}
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-white truncate">{persona.name}</h3>
          <p className="text-sm text-gray-400 truncate">{persona.description || "No description"}</p>
        </div>
        <div className="flex gap-1">
          <button
            onClick={onEdit}
            className="icon-btn"
            title="Edit persona"
          >
            <Edit2 size={15} />
          </button>
          <button
            onClick={onDelete}
            className="icon-btn text-red-400 hover:text-red-300 hover:bg-red-900/30"
            title="Delete persona"
          >
            <Trash2 size={15} />
          </button>
        </div>
      </div>

      {/* Details */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <Detail label="Proxy" value={proxyLabel} />
        <Detail label="Timezone" value={persona.timezone} />
        <Detail label="Locale" value={persona.locale} />
        <Detail
          label="Fingerprint"
          value={persona.user_agent ? "Custom UA" : "Default"}
        />
      </div>

      {/* Actions */}
      <div className="flex gap-2 pt-1">
        <button onClick={onLaunch} className="btn-primary flex-1 flex items-center justify-center gap-2 text-sm">
          <Play size={14} /> Launch Browser
        </button>
        <button onClick={onCheck} className="btn-secondary flex items-center gap-1.5 text-sm px-3" title="Mask Check">
          <Shield size={14} /> Check
        </button>
      </div>
    </div>
  );
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span className="text-gray-500">{label}: </span>
      <span className="text-gray-300">{value}</span>
    </div>
  );
}

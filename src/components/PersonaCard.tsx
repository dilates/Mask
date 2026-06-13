import { useState } from "react";
import { Edit2, Trash2, Play, Shield, Wifi, WifiOff, Box, StopCircle } from "lucide-react";
import { api } from "../api";
import type { Persona } from "../types";

const BROWSER_ICON: Record<string, string> = {
  firefox: "FF",
  chromium: "CR",
  brave: "BR",
};

interface Props {
  persona: Persona;
  onEdit: () => void;
  onDelete: () => void;
  onLaunch: () => void;
  onCheck: () => void;
  onNotify: (msg: string) => void;
}

export function PersonaCard({ persona, onEdit, onDelete, onLaunch, onCheck, onNotify }: Props) {
  const [wgUp, setWgUp] = useState<boolean | null>(null);
  const [wgBusy, setWgBusy] = useState(false);
  const [containerRunning, setContainerRunning] = useState(false);

  const proxyLabel = persona.proxy_type
    ? `${persona.proxy_type.toUpperCase()} ${persona.proxy_host}:${persona.proxy_port}`
    : persona.vpn_config
    ? "WireGuard"
    : "No proxy";

  const hasWg = !!(persona.vpn_config || persona.wg_interface);
  const hasContainer = persona.container_mode !== "none";

  const handleWgToggle = async () => {
    setWgBusy(true);
    try {
      if (wgUp) {
        const status = await api.wgDown(persona.id);
        setWgUp(false);
        onNotify(`WireGuard down: ${status.interface}`);
      } else {
        const status = await api.wgUp(persona.id);
        setWgUp(true);
        onNotify(`WireGuard up: ${status.interface}`);
      }
    } catch (e) {
      onNotify(`WireGuard error: ${e}`);
    } finally {
      setWgBusy(false);
    }
  };

  const handleContainerLaunch = async () => {
    try {
      const msg = await api.launchContainerBrowser(persona.id);
      setContainerRunning(true);
      onNotify(msg);
    } catch (e) {
      onNotify(`Container error: ${e}`);
    }
  };

  const handleContainerStop = async () => {
    try {
      const msg = await api.stopContainer(persona.id);
      setContainerRunning(false);
      onNotify(msg);
    } catch (e) {
      onNotify(`Stop error: ${e}`);
    }
  };

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
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-white truncate">{persona.name}</h3>
            <span className="text-xs px-1.5 py-0.5 rounded bg-white/10 text-gray-400 font-mono">
              {BROWSER_ICON[persona.browser_type] ?? persona.browser_type}
            </span>
            {hasContainer && (
              <span className="text-xs px-1.5 py-0.5 rounded bg-indigo-900/40 text-indigo-300">
                <Box size={10} className="inline mr-1" />{persona.container_mode}
              </span>
            )}
          </div>
          <p className="text-sm text-gray-400 truncate">{persona.description || "No description"}</p>
        </div>
        <div className="flex gap-1">
          <button onClick={onEdit} className="icon-btn" title="Edit"><Edit2 size={15} /></button>
          <button onClick={onDelete} className="icon-btn text-red-400 hover:text-red-300 hover:bg-red-900/30" title="Delete"><Trash2 size={15} /></button>
        </div>
      </div>

      {/* Details */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <Detail label="Proxy" value={proxyLabel} />
        <Detail label="Timezone" value={persona.timezone} />
        <Detail label="Locale" value={persona.locale} />
        <Detail label="Fingerprint" value={persona.user_agent ? "Custom UA" : "Default"} />
      </div>

      {/* WireGuard row */}
      {hasWg && (
        <div className="flex items-center justify-between bg-surface rounded-lg px-3 py-2 text-xs">
          <span className="text-gray-400">
            WireGuard: <span className="text-gray-300 font-mono">
              {persona.wg_interface ?? persona.vpn_config?.split("/").pop()}
            </span>
          </span>
          <button
            onClick={handleWgToggle}
            disabled={wgBusy}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium transition-colors ${
              wgUp
                ? "bg-green-900/40 text-green-300 hover:bg-green-900/60"
                : "bg-gray-700/40 text-gray-300 hover:bg-gray-700/60"
            } disabled:opacity-50`}
          >
            {wgUp ? <Wifi size={12} /> : <WifiOff size={12} />}
            {wgBusy ? "…" : wgUp ? "Up" : "Down"}
          </button>
        </div>
      )}

      {/* Actions */}
      <div className="flex gap-2 pt-1">
        {hasContainer ? (
          <>
            <button
              onClick={containerRunning ? handleContainerStop : handleContainerLaunch}
              className={`flex-1 flex items-center justify-center gap-2 text-sm rounded-lg px-3 py-2 font-medium transition-colors ${
                containerRunning
                  ? "bg-red-600/80 hover:bg-red-500 text-white"
                  : "bg-indigo-600 hover:bg-indigo-500 text-white"
              }`}
            >
              {containerRunning ? <StopCircle size={14} /> : <Box size={14} />}
              {containerRunning ? "Stop Container" : "Launch Container"}
            </button>
          </>
        ) : (
          <button onClick={onLaunch} className="btn-primary flex-1 flex items-center justify-center gap-2 text-sm">
            <Play size={14} /> Launch Browser
          </button>
        )}
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

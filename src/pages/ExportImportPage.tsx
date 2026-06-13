import { useState, useEffect } from "react";
import { Download, Upload, Eye, EyeOff, Copy, Check } from "lucide-react";
import { api } from "../api";
import type { Persona } from "../types";

export function ExportImportPage() {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [password, setPassword] = useState("");
  const [showPw, setShowPw] = useState(false);
  const [exportBlob, setExportBlob] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [importBlob, setImportBlob] = useState("");
  const [importPw, setImportPw] = useState("");
  const [showImportPw, setShowImportPw] = useState(false);
  const [status, setStatus] = useState<{ kind: "ok" | "err"; msg: string } | null>(null);

  useEffect(() => {
    api.listPersonas().then(setPersonas);
  }, []);

  const notify = (kind: "ok" | "err", msg: string) => {
    setStatus({ kind, msg });
    setTimeout(() => setStatus(null), 4000);
  };

  const toggleSelect = (id: string) =>
    setSelectedIds((s) => {
      const n = new Set(s);
      n.has(id) ? n.delete(id) : n.add(id);
      return n;
    });

  const handleExport = async () => {
    if (!password) { notify("err", "Enter an encryption password"); return; }
    try {
      const ids = selectedIds.size > 0 ? [...selectedIds] : [];
      const blob = await api.exportPersonas(ids, password);
      setExportBlob(blob);
      notify("ok", `Exported ${ids.length || personas.length} persona(s)`);
    } catch (e) { notify("err", String(e)); }
  };

  const handleCopy = () => {
    if (!exportBlob) return;
    navigator.clipboard.writeText(exportBlob);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleSaveFile = () => {
    if (!exportBlob) return;
    const a = document.createElement("a");
    a.href = "data:text/plain;charset=utf-8," + encodeURIComponent(exportBlob);
    a.download = `mask-export-${new Date().toISOString().slice(0, 10)}.enc`;
    a.click();
  };

  const handleImport = async () => {
    if (!importBlob.trim()) { notify("err", "Paste an export blob"); return; }
    if (!importPw) { notify("err", "Enter the decryption password"); return; }
    try {
      const count = await api.importPersonas(importBlob.trim(), importPw);
      notify("ok", `Imported ${count} new persona(s)`);
      setImportBlob("");
      setImportPw("");
      api.listPersonas().then(setPersonas);
    } catch (e) { notify("err", String(e)); }
  };

  const handleFileLoad = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => setImportBlob(String(ev.target?.result ?? ""));
    reader.readAsText(file);
  };

  return (
    <div className="space-y-6 max-w-2xl">
      <div>
        <h2 className="text-lg font-semibold text-white">Export / Import</h2>
        <p className="text-sm text-gray-400 mt-0.5">
          Personas are encrypted with AES-256-GCM before export. The password is never stored.
        </p>
      </div>

      {status && (
        <div className={`rounded-lg px-4 py-2.5 text-sm ${
          status.kind === "ok"
            ? "bg-green-900/30 border border-green-700/50 text-green-300"
            : "bg-red-900/30 border border-red-700/50 text-red-300"
        }`}>
          {status.msg}
        </div>
      )}

      {/* Export */}
      <section className="bg-panel border border-border rounded-xl p-5 space-y-4">
        <h3 className="font-medium text-white flex items-center gap-2">
          <Download size={16} className="text-indigo-400" /> Export
        </h3>

        <div>
          <label className="label">Select personas (leave all unchecked to export all)</label>
          <div className="space-y-1 mt-1">
            {personas.map((p) => (
              <label key={p.id} className="flex items-center gap-2.5 cursor-pointer group">
                <input
                  type="checkbox"
                  checked={selectedIds.has(p.id)}
                  onChange={() => toggleSelect(p.id)}
                  className="w-4 h-4 accent-indigo-500"
                />
                <span
                  className="w-3 h-3 rounded-full flex-shrink-0"
                  style={{ backgroundColor: p.color }}
                />
                <span className="text-sm text-gray-300 group-hover:text-white">
                  {p.name}
                </span>
                <span className="text-xs text-gray-500">{p.browser_type}</span>
              </label>
            ))}
          </div>
        </div>

        <div>
          <label className="label">Encryption password</label>
          <div className="relative">
            <input
              className="input pr-10"
              type={showPw ? "text" : "password"}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Strong passphrase"
            />
            <button
              type="button"
              onClick={() => setShowPw((s) => !s)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
            >
              {showPw ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
          </div>
        </div>

        <button onClick={handleExport} className="btn-primary flex items-center gap-2">
          <Download size={15} /> Generate Export
        </button>

        {exportBlob && (
          <div className="space-y-2">
            <textarea
              className="input font-mono text-xs resize-none h-28"
              readOnly
              value={exportBlob}
            />
            <div className="flex gap-2">
              <button onClick={handleCopy} className="btn-secondary flex items-center gap-1.5 text-sm">
                {copied ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
                {copied ? "Copied" : "Copy"}
              </button>
              <button onClick={handleSaveFile} className="btn-secondary flex items-center gap-1.5 text-sm">
                <Download size={14} /> Save .enc file
              </button>
            </div>
          </div>
        )}
      </section>

      {/* Import */}
      <section className="bg-panel border border-border rounded-xl p-5 space-y-4">
        <h3 className="font-medium text-white flex items-center gap-2">
          <Upload size={16} className="text-indigo-400" /> Import
        </h3>

        <div>
          <label className="label">Load from file</label>
          <input
            type="file"
            accept=".enc,.txt"
            onChange={handleFileLoad}
            className="text-sm text-gray-400 file:mr-3 file:py-1.5 file:px-3 file:rounded file:border-0 file:bg-white/10 file:text-white file:text-sm hover:file:bg-white/20 cursor-pointer"
          />
        </div>

        <div>
          <label className="label">Or paste export blob</label>
          <textarea
            className="input font-mono text-xs resize-none h-28"
            value={importBlob}
            onChange={(e) => setImportBlob(e.target.value)}
            placeholder="Paste base64-encoded export here…"
          />
        </div>

        <div>
          <label className="label">Decryption password</label>
          <div className="relative">
            <input
              className="input pr-10"
              type={showImportPw ? "text" : "password"}
              value={importPw}
              onChange={(e) => setImportPw(e.target.value)}
              placeholder="Password used during export"
            />
            <button
              type="button"
              onClick={() => setShowImportPw((s) => !s)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
            >
              {showImportPw ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
          </div>
        </div>

        <button onClick={handleImport} className="btn-primary flex items-center gap-2">
          <Upload size={15} /> Import Personas
        </button>
      </section>
    </div>
  );
}

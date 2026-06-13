import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import { api } from "../api";
import { PersonaCard } from "../components/PersonaCard";
import { PersonaForm } from "../components/PersonaForm";
import { LeakCheckPanel } from "../components/LeakCheckPanel";
import type { Persona, CreatePersona } from "../types";

export function PersonasPage() {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editTarget, setEditTarget] = useState<Persona | null>(null);
  const [checkTarget, setCheckTarget] = useState<Persona | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const [loadError, setLoadError] = useState<string | null>(null);

  const load = () => {
    setLoadError(null);
    api.listPersonas()
      .then(setPersonas)
      .catch((e) => setLoadError(String(e)))
      .finally(() => setLoading(false));
  };

  useEffect(() => { load(); }, []);

  const notify = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  const handleSave = async (data: CreatePersona) => {
    if (editTarget) {
      await api.updatePersona({ id: editTarget.id, ...data });
      notify("Persona updated.");
    } else {
      await api.createPersona(data);
      notify("Persona created.");
    }
    setShowForm(false);
    setEditTarget(null);
    load();
  };

  const handleDelete = async (p: Persona) => {
    if (!confirm(`Delete persona "${p.name}"? This cannot be undone.`)) return;
    await api.deletePersona(p.id);
    notify("Persona deleted.");
    load();
  };

  const handleLaunch = async (p: Persona) => {
    try {
      const msg = await api.launchBrowser(p.id);
      notify(msg);
    } catch (e) {
      notify(`Error: ${e}`);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-white">Personas</h2>
          <p className="text-sm text-gray-400">Each persona is an isolated browser identity with its own proxy, fingerprint, and storage.</p>
        </div>
        <button
          onClick={() => { setEditTarget(null); setShowForm(true); }}
          className="btn-primary flex items-center gap-2"
        >
          <Plus size={16} /> New Persona
        </button>
      </div>

      {loadError && (
        <div className="bg-red-900/30 border border-red-700/50 rounded-lg p-4 text-red-300 text-sm font-mono">
          IPC error: {loadError}
        </div>
      )}

      {loading && (
        <div className="text-center text-gray-500 py-12">Loading...</div>
      )}

      {!loading && personas.length === 0 && (
        <div className="text-center py-16 space-y-3">
          <p className="text-gray-400">No personas yet.</p>
          <button
            onClick={() => setShowForm(true)}
            className="btn-primary mx-auto flex items-center gap-2"
          >
            <Plus size={16} /> Create your first persona
          </button>
        </div>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {personas.map((p) => (
          <PersonaCard
            key={p.id}
            persona={p}
            onEdit={() => { setEditTarget(p); setShowForm(true); }}
            onDelete={() => handleDelete(p)}
            onLaunch={() => handleLaunch(p)}
            onCheck={() => setCheckTarget(p)}
            onNotify={notify}
          />
        ))}
      </div>

      {showForm && (
        <PersonaForm
          persona={editTarget ?? undefined}
          onSave={handleSave}
          onClose={() => { setShowForm(false); setEditTarget(null); }}
        />
      )}

      {checkTarget && (
        <LeakCheckPanel
          persona={checkTarget}
          onClose={() => setCheckTarget(null)}
        />
      )}

      {toast && (
        <div className="fixed bottom-6 right-6 bg-indigo-600 text-white px-4 py-2.5 rounded-lg shadow-lg text-sm animate-fade-in">
          {toast}
        </div>
      )}
    </div>
  );
}

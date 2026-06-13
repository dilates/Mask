import { useState } from "react";
import { Shield, Users, BookOpen, ArrowLeftRight } from "lucide-react";
import { PersonasPage } from "./pages/PersonasPage";
import { AuditLog } from "./components/AuditLog";
import { ExportImportPage } from "./pages/ExportImportPage";

type Tab = "personas" | "audit" | "export";

export default function App() {
  const [tab, setTab] = useState<Tab>("personas");

  return (
    <div className="min-h-screen bg-surface text-white flex flex-col">
      <header
        className="border-b border-border bg-panel px-6 py-3 flex items-center gap-3 select-none"
        data-tauri-drag-region
      >
        <Shield size={20} className="text-indigo-400" />
        <span className="font-bold text-white tracking-wide">MASK</span>
        <span className="text-gray-600 text-sm">Persona Manager</span>
      </header>

      <div className="flex flex-1 overflow-hidden">
        <nav className="w-48 bg-panel border-r border-border flex-shrink-0 py-4 flex flex-col">
          <NavItem
            icon={<Users size={16} />}
            label="Personas"
            active={tab === "personas"}
            onClick={() => setTab("personas")}
          />
          <NavItem
            icon={<ArrowLeftRight size={16} />}
            label="Export / Import"
            active={tab === "export"}
            onClick={() => setTab("export")}
          />
          <NavItem
            icon={<BookOpen size={16} />}
            label="Audit Log"
            active={tab === "audit"}
            onClick={() => setTab("audit")}
          />
        </nav>

        <main className="flex-1 overflow-y-auto p-6">
          {tab === "personas" && <PersonasPage />}
          {tab === "export" && <ExportImportPage />}
          {tab === "audit" && <AuditLog />}
        </main>
      </div>
    </div>
  );
}

function NavItem({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2.5 px-4 py-2.5 text-sm text-left transition-colors w-full ${
        active
          ? "bg-indigo-600/20 text-indigo-300 border-r-2 border-indigo-500"
          : "text-gray-400 hover:text-white hover:bg-white/5"
      }`}
    >
      {icon}
      {label}
    </button>
  );
}

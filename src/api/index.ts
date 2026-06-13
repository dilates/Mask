import { invoke } from "@tauri-apps/api/core";
import type {
  Persona, CreatePersona, UpdatePersona,
  AuditEntry, LeakCheckResult, WgStatus,
} from "../types";

export const api = {
  // Persona CRUD
  listPersonas: () => invoke<Persona[]>("list_personas"),
  createPersona: (input: CreatePersona) => invoke<Persona>("create_persona", { input }),
  updatePersona: (input: UpdatePersona) => invoke<Persona>("update_persona", { input }),
  deletePersona: (id: string) => invoke<void>("delete_persona", { id }),

  // Browser
  launchBrowser: (personaId: string) => invoke<string>("launch_browser", { personaId }),
  getProfilePath: (personaId: string) => invoke<string>("get_profile_path", { personaId }),
  detectBrowsers: () => invoke<string[]>("detect_browsers"),

  // Leak check
  runLeakCheck: (params: {
    personaId: string;
    expectedTimezone: string;
    proxyType?: string;
    proxyHost?: string;
    proxyPort?: number;
  }) =>
    invoke<LeakCheckResult>("run_leak_check", {
      personaId: params.personaId,
      expectedTimezone: params.expectedTimezone,
      proxyType: params.proxyType ?? null,
      proxyHost: params.proxyHost ?? null,
      proxyPort: params.proxyPort ?? null,
    }),

  // Audit
  getAuditLog: (limit?: number) => invoke<AuditEntry[]>("get_audit_log", { limit: limit ?? 200 }),
  getDataDir: () => invoke<string>("get_data_dir"),

  // WireGuard
  wgUp: (personaId: string) => invoke<WgStatus>("wg_up", { personaId }),
  wgDown: (personaId: string) => invoke<WgStatus>("wg_down", { personaId }),
  wgStatus: (personaId: string) => invoke<WgStatus>("wg_status", { personaId }),

  // Export / Import
  exportPersonas: (personaIds: string[], password: string) =>
    invoke<string>("export_personas", { personaIds, password }),
  importPersonas: (blob: string, password: string) =>
    invoke<number>("import_personas", { blob, password }),

  // Container
  launchContainerBrowser: (personaId: string) =>
    invoke<string>("launch_container_browser", { personaId }),
  stopContainer: (personaId: string) => invoke<string>("stop_container", { personaId }),
  detectContainerRuntimes: () => invoke<string[]>("detect_container_runtimes"),
};

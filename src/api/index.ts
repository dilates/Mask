import { invoke } from "@tauri-apps/api/core";
import type {
  Persona,
  CreatePersona,
  UpdatePersona,
  AuditEntry,
  LeakCheckResult,
} from "../types";

export const api = {
  listPersonas: () => invoke<Persona[]>("list_personas"),
  createPersona: (input: CreatePersona) =>
    invoke<Persona>("create_persona", { input }),
  updatePersona: (input: UpdatePersona) =>
    invoke<Persona>("update_persona", { input }),
  deletePersona: (id: string) => invoke<void>("delete_persona", { id }),
  launchBrowser: (personaId: string) =>
    invoke<string>("launch_browser", { personaId }),
  getProfilePath: (personaId: string) =>
    invoke<string>("get_profile_path", { personaId }),
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
  getAuditLog: (limit?: number) =>
    invoke<AuditEntry[]>("get_audit_log", { limit: limit ?? 200 }),
  getDataDir: () => invoke<string>("get_data_dir"),
};

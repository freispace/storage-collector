import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Types ────────────────────────────────────────────────────────────────────

export interface LogEntry {
  id: string;
  level: string;
  message: string;
  context: string | null;
  created_at: string;
}

export interface FolderConfig {
  id: string;
  storage_id: string;
  project_id: string;
  folder_path: string;
  custom_schedule: string | null;
  created_at: string;
  updated_at: string;
}

export interface FolderConfigInput {
  storage_id: string;
  project_id: string;
  folder_path: string;
  custom_schedule: string | null;
}

export interface FreispaceProject {
  id: string;
  name: string | null;
}

export interface FreispaceStorage {
  id: string;
  name: string | null;
  is_active: boolean | null;
}

export interface StorageProjectItem {
  id: string;
  storage_id: string | null;
  project_id: string | null;
  storage_size_estimated: number | null;
  storage_size_current: number | null;
}

export interface PaginationLinks {
  first: string | null;
  last: string | null;
  prev: string | null;
  next: string | null;
}

export interface PaginationMeta {
  current_page: number;
  from: number | null;
  per_page: number;
  to: number | null;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination_links: PaginationLinks | null;
  pagination_meta: PaginationMeta | null;
}

export interface SchedulerTickPayload {
  storage_id: string;
  project_id: string;
  status: string;
}

export interface CachedEntityName {
  entity_type: string;
  entity_id: string;
  name: string | null;
}

export interface StorageProjectSetting {
  storage_id: string;
  project_id: string;
  enabled: boolean;
}

// ── Commands ─────────────────────────────────────────────────────────────────

export const api = {
  getApiKey: () =>
    invoke<string>("get_api_key"),

  setApiKey: (key: string) =>
    invoke<null>("set_api_key", { key }),

  getGlobalSchedule: () =>
    invoke<string>("get_global_schedule"),

  setGlobalSchedule: (time: string) =>
    invoke<null>("set_global_schedule", { time }),

  getSchedulerAutoRun: () =>
    invoke<boolean>("get_scheduler_auto_run"),

  setSchedulerAutoRun: (enabled: boolean) =>
    invoke<null>("set_scheduler_auto_run", { enabled }),

  getLaunchAtStartup: () =>
    invoke<boolean>("get_launch_at_startup"),

  setLaunchAtStartup: (enabled: boolean) =>
    invoke<null>("set_launch_at_startup", { enabled }),

  showLogsWindow: () =>
    invoke<null>("show_logs_window"),

  fetchProjectsPage: (page: number) =>
    invoke<PaginatedResponse<FreispaceProject>>("fetch_projects_page", { page }),

  fetchStoragesPage: (page: number) =>
    invoke<PaginatedResponse<FreispaceStorage>>("fetch_storages_page", { page }),

  fetchStorageProjectsPage: (page: number) =>
    invoke<PaginatedResponse<StorageProjectItem>>("fetch_storage_projects_page", { page }),

  listFolderConfigs: () =>
    invoke<FolderConfig[]>("list_folder_configs"),

  upsertFolderConfig: (input: FolderConfigInput) =>
    invoke<FolderConfig>("upsert_folder_config", { input }),

  deleteFolderConfig: (id: string) =>
    invoke<null>("delete_folder_config", { id }),

  pickFolder: () =>
    invoke<string | null>("pick_folder"),

  triggerAll: () =>
    invoke<null>("trigger_all"),

  triggerStorageProject: (storageId: string, projectId: string) =>
    invoke<null>("trigger_storage_project", { storageId, projectId }),

  listLogEntries: (levelFilter: string | null, limit: number, offset: number) =>
    invoke<LogEntry[]>("list_log_entries", { levelFilter, limit, offset }),

  clearLogEntries: () =>
    invoke<null>("clear_log_entries"),

  saveLogFile: (content: string, defaultName: string) =>
    invoke<boolean>("save_log_file", { content, defaultName }),

  syncEntityNames: () =>
    invoke<null>("sync_entity_names"),

  getEntityNames: () =>
    invoke<CachedEntityName[]>("get_entity_names"),

  listStorageProjectSettings: () =>
    invoke<StorageProjectSetting[]>("list_storage_project_settings"),

  setStorageProjectEnabled: (storageId: string, projectId: string, enabled: boolean) =>
    invoke<null>("set_storage_project_enabled", { storageId, projectId, enabled }),

  removeStorageProject: (storageId: string, projectId: string) =>
    invoke<null>("remove_storage_project", { storageId, projectId }),
};

// ── Events ───────────────────────────────────────────────────────────────────

export function onLogEntry(cb: (entry: LogEntry) => void): Promise<UnlistenFn> {
  return listen<LogEntry>("log_entry", (event) => cb(event.payload));
}

export function onSchedulerTick(
  cb: (payload: SchedulerTickPayload) => void
): Promise<UnlistenFn> {
  return listen<SchedulerTickPayload>("scheduler_tick", (event) => cb(event.payload));
}

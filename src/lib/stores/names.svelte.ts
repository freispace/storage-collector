import { api } from "$lib/api";

export interface ProjectMeta {
  parent_id: string | null;
  project_number: string | null;
  color: string | null;
}

function createNamesStore() {
  let names = $state(new Map<string, string>());
  let projectMeta = $state(new Map<string, ProjectMeta>());

  return {
    get names() {
      return names;
    },
    get projectMeta() {
      return projectMeta;
    },
    lookup(id: string): string | null {
      return names.get(id.toLowerCase()) ?? names.get(id) ?? null;
    },
    lookupProjectMeta(id: string): ProjectMeta | null {
      return projectMeta.get(id.toLowerCase()) ?? projectMeta.get(id) ?? null;
    },
    async load() {
      try {
        const entities = await api.getEntityNames();
        const map = new Map<string, string>();
        const projectMap = new Map<string, ProjectMeta>();
        for (const e of entities) {
          if (e.name) map.set(e.entity_id, e.name);
          if (e.entity_type === "project") {
            projectMap.set(e.entity_id, {
              parent_id: e.parent_id,
              project_number: e.project_number,
              color: e.color,
            });
          }
        }
        names = map;
        projectMeta = projectMap;
      } catch {
        // names are best-effort — ignore errors
      }
    },
  };
}

export const namesStore = createNamesStore();

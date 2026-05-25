import { api } from "$lib/api";

function createNamesStore() {
  let names = $state(new Map<string, string>());

  return {
    get names() {
      return names;
    },
    lookup(id: string): string | null {
      return names.get(id.toLowerCase()) ?? names.get(id) ?? null;
    },
    async load() {
      try {
        const entities = await api.getEntityNames();
        const map = new Map<string, string>();
        for (const e of entities) {
          if (e.name) map.set(e.entity_id, e.name);
        }
        names = map;
      } catch {
        // names are best-effort — ignore errors
      }
    },
  };
}

export const namesStore = createNamesStore();

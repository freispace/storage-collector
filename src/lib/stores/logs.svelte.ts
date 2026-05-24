import type { LogEntry } from "$lib/api";

function createLogsStore() {
  let entries = $state<LogEntry[]>([]);
  let levelFilter = $state<string | null>(null);

  return {
    get entries() {
      return entries;
    },
    get levelFilter() {
      return levelFilter;
    },
    get filteredEntries() {
      if (!levelFilter) return entries;
      return entries.filter((e) => e.level === levelFilter);
    },
    setFilter(filter: string | null) {
      levelFilter = filter;
    },
    addEntry(entry: LogEntry) {
      entries = [entry, ...entries].slice(0, 500);
    },
    setEntries(newEntries: LogEntry[]) {
      entries = newEntries;
    },
    clear() {
      entries = [];
    },
  };
}

export const logsStore = createLogsStore();

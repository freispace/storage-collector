import { api } from "$lib/api";

function createSettingsStore() {
  let apiKey = $state("");
  let globalSchedule = $state("17:55");
  let schedulerAutoRun = $state(true);
  let loaded = $state(false);
  let error = $state<string | null>(null);

  return {
    get apiKey() {
      return apiKey;
    },
    get globalSchedule() {
      return globalSchedule;
    },
    get schedulerAutoRun() {
      return schedulerAutoRun;
    },
    get loaded() {
      return loaded;
    },
    get error() {
      return error;
    },
    setApiKey(key: string) {
      apiKey = key;
    },
    setGlobalSchedule(time: string) {
      globalSchedule = time;
    },
    setSchedulerAutoRun(val: boolean) {
      schedulerAutoRun = val;
    },
    async load() {
      try {
        [apiKey, globalSchedule, schedulerAutoRun] = await Promise.all([
          api.getApiKey(),
          api.getGlobalSchedule(),
          api.getSchedulerAutoRun(),
        ]);
        loaded = true;
      } catch (e) {
        error = String(e);
      }
    },
  };
}

export const settingsStore = createSettingsStore();

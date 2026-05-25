import { api } from "$lib/api";

function createSettingsStore() {
  let apiKey = $state("");
  let globalSchedule = $state("17:55");
  let schedulerAutoRun = $state(true);
  let launchAtStartup = $state(false);
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
    get launchAtStartup() {
      return launchAtStartup;
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
    setLaunchAtStartup(val: boolean) {
      launchAtStartup = val;
    },
    async load() {
      try {
        [apiKey, globalSchedule, schedulerAutoRun, launchAtStartup] = await Promise.all([
          api.getApiKey(),
          api.getGlobalSchedule(),
          api.getSchedulerAutoRun(),
          api.getLaunchAtStartup(),
        ]);
        loaded = true;
      } catch (e) {
        error = String(e);
      }
    },
  };
}

export const settingsStore = createSettingsStore();

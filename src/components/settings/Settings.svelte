<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import ApiKeyInput from "./ApiKeyInput.svelte";
  import GlobalSchedule from "./GlobalSchedule.svelte";
  import StorageProjectList from "./StorageProjectList.svelte";
  import LoadingText from "../general/loadingText.svelte";

  let activeTab = $state<"settings" | "projects" | "info">("settings");
  let running = $state(false);
  let runError = $state<string | null>(null);

  onMount(() => {
    settingsStore.load();
  });

  async function triggerAll() {
    running = true;
    runError = null;
    try {
      await api.triggerAll();
    } catch (e) {
      runError = String(e);
    } finally {
      running = false;
    }
  }
</script>

<div class="flex flex-col h-full p-2">
  <div class="tabs tabs-lift shrink-0 h-full">
    <button
      class="tab {activeTab === 'settings' ? 'tab-active' : ''}"
      onclick={() => (activeTab = "settings")}
    >
      Settings
    </button>
    <div class="tab-content bg-base-100 border-base-300">
      {#if !settingsStore.loaded}
        <div
          class="flex-1 flex items-center justify-center text-gray-500 font-medium text-xl"
        >
          <span class="loading loading-infinity loading-xl me-1"></span> Loading…
        </div>
      {:else if activeTab === "settings"}
        <div class="flex-1 overflow-y-auto space-y-5 p-6">
          <ApiKeyInput
            apiKey={settingsStore.apiKey}
            onChange={(key) => settingsStore.setApiKey(key)}
          />

          <div class="divider -mx-6"></div>

          <GlobalSchedule
            schedule={settingsStore.globalSchedule}
            autoRun={settingsStore.schedulerAutoRun}
            launchAtStartup={settingsStore.launchAtStartup}
            onScheduleChange={(t) => settingsStore.setGlobalSchedule(t)}
            onAutoRunChange={(v) => settingsStore.setSchedulerAutoRun(v)}
            onLaunchAtStartupChange={(v) => settingsStore.setLaunchAtStartup(v)}
          />

          <div class="divider -mx-6"></div>

          <div>
            <div class="flex items-center gap-3">
              <button
                class="btn btn-success btn-md"
                onclick={triggerAll}
                disabled={running}
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 16 16"
                  fill="currentColor"
                  class="size-4"
                >
                  <path
                    d="M3 3.732a1.5 1.5 0 0 1 2.305-1.265l6.706 4.267a1.5 1.5 0 0 1 0 2.531l-6.706 4.268A1.5 1.5 0 0 1 3 12.267V3.732Z"
                  />
                </svg>
                <span>Run all now</span>
              </button>
              {#if running}
                <span class="loading loading-spinner loading-xs"></span>
              {/if}
            </div>
            {#if runError}
              <p class="text-xs text-red-400 mt-2">{runError}</p>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <button
      class="tab {activeTab === 'projects' ? 'tab-active' : ''}"
      onclick={() => (activeTab = "projects")}
    >
      Storage Projects
    </button>
    <div class="tab-content bg-base-100 border-base-300">
      {#if !settingsStore.loaded}
        <div
          class="flex-1 flex items-center justify-center text-gray-500 font-medium text-xl"
        >
          <LoadingText />
        </div>
      {:else}
        <StorageProjectList
          globalSchedule={settingsStore.globalSchedule}
          apiKey={settingsStore.apiKey}
        />
      {/if}
    </div>

    <button
      class="tab {activeTab === 'info' ? 'tab-active' : ''}"
      onclick={() => (activeTab = "info")}
    >
      Info
    </button>
    <div class="tab-content bg-base-100 border-base-300 p-6">
      <svg
        class="max-w-48 mt-4 mb-6"
        xmlns="http://www.w3.org/2000/svg"
        xmlns:xlink="http://www.w3.org/1999/xlink"
        viewBox="0 0 2339 422"
        fill-rule="evenodd"
        stroke-linejoin="round"
        stroke-miterlimit="2"
        ><g fill="#fff"
          ><g fill-rule="nonzero"
            ><path
              d="M588.368 135.342h-45.042V320.6h-35.642V135.342h-27.808v-29.375h27.808V90.691c0-23.892 6.267-41.517 18.408-52.875 12.533-10.967 32.508-16.45 59.925-16.45v29.767c-15.667 0-26.633 3.133-32.9 9.008-6.658 6.267-9.792 16.45-9.792 30.55v15.275h45.042v29.375zm56.4 5.483c6.267-12.142 15.275-21.933 26.633-28.592 11.75-6.658 25.85-10.183 42.692-10.183v36.817h-9.4c-39.95 0-59.925 21.542-59.925 65.017V320.6h-35.642V105.966h35.642v34.858z"
            /><use xlink:href="#B" /><path
              d="M974.943 71.108c-6.658 0-12.533-2.35-17.233-7.05s-7.05-10.575-7.05-17.233 2.35-12.533 7.05-17.233 10.575-7.05 17.233-7.05 12.142 2.35 16.842 7.05c4.308 4.7 6.658 10.575 6.658 17.233s-2.35 12.533-6.658 17.233c-4.7 4.7-10.183 7.05-16.842 7.05zm17.233 34.858V320.6h-35.642V105.966h35.642zm337.634 217.717c-16.45 0-31.333-2.742-44.258-8.617-12.925-5.483-23.5-13.317-30.942-23.5-7.442-9.792-11.75-21.542-12.533-34.467h36.817c1.175 10.575 6.267 19.583 15.275 26.242s20.758 10.183 35.25 10.183c13.708 0 24.283-3.133 32.117-9.008s11.75-13.708 11.75-22.717c0-9.4-4.308-16.45-12.533-21.15-8.225-4.308-21.15-9.008-38.775-13.317-16.058-4.308-28.983-8.617-38.775-12.925-10.183-4.308-18.8-10.575-25.85-19.192-7.442-8.225-10.967-19.583-10.967-33.292 0-10.967 3.133-21.15 9.792-30.158s15.667-16.45 27.808-21.933c12.142-5.092 25.85-7.833 41.125-7.833 23.892 0 43.083 5.875 57.575 18.017s22.325 28.592 23.5 49.35h-35.642c-.783-11.358-5.092-20.367-13.317-27.025s-19.583-10.183-33.292-10.183c-12.925 0-23.108 2.742-30.55 8.225s-11.358 12.533-11.358 21.542c0 7.05 2.35 12.925 7.05 17.233 4.308 4.7 10.183 8.225 17.233 10.967 6.658 2.742 16.45 5.875 28.592 9.4 15.275 4.308 27.808 8.225 37.6 12.142 9.792 4.308 18.017 10.183 25.067 18.408 6.658 8.225 10.183 18.8 10.575 31.725 0 11.75-3.133 22.325-9.792 31.725s-15.667 16.842-27.417 21.933c-12.142 5.483-25.85 8.225-41.125 8.225zm143.349-178.6c7.05-12.142 17.625-22.325 31.725-30.942 13.708-8.225 30.158-12.142 48.567-12.142 19.192 0 36.425 4.7 51.7 13.708 15.667 9.008 27.808 21.933 36.817 38.383 9.008 16.842 13.317 36.033 13.317 57.967 0 21.542-4.308 41.125-13.317 57.967s-21.15 30.158-36.817 39.558c-15.275 9.4-32.508 14.1-51.7 14.1-18.408 0-34.467-4.308-48.175-12.533-14.1-7.833-24.675-18.408-32.117-30.55v141.392h-35.642V105.524h35.642v39.558zm145.7 66.975c0-16.058-3.133-30.158-9.792-42.3s-15.275-21.15-26.242-27.417c-11.358-6.267-23.5-9.4-36.817-9.4-12.925 0-25.067 3.133-36.033 9.4-11.358 6.658-19.975 16.058-26.633 27.808-6.658 12.142-10.183 26.242-10.183 42.3s3.525 30.158 10.183 42.3 15.275 21.542 26.633 27.808c10.967 6.658 23.108 9.792 36.033 9.792 13.317 0 25.458-3.133 36.817-9.792 10.967-6.267 19.583-15.667 26.242-27.808s9.792-26.242 9.792-42.692zm49.351 0c0-21.933 4.308-41.125 13.317-57.967 9.008-16.45 21.15-29.375 36.817-38.383 15.275-9.008 32.508-13.708 51.7-13.708 18.8 0 35.25 3.917 48.958 12.142s24.283 18.408 30.942 30.55v-39.167h36.033v214.634h-36.033v-39.95c-7.05 12.533-17.625 23.108-31.333 30.942-14.1 8.225-30.55 12.533-48.958 12.533-19.192 0-36.425-4.7-51.7-14.1s-27.417-22.717-36.425-39.558-13.317-36.425-13.317-57.967zm181.733.392c0-16.058-3.133-30.158-9.792-42.3s-15.275-21.15-26.633-27.808c-10.967-6.267-23.108-9.4-36.425-9.4s-25.458 3.133-36.425 9.4-19.583 15.275-26.242 27.417-9.792 26.242-9.792 42.3c0 16.45 3.133 30.55 9.792 42.692s15.275 21.542 26.242 27.808c10.967 6.658 23.108 9.792 36.425 9.792s25.458-3.133 36.425-9.792c11.358-6.267 19.975-15.667 26.633-27.808s9.792-26.242 9.792-42.3zm62.275 0c0-22.325 4.308-41.517 13.317-58.358 9.008-16.45 21.15-29.375 37.208-38.383 15.667-9.008 33.683-13.708 54.05-13.708 26.242 0 48.175 6.267 65.017 19.192 17.233 12.925 28.592 30.55 34.075 53.267h-38.383c-3.525-12.925-10.575-23.5-21.542-30.942-10.183-7.442-23.5-11.358-39.167-11.358-20.367 0-36.817 7.05-49.35 20.758-12.533 14.1-18.8 34.075-18.8 59.533 0 25.85 6.267 45.825 18.8 59.925s28.983 21.15 49.35 21.15c15.667 0 28.592-3.525 39.167-10.967s17.625-17.625 21.542-31.333h38.383c-5.875 21.933-17.233 39.558-34.467 52.483-17.233 13.317-38.775 19.975-64.625 19.975-20.367 0-38.383-4.7-54.05-13.708-16.058-9.008-28.2-21.933-37.208-38.775s-13.317-36.425-13.317-58.75z"
            /><use xlink:href="#B" x="1408.058" y="-0.442" /></g
          ><path
            d="M509.068 356.001h733v32h-733zM388-.001H0v388h388v-388zm-155.73 44.53v30.4H90.54v237.6h31.73v30.93H55.34V44.529h176.93zm100.4 298.93H155.74v-30.93h141.73v-237.6h-31.73v-30.4h66.93v298.93z"
          /></g
        ><defs
          ><path
            id="B"
            d="M930.293 205.058c0 6.658-.392 14.1-1.175 21.542h-171.55c1.175 21.15 8.617 37.6 21.933 49.35 12.925 12.142 28.983 18.017 47.783 18.017 15.275 0 28.2-3.525 38.383-10.967 10.575-7.05 17.625-16.45 21.933-28.592h38.383c-5.875 20.758-17.233 37.208-34.467 50.133s-38.775 19.583-64.233 19.583c-20.367 0-38.383-4.7-54.833-13.708-15.667-9.008-28.592-22.325-37.6-39.167s-13.708-36.033-13.708-58.358 4.308-41.517 13.317-58.358 21.542-29.767 37.208-38.775c16.45-9.008 34.858-13.317 55.617-13.317 20.367 0 38.383 4.308 54.05 13.317s27.808 21.15 36.425 36.425c8.225 15.667 12.533 33.292 12.533 52.875zm-36.817-7.442c0-13.708-3.133-25.458-9.008-35.25s-14.1-17.233-24.283-22.325c-10.575-4.7-21.933-7.442-34.467-7.442-18.017 0-33.292 5.875-45.825 17.233-12.925 11.358-19.975 27.417-21.933 47.783h135.517z"
          /></defs
        ></svg
      >
      <p>Version 0.1</p>
      <p>© Leonardo Re, freispace GmbH</p>
      <button class="btn mt-4" onclick={() => api.showLogsWindow()}
        >Show logs</button
      >
    </div>
  </div>
</div>

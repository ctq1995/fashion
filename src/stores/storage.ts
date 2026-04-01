import { defineStore } from 'pinia';
import { ref } from 'vue';
import { openPath } from '@tauri-apps/plugin-opener';
import {
  chooseDirectory,
  getStoragePreferences,
  getStorageSnapshot,
  isPersistenceBackendAvailable,
  type StoragePreferences,
  updateDataDirectory,
  updateDownloadDirectory,
} from '@/utils/persistence';

type StorageTarget = 'data' | 'download' | null;

function clonePreferences(): StoragePreferences {
  const preferences = getStoragePreferences();
  return { ...preferences };
}

export const useStorageStore = defineStore('storage', () => {
  const supported = ref(isPersistenceBackendAvailable());
  const preferences = ref<StoragePreferences>(clonePreferences());
  const busyTarget = ref<StorageTarget>(null);
  const errorMessage = ref('');

  function syncPreferences(next?: StoragePreferences) {
    preferences.value = next ? { ...next } : clonePreferences();
  }

  async function chooseDataDirectory() {
    if (!supported.value || busyTarget.value) return;

    const selected = await chooseDirectory(
      preferences.value.dataDirectory ?? preferences.value.effectiveDataDirectory,
    );
    if (!selected) return;

    busyTarget.value = 'data';
    errorMessage.value = '';
    try {
      const next = await updateDataDirectory(selected);
      syncPreferences(next);
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      busyTarget.value = null;
    }
  }

  async function resetDataDirectory() {
    if (!supported.value || busyTarget.value) return;

    busyTarget.value = 'data';
    errorMessage.value = '';
    try {
      const next = await updateDataDirectory(null);
      syncPreferences(next);
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      busyTarget.value = null;
    }
  }

  async function chooseDownloadDirectory() {
    if (!supported.value || busyTarget.value) return;

    const selected = await chooseDirectory(
      preferences.value.downloadDirectory ?? preferences.value.effectiveDownloadDirectory,
    );
    if (!selected) return;

    busyTarget.value = 'download';
    errorMessage.value = '';
    try {
      const next = await updateDownloadDirectory(selected);
      syncPreferences(next);
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      busyTarget.value = null;
    }
  }

  async function resetDownloadDirectory() {
    if (!supported.value || busyTarget.value) return;

    busyTarget.value = 'download';
    errorMessage.value = '';
    try {
      const next = await updateDownloadDirectory(null);
      syncPreferences(next);
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      busyTarget.value = null;
    }
  }

  async function openDataDirectory() {
    if (!preferences.value.effectiveDataDirectory) return;
    await openPath(preferences.value.effectiveDataDirectory);
  }

  async function openDownloadDirectory() {
    if (!preferences.value.effectiveDownloadDirectory) return;
    await openPath(preferences.value.effectiveDownloadDirectory);
  }

  return {
    supported,
    preferences,
    busyTarget,
    errorMessage,
    chooseDataDirectory,
    resetDataDirectory,
    chooseDownloadDirectory,
    resetDownloadDirectory,
    openDataDirectory,
    openDownloadDirectory,
    syncPreferences,
    getStorageSnapshot,
  };
});

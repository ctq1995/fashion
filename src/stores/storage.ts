import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { clearMusicApiTransientState } from '@/api/music';
import { useMediaStore } from '@/stores/media';
import {
  clearManagedStoragePrefixes,
  chooseDirectory,
  getStoragePreferences,
  getStorageSnapshot,
  isPersistenceBackendAvailable,
  type StoragePreferences,
  updateDataDirectory,
  updateDownloadDirectory,
} from '@/utils/persistence';

type StorageTarget = 'data' | 'download' | 'cache' | null;

interface CacheClearResult {
  filesRemoved: number;
  bytesFreed: number;
}

function clonePreferences(): StoragePreferences {
  const preferences = getStoragePreferences();
  return { ...preferences };
}

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB'] as const;
  let value = bytes;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  const digits = value >= 10 || unitIndex === 0 ? 0 : 1;
  return `${value.toFixed(digits)} ${units[unitIndex]}`;
}

export const useStorageStore = defineStore('storage', () => {
  const supported = ref(isPersistenceBackendAvailable());
  const preferences = ref<StoragePreferences>(clonePreferences());
  const busyTarget = ref<StorageTarget>(null);
  const errorMessage = ref('');
  const cacheMessage = ref('');

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
    cacheMessage.value = '';
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
    cacheMessage.value = '';
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
    cacheMessage.value = '';
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
    cacheMessage.value = '';
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

  async function clearCache() {
    if (busyTarget.value) return;

    busyTarget.value = 'cache';
    errorMessage.value = '';
    cacheMessage.value = '';

    try {
      const fileResult = supported.value
        ? await invoke<CacheClearResult>('clear_cached_audio_files')
        : { filesRemoved: 0, bytesFreed: 0 };
      const storageEntriesRemoved = await clearManagedStoragePrefixes(['fashion:cache:']);

      clearMusicApiTransientState();
      useMediaStore().clearRuntimeCoverCache();

      const messageParts: string[] = [];
      if (fileResult.filesRemoved > 0) {
        messageParts.push(`音频缓存 ${fileResult.filesRemoved} 个文件 (${formatBytes(fileResult.bytesFreed)})`);
      }
      if (storageEntriesRemoved > 0) {
        messageParts.push(`本地缓存 ${storageEntriesRemoved} 条`);
      }

      cacheMessage.value = messageParts.length
        ? `已清理 ${messageParts.join('，')}`
        : '没有可清理的缓存';
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      busyTarget.value = null;
    }
  }

  return {
    supported,
    preferences,
    busyTarget,
    errorMessage,
    cacheMessage,
    chooseDataDirectory,
    resetDataDirectory,
    chooseDownloadDirectory,
    resetDownloadDirectory,
    openDataDirectory,
    openDownloadDirectory,
    clearCache,
    syncPreferences,
    getStorageSnapshot,
  };
});

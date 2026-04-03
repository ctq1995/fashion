import { invoke } from '@tauri-apps/api/core';

export interface StorageEnvelope<T> {
  version: number;
  updatedAt: number;
  data: T;
}

export interface StoragePreferences {
  dataDirectory: string | null;
  downloadDirectory: string | null;
  effectiveDataDirectory: string;
  effectiveDownloadDirectory: string;
  usesDefaultDataDirectory: boolean;
  usesDefaultDownloadDirectory: boolean;
}

interface ReadStorageOptions<T> {
  fallback: T;
  validate?: (value: unknown) => value is T;
  migrateLegacy?: (raw: string) => T | null;
}

interface PersistenceBootstrap {
  entries: Record<string, string>;
  preferences: StoragePreferences;
}

const STORAGE_KEY_PREFIX = 'fashion:';

function matchesManagedStorageKey(key: string) {
  return key.startsWith(STORAGE_KEY_PREFIX);
}

function normalizeStorageKey(key: string) {
  return key;
}

function matchesAnyStoragePrefix(key: string, prefixes: string[]) {
  return prefixes.some((prefix) => key.startsWith(prefix));
}

function storageKeyCandidates(key: string) {
  return [key];
}

let backendInitialized = false;
let backendEntries = new Map<string, string>();
let storagePreferences: StoragePreferences = {
  dataDirectory: null,
  downloadDirectory: null,
  effectiveDataDirectory: '',
  effectiveDownloadDirectory: '',
  usesDefaultDataDirectory: true,
  usesDefaultDownloadDirectory: true,
};

function isEnvelope<T>(value: unknown): value is StorageEnvelope<T> {
  if (!value || typeof value !== 'object') return false;
  const envelope = value as Record<string, unknown>;
  return (
    typeof envelope.version === 'number' &&
    typeof envelope.updatedAt === 'number' &&
    'data' in envelope
  );
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

function collectLocalEntries(): Record<string, string> {
  if (typeof window === 'undefined') return {};

  const entries: Record<string, string> = {};
  for (let index = 0; index < window.localStorage.length; index++) {
    const key = window.localStorage.key(index);
    if (!key || !matchesManagedStorageKey(key)) continue;

    const value = window.localStorage.getItem(key);
    if (value !== null) {
      entries[normalizeStorageKey(key)] = value;
    }
  }

  return entries;
}

function syncLocalStorage(entries: Record<string, string>) {
  if (typeof window === 'undefined') return;

  const existingKeys: string[] = [];
  for (let index = 0; index < window.localStorage.length; index++) {
    const key = window.localStorage.key(index);
    if (key && matchesManagedStorageKey(key)) {
      existingKeys.push(key);
    }
  }

  for (const key of existingKeys) {
    if (!(key in entries)) {
      window.localStorage.removeItem(key);
    }
  }

  for (const [key, value] of Object.entries(entries)) {
    window.localStorage.setItem(key, value);
  }
}

function readRawStorage(key: string): string | null {
  if (backendInitialized) {
    for (const candidate of storageKeyCandidates(key)) {
      const value = backendEntries.get(candidate);
      if (value !== undefined) return value;
    }
    return null;
  }

  if (typeof window === 'undefined') return null;
  for (const candidate of storageKeyCandidates(key)) {
    const value = window.localStorage.getItem(candidate);
    if (value !== null) return value;
  }
  return null;
}

async function persistSnapshot(entries: Record<string, string>) {
  const preferences = await invoke<StoragePreferences>('save_persistence_snapshot', { entries });
  storagePreferences = preferences;
  return preferences;
}

export async function initializePersistence() {
  if (!isTauriRuntime()) {
    return storagePreferences;
  }

  try {
    const bootstrap = await invoke<PersistenceBootstrap>('load_persistence_bootstrap');
    storagePreferences = bootstrap.preferences;

    const backendHasEntries = Object.keys(bootstrap.entries).length > 0;
    if (backendHasEntries) {
      backendEntries = new Map(Object.entries(bootstrap.entries));
      backendInitialized = true;
      syncLocalStorage(bootstrap.entries);
      return storagePreferences;
    }

    const localEntries = collectLocalEntries();
    backendEntries = new Map(Object.entries(localEntries));
    backendInitialized = true;

    if (Object.keys(localEntries).length) {
      await persistSnapshot(localEntries);
    }

    return storagePreferences;
  } catch (error) {
    console.error('initializePersistence failed', error);
    backendInitialized = false;
    backendEntries = new Map();
    return storagePreferences;
  }
}

export function isPersistenceBackendAvailable() {
  return isTauriRuntime();
}

export function getStoragePreferences() {
  return storagePreferences;
}

export function getStorageSnapshot(): Record<string, string> {
  if (backendInitialized) {
    return Object.fromEntries(backendEntries);
  }

  return collectLocalEntries();
}

function collectManagedKeysByPrefixes(prefixes: string[]) {
  const normalizedPrefixes = prefixes
    .map((prefix) => normalizeStorageKey(prefix.trim()))
    .filter(Boolean);

  if (!normalizedPrefixes.length) return [];

  if (backendInitialized) {
    return Array.from(backendEntries.keys())
      .filter((key) => matchesManagedStorageKey(key) && matchesAnyStoragePrefix(key, normalizedPrefixes));
  }

  if (typeof window === 'undefined') return [];

  const keys: string[] = [];
  for (let index = 0; index < window.localStorage.length; index++) {
    const key = window.localStorage.key(index);
    if (!key || !matchesManagedStorageKey(key) || !matchesAnyStoragePrefix(key, normalizedPrefixes)) {
      continue;
    }
    keys.push(key);
  }

  return keys;
}

export async function chooseDirectory(startDirectory?: string | null) {
  if (!isTauriRuntime()) return null;
  return invoke<string | null>('pick_folder', {
    startDirectory: startDirectory ?? null,
  });
}

export async function updateDataDirectory(path: string | null) {
  const preferences = await invoke<StoragePreferences>('set_data_directory', {
    path,
    entries: getStorageSnapshot(),
  });
  storagePreferences = preferences;
  return preferences;
}

export async function updateDownloadDirectory(path: string | null) {
  const preferences = await invoke<StoragePreferences>('set_download_directory', { path });
  storagePreferences = preferences;
  return preferences;
}

export async function clearManagedStoragePrefixes(prefixes: string[]) {
  const keys = [...new Set(collectManagedKeysByPrefixes(prefixes))];
  if (!keys.length) return 0;

  if (backendInitialized) {
    await invoke('remove_persistence_entries', { keys });
    for (const key of keys) {
      backendEntries.delete(key);
    }
  }

  if (typeof window !== 'undefined') {
    for (const key of keys) {
      window.localStorage.removeItem(key);
    }
  }

  return keys.length;
}

export function readVersionedStorage<T>(
  key: string,
  version: number,
  options: ReadStorageOptions<T>,
): T {
  const raw = readRawStorage(key);
  if (!raw) return options.fallback;

  try {
    const parsed = JSON.parse(raw) as unknown;
    if (isEnvelope<T>(parsed) && parsed.version === version) {
      if (!options.validate || options.validate(parsed.data)) {
        return parsed.data;
      }
    }
  } catch {
    // Fall through to legacy migration.
  }

  if (options.migrateLegacy) {
    const migrated = options.migrateLegacy(raw);
    if (migrated !== null) {
      writeVersionedStorage(key, version, migrated);
      return migrated;
    }
  }

  return options.fallback;
}

export function writeVersionedStorage<T>(key: string, version: number, data: T) {
  if (typeof window === 'undefined') return;

  const envelope: StorageEnvelope<T> = {
    version,
    updatedAt: Date.now(),
    data,
  };
  const raw = JSON.stringify(envelope);

  window.localStorage.setItem(key, raw);

  if (!backendInitialized) return;

  backendEntries.set(key, raw);
  void invoke('write_persistence_entry', { key, value: raw }).catch((error) => {
    console.error('write_persistence_entry failed', error);
  });
}

import { invoke } from '@tauri-apps/api/core';
import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';
import { chooseDirectory, readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';
import type { Track } from '@/stores/player';

const LOCAL_LIBRARY_STORAGE_KEY = 'fashion:local-library';
const LOCAL_LIBRARY_STORAGE_VERSION = 1;
const LOCAL_SOURCE = 'local';

export interface LocalLibraryFolder {
  id: string;
  path: string;
  addedAt: number;
}

export interface LocalTrackRecord {
  id: string;
  path: string;
  fileName: string;
  title: string;
  artist: string;
  album: string;
  durationSec: number | null;
  coverPath: string | null;
  lyricPath: string | null;
  fileSize: number;
  modifiedAt: number;
  createdAt: number;
  updatedAt: number;
}

interface PersistedLocalLibrary {
  folders: LocalLibraryFolder[];
  tracks: LocalTrackRecord[];
  lastScanAt: number | null;
}

interface LocalLibraryScanResult {
  scannedFiles: number;
  importedFiles: number;
  updatedFiles: number;
  removedFiles: number;
  skippedFiles: number;
}

interface ScanLocalLibraryResponse extends PersistedLocalLibrary {
  scanResult: LocalLibraryScanResult;
}

export interface LocalTrackView {
  record: LocalTrackRecord;
  track: Track;
}

function isFolderShape(value: unknown): value is LocalLibraryFolder {
  if (!value || typeof value !== 'object') return false;
  const folder = value as Record<string, unknown>;
  return (
    typeof folder.id === 'string' &&
    typeof folder.path === 'string' &&
    typeof folder.addedAt === 'number'
  );
}

function isTrackRecordShape(value: unknown): value is LocalTrackRecord {
  if (!value || typeof value !== 'object') return false;
  const track = value as Record<string, unknown>;
  return (
    typeof track.id === 'string' &&
    typeof track.path === 'string' &&
    typeof track.fileName === 'string' &&
    typeof track.title === 'string' &&
    typeof track.artist === 'string' &&
    typeof track.album === 'string' &&
    (typeof track.durationSec === 'number' || track.durationSec === null) &&
    (typeof track.coverPath === 'string' || track.coverPath === null) &&
    (typeof track.lyricPath === 'string' || track.lyricPath === null) &&
    typeof track.fileSize === 'number' &&
    typeof track.modifiedAt === 'number' &&
    typeof track.createdAt === 'number' &&
    typeof track.updatedAt === 'number'
  );
}

function isPersistedLocalLibrary(value: unknown): value is PersistedLocalLibrary {
  if (!value || typeof value !== 'object') return false;
  const library = value as Record<string, unknown>;
  return (
    Array.isArray(library.folders) &&
    library.folders.every(isFolderShape) &&
    Array.isArray(library.tracks) &&
    library.tracks.every(isTrackRecordShape) &&
    (typeof library.lastScanAt === 'number' || library.lastScanAt === null)
  );
}

function normalizeText(value: string, fallback: string) {
  const normalized = value.trim();
  return normalized || fallback;
}

function recordToTrack(record: LocalTrackRecord): Track {
  return {
    id: record.id,
    name: normalizeText(record.title, record.fileName),
    artist: normalizeText(record.artist, '未知艺术家'),
    album: normalizeText(record.album, '未知专辑'),
    pic_id: record.coverPath ?? '',
    lyric_id: record.lyricPath ?? '',
    source: LOCAL_SOURCE,
    url: record.path,
    coverUrl: record.coverPath ?? undefined,
    durationSec: record.durationSec ?? undefined,
  };
}

function sortRecords(records: LocalTrackRecord[]) {
  return [...records].sort((left, right) => {
    const byTitle = left.title.localeCompare(right.title, 'zh-CN');
    if (byTitle !== 0) return byTitle;
    return left.path.localeCompare(right.path, 'zh-CN');
  });
}

function readInitialLibrary(): PersistedLocalLibrary {
  return readVersionedStorage<PersistedLocalLibrary>(LOCAL_LIBRARY_STORAGE_KEY, LOCAL_LIBRARY_STORAGE_VERSION, {
    fallback: {
      folders: [],
      tracks: [],
      lastScanAt: null,
    },
    validate: isPersistedLocalLibrary,
  });
}

export const useLocalLibraryStore = defineStore('local-library', () => {
  const initialLibrary = readInitialLibrary();
  const folders = ref<LocalLibraryFolder[]>(initialLibrary.folders);
  const records = ref<LocalTrackRecord[]>(sortRecords(initialLibrary.tracks));
  const lastScanAt = ref<number | null>(initialLibrary.lastScanAt);
  const lastScanResult = ref<LocalLibraryScanResult | null>(null);
  const scanError = ref('');
  const scanning = ref(false);

  watch(
    [folders, records, lastScanAt],
    () => {
      writeVersionedStorage(LOCAL_LIBRARY_STORAGE_KEY, LOCAL_LIBRARY_STORAGE_VERSION, {
        folders: folders.value,
        tracks: records.value,
        lastScanAt: lastScanAt.value,
      } satisfies PersistedLocalLibrary);
    },
    { deep: true, immediate: true },
  );

  const tracks = computed<LocalTrackView[]>(() =>
    records.value.map((record) => ({
      record,
      track: recordToTrack(record),
    })),
  );

  async function addFolder() {
    const selected = await chooseDirectory(folders.value[0]?.path ?? null);
    if (!selected) return null;

    const existing = folders.value.find((item) => item.path === selected);
    if (existing) return existing.id;

    const folder: LocalLibraryFolder = {
      id: `folder-${Date.now()}`,
      path: selected,
      addedAt: Date.now(),
    };

    folders.value = [...folders.value, folder];
    return folder.id;
  }

  function removeFolder(folderId: string) {
    const folder = folders.value.find((item) => item.id === folderId);
    if (!folder) return;
    folders.value = folders.value.filter((item) => item.id !== folderId);
    const normalizedFolderPath = folder.path.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
    records.value = records.value.filter((record) => {
      const normalizedRecordPath = record.path.replace(/\\/g, '/').toLowerCase();
      return !(normalizedRecordPath === normalizedFolderPath || normalizedRecordPath.startsWith(`${normalizedFolderPath}/`));
    });
  }

  function removeTrack(trackId: string) {
    records.value = records.value.filter((record) => record.id !== trackId);
  }

  function replaceLibrary(next: PersistedLocalLibrary) {
    folders.value = [...next.folders];
    records.value = sortRecords(next.tracks);
    lastScanAt.value = next.lastScanAt;
  }

  async function scanLibrary() {
    if (scanning.value || !folders.value.length) return null;

    scanning.value = true;
    scanError.value = '';
    try {
      const response = await invoke<ScanLocalLibraryResponse>('scan_local_library', {
        folders: folders.value.map((item) => item.path),
      });
      replaceLibrary(response);
      lastScanResult.value = response.scanResult;
      return response.scanResult;
    } catch (error) {
      scanError.value = error instanceof Error ? error.message : String(error);
      return null;
    } finally {
      scanning.value = false;
    }
  }

  return {
    folders,
    tracks,
    lastScanAt,
    lastScanResult,
    scanError,
    scanning,
    addFolder,
    removeFolder,
    removeTrack,
    replaceLibrary,
    scanLibrary,
  };
});

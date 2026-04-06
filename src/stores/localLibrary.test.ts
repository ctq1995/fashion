import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';

const persistenceMocks = vi.hoisted(() => ({
  chooseDirectory: vi.fn(),
}));

vi.mock('@/utils/persistence', async () => {
  const actual = await vi.importActual<typeof import('@/utils/persistence')>('@/utils/persistence');
  return {
    ...actual,
    chooseDirectory: persistenceMocks.chooseDirectory,
  };
});

const tauriMocks = vi.hoisted(() => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: tauriMocks.invoke,
}));

import { useLocalLibraryStore } from '@/stores/localLibrary';

describe('local library store', () => {
  beforeEach(() => {
    window.localStorage.clear();
    setActivePinia(createPinia());
    persistenceMocks.chooseDirectory.mockReset();
    tauriMocks.invoke.mockReset();
  });

  it('hydrates local tracks from persisted storage', () => {
    window.localStorage.setItem(
      'fashion:local-library',
      JSON.stringify({
        version: 1,
        updatedAt: Date.now(),
        data: {
          folders: [{ id: 'folder-1', path: 'D:/Music', addedAt: 1 }],
          tracks: [
            {
              id: 'local-1',
              path: 'D:/Music/song.mp3',
              fileName: 'song.mp3',
              title: 'Song',
              artist: 'Artist',
              album: 'Album',
              durationSec: 200,
              coverPath: null,
              lyricPath: null,
              fileSize: 1024,
              modifiedAt: 2,
              createdAt: 1,
              updatedAt: 2,
            },
          ],
          lastScanAt: 2,
        },
      }),
    );

    const store = useLocalLibraryStore();

    expect(store.folders).toHaveLength(1);
    expect(store.tracks).toHaveLength(1);
    expect(store.tracks[0].track).toMatchObject({
      id: 'local-1',
      source: 'local',
      name: 'Song',
      artist: 'Artist',
      durationSec: 200,
    });
  });

  it('adds selected folders without duplicates', async () => {
    persistenceMocks.chooseDirectory
      .mockResolvedValueOnce('D:/Music')
      .mockResolvedValueOnce('D:/Music');

    const store = useLocalLibraryStore();

    await store.addFolder();
    await store.addFolder();

    expect(store.folders).toHaveLength(1);
    expect(store.folders[0].path).toBe('D:/Music');
  });

  it('removes tracks under removed folder and supports removing a single track', () => {
    window.localStorage.setItem(
      'fashion:local-library',
      JSON.stringify({
        version: 1,
        updatedAt: Date.now(),
        data: {
          folders: [
            { id: 'folder-1', path: 'C:/Music', addedAt: 1 },
            { id: 'folder-2', path: 'D:/Other', addedAt: 2 },
          ],
          tracks: [
            {
              id: 'local-1',
              path: 'C:/Music/song-a.mp3',
              fileName: 'song-a.mp3',
              title: 'Song A',
              artist: 'Artist A',
              album: 'Album A',
              durationSec: 120,
              coverPath: null,
              lyricPath: null,
              fileSize: 100,
              modifiedAt: 1,
              createdAt: 1,
              updatedAt: 1,
            },
            {
              id: 'local-2',
              path: 'D:/Other/song-b.mp3',
              fileName: 'song-b.mp3',
              title: 'Song B',
              artist: 'Artist B',
              album: 'Album B',
              durationSec: 140,
              coverPath: null,
              lyricPath: null,
              fileSize: 100,
              modifiedAt: 1,
              createdAt: 1,
              updatedAt: 1,
            },
          ],
          lastScanAt: 1,
        },
      }),
    );

    const store = useLocalLibraryStore();
    store.removeTrack('local-2');
    expect(store.tracks.map((item) => item.record.id)).toEqual(['local-1']);

    store.removeFolder('folder-1');
    expect(store.folders.map((item) => item.id)).toEqual(['folder-2']);
    expect(store.tracks).toHaveLength(0);
  });


  it('imports scan results and sorts tracks by title', async () => {
    tauriMocks.invoke.mockResolvedValue({
      folders: [{ id: 'folder-1', path: 'D:/Music', addedAt: 1 }],
      tracks: [
        {
          id: 'local-b',
          path: 'D:/Music/b.mp3',
          fileName: 'b.mp3',
          title: 'Bravo',
          artist: 'B',
          album: '',
          durationSec: null,
          coverPath: null,
          lyricPath: null,
          fileSize: 2,
          modifiedAt: 2,
          createdAt: 2,
          updatedAt: 2,
        },
        {
          id: 'local-a',
          path: 'D:/Music/a.mp3',
          fileName: 'a.mp3',
          title: 'Alpha',
          artist: 'A',
          album: '',
          durationSec: null,
          coverPath: null,
          lyricPath: null,
          fileSize: 1,
          modifiedAt: 1,
          createdAt: 1,
          updatedAt: 1,
        },
      ],
      lastScanAt: 3,
      scanResult: {
        scannedFiles: 2,
        importedFiles: 2,
        updatedFiles: 0,
        removedFiles: 0,
        skippedFiles: 0,
      },
    });

    const store = useLocalLibraryStore();
    store.replaceLibrary({
      folders: [{ id: 'folder-1', path: 'D:/Music', addedAt: 1 }],
      tracks: [],
      lastScanAt: null,
    });

    await store.scanLibrary();

    expect(tauriMocks.invoke).toHaveBeenCalledWith('scan_local_library', {
      folders: ['D:/Music'],
    });
    expect(store.tracks.map((item) => item.track.name)).toEqual(['Alpha', 'Bravo']);
    expect(store.lastScanResult?.importedFiles).toBe(2);
  });
});

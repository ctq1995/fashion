import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';
import type { Track } from '@/stores/player';
import { useMediaStore } from '@/stores/media';
import { readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';

export interface CustomPlaylist {
  id: string;
  name: string;
  description: string;
  tracks: Track[];
  createdAt: number;
  updatedAt: number;
}

const FAVORITES_STORAGE_KEY = 'fashion:favorites';
const PLAYLISTS_STORAGE_KEY = 'fashion:playlists';
const FAVORITES_STORAGE_VERSION = 1;
const PLAYLISTS_STORAGE_VERSION = 2;

function isTrackShape(value: unknown): value is Track {
  if (!value || typeof value !== 'object') return false;
  const track = value as Record<string, unknown>;
  return (
    typeof track.id === 'string' &&
    typeof track.name === 'string' &&
    typeof track.artist === 'string' &&
    typeof track.album === 'string' &&
    typeof track.pic_id === 'string' &&
    typeof track.lyric_id === 'string' &&
    typeof track.source === 'string'
  );
}

function readFavorites(): Track[] {
  return readVersionedStorage<Track[]>(FAVORITES_STORAGE_KEY, FAVORITES_STORAGE_VERSION, {
    fallback: [],
    validate: (value): value is Track[] => Array.isArray(value) && value.every(isTrackShape),
    migrateLegacy: (raw) => {
      try {
        const parsed = JSON.parse(raw);
        return Array.isArray(parsed) ? parsed.filter(isTrackShape) : null;
      } catch {
        return null;
      }
    },
  });
}

function isPlaylistShape(value: unknown): value is CustomPlaylist {
  if (!value || typeof value !== 'object') return false;
  const playlist = value as Record<string, unknown>;
  return (
    typeof playlist.id === 'string' &&
    typeof playlist.name === 'string' &&
    typeof playlist.description === 'string' &&
    typeof playlist.createdAt === 'number' &&
    typeof playlist.updatedAt === 'number' &&
    Array.isArray(playlist.tracks) &&
    playlist.tracks.every(isTrackShape)
  );
}

function readPlaylists(): CustomPlaylist[] {
  return readVersionedStorage<CustomPlaylist[]>(PLAYLISTS_STORAGE_KEY, PLAYLISTS_STORAGE_VERSION, {
    fallback: [],
    validate: (value): value is CustomPlaylist[] => Array.isArray(value) && value.every(isPlaylistShape),
    migrateLegacy: (raw) => {
      try {
        const parsed = JSON.parse(raw);
        if (!Array.isArray(parsed)) return null;

        return parsed
          .filter((item): item is Record<string, unknown> => !!item && typeof item === 'object')
          .map((item, index) => ({
            id: typeof item.id === 'string' ? item.id : `legacy-playlist-${index}`,
            name: typeof item.name === 'string' ? item.name : `歌单 ${index + 1}`,
            description: typeof item.description === 'string' ? item.description : '',
            tracks: Array.isArray(item.tracks) ? item.tracks.filter(isTrackShape) : [],
            createdAt: typeof item.createdAt === 'number' ? item.createdAt : Date.now(),
            updatedAt: typeof item.updatedAt === 'number' ? item.updatedAt : Date.now(),
          }));
      } catch {
        return null;
      }
    },
  });
}

function trackKey(track: Pick<Track, 'id' | 'source'>): string {
  return `${track.source}:${track.id}`;
}

export const useLibraryStore = defineStore('library', () => {
  const media = useMediaStore();
  const favorites = ref<Track[]>(readFavorites().map((track) => media.attachTrackCover(track)));
  const playlists = ref<CustomPlaylist[]>(
    readPlaylists().map((playlist) => ({
      ...playlist,
      tracks: playlist.tracks.map((track) => media.attachTrackCover(track)),
    })),
  );

  watch(favorites, (value) => {
    writeVersionedStorage(FAVORITES_STORAGE_KEY, FAVORITES_STORAGE_VERSION, value);
  }, { deep: true, immediate: true });

  watch(playlists, (value) => {
    writeVersionedStorage(PLAYLISTS_STORAGE_KEY, PLAYLISTS_STORAGE_VERSION, value);
  }, { deep: true, immediate: true });

  const playlistCount = computed(() => playlists.value.length);

  function isFavorite(track: Pick<Track, 'id' | 'source'>): boolean {
    const key = trackKey(track);
    return favorites.value.some((item) => trackKey(item) === key);
  }

  function toggleFavorite(track: Track) {
    const nextTrack = media.attachTrackCover(track);
    if (isFavorite(track)) {
      favorites.value = favorites.value.filter((item) => trackKey(item) !== trackKey(track));
      return;
    }

    favorites.value = [nextTrack, ...favorites.value];
  }

  function removeFavorite(track: Pick<Track, 'id' | 'source'>) {
    favorites.value = favorites.value.filter((item) => trackKey(item) !== trackKey(track));
  }

  function createPlaylist(name: string): string | null {
    const normalized = name.trim();
    const fallbackName = `新建歌单 ${playlists.value.length + 1}`;
    const finalName = normalized || fallbackName;

    const now = Date.now();
    const playlist: CustomPlaylist = {
      id: `playlist-${now}`,
      name: finalName,
      description: '',
      tracks: [],
      createdAt: now,
      updatedAt: now,
    };

    playlists.value = [playlist, ...playlists.value];
    return playlist.id;
  }

  function deletePlaylist(playlistId: string) {
    playlists.value = playlists.value.filter((item) => item.id !== playlistId);
  }

  function updatePlaylist(playlistId: string, patch: Partial<Pick<CustomPlaylist, 'name' | 'description'>>) {
    playlists.value = playlists.value.map((playlist) => {
      if (playlist.id !== playlistId) return playlist;
      return {
        ...playlist,
        name: patch.name?.trim() ? patch.name.trim() : playlist.name,
        description: typeof patch.description === 'string' ? patch.description : playlist.description,
        updatedAt: Date.now(),
      };
    });
  }

  function addTrackToPlaylist(playlistId: string, track: Track) {
    const nextTrack = media.attachTrackCover(track);
    playlists.value = playlists.value.map((playlist) => {
      if (playlist.id !== playlistId) return playlist;

      const existingIndex = playlist.tracks.findIndex((item) => trackKey(item) === trackKey(track));
      if (existingIndex >= 0) {
        if (!nextTrack.coverUrl || playlist.tracks[existingIndex]?.coverUrl === nextTrack.coverUrl) {
          return playlist;
        }

        return {
          ...playlist,
          updatedAt: Date.now(),
          tracks: playlist.tracks.map((item, index) =>
            index === existingIndex ? { ...item, coverUrl: nextTrack.coverUrl } : item,
          ),
        };
      }

      return {
        ...playlist,
        updatedAt: Date.now(),
        tracks: [nextTrack, ...playlist.tracks],
      };
    });
  }

  function removeTrackFromPlaylist(playlistId: string, track: Pick<Track, 'id' | 'source'>) {
    playlists.value = playlists.value.map((playlist) => {
      if (playlist.id !== playlistId) return playlist;
      return {
        ...playlist,
        updatedAt: Date.now(),
        tracks: playlist.tracks.filter((item) => trackKey(item) !== trackKey(track)),
      };
    });
  }

  function syncTrackCover(track: Pick<Track, 'id' | 'source'>, coverUrl: string) {
    let favoritesChanged = false;
    const nextFavorites = favorites.value.map((item) => {
      if (trackKey(item) !== trackKey(track) || item.coverUrl === coverUrl) {
        return item;
      }

      favoritesChanged = true;
      return { ...item, coverUrl };
    });

    if (favoritesChanged) {
      favorites.value = nextFavorites;
    }

    let playlistsChanged = false;
    const nextPlaylists = playlists.value.map((playlist) => {
      let playlistChanged = false;
      const nextTracks = playlist.tracks.map((item) => {
        if (trackKey(item) !== trackKey(track) || item.coverUrl === coverUrl) {
          return item;
        }

        playlistChanged = true;
        return { ...item, coverUrl };
      });

      if (!playlistChanged) {
        return playlist;
      }

      playlistsChanged = true;
      return {
        ...playlist,
        tracks: nextTracks,
        updatedAt: Date.now(),
      };
    });

    if (playlistsChanged) {
      playlists.value = nextPlaylists;
    }
  }

  function exportPlaylist(playlistId: string) {
    const playlist = playlists.value.find((item) => item.id === playlistId);
    if (!playlist || typeof window === 'undefined') return;

    const blob = new Blob([JSON.stringify(playlist, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${playlist.name}.json`;
    link.click();
    URL.revokeObjectURL(url);
  }

  return {
    favorites,
    playlists,
    playlistCount,
    isFavorite,
    toggleFavorite,
    removeFavorite,
    syncTrackCover,
    createPlaylist,
    deletePlaylist,
    updatePlaylist,
    addTrackToPlaylist,
    removeTrackFromPlaylist,
    exportPlaylist,
  };
});

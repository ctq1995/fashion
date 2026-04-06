import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useLibraryStore } from '@/stores/library';
import { useLocalLibraryStore } from '@/stores/localLibrary';
import { useMediaStore } from '@/stores/media';
import type { Track } from '@/stores/player';

const track: Track = {
  id: '1',
  name: 'Track One',
  artist: 'Artist',
  album: 'Album',
  pic_id: 'pic-1',
  lyric_id: 'lyric-1',
  source: 'netease',
};

describe('library store', () => {
  beforeEach(() => {
    window.localStorage.clear();
    setActivePinia(createPinia());
  });

  it('toggles favorites', () => {
    const store = useLibraryStore();

    store.toggleFavorite(track);
    expect(store.favorites).toHaveLength(1);

    store.toggleFavorite(track);
    expect(store.favorites).toHaveLength(0);
  });

  it('creates playlists and adds tracks', () => {
    const store = useLibraryStore();
    const playlistId = store.createPlaylist('My Playlist');

    expect(playlistId).toBeTruthy();
    store.addTrackToPlaylist(playlistId!, track);

    expect(store.playlists[0].tracks).toHaveLength(1);
    expect(store.playlists[0].name).toBe('My Playlist');
  });

  it('hydrates and syncs cached covers across library lists', () => {
    const media = useMediaStore();
    media.rememberCover(track.source, track.pic_id, 'https://cdn.example/cover.jpg');

    const store = useLibraryStore();
    store.toggleFavorite(track);

    const playlistId = store.createPlaylist('With Cover');
    store.addTrackToPlaylist(playlistId!, track);

    expect(store.favorites[0].coverUrl).toBe('https://cdn.example/cover.jpg');
    expect(store.playlists[0].tracks[0].coverUrl).toBe('https://cdn.example/cover.jpg');

    store.syncTrackCover(track, 'https://cdn.example/cover-2.jpg');

    expect(store.favorites[0].coverUrl).toBe('https://cdn.example/cover-2.jpg');
    expect(store.playlists[0].tracks[0].coverUrl).toBe('https://cdn.example/cover-2.jpg');
  });

  it('adds local library tracks into playlists like normal tracks', () => {
    const localLibrary = useLocalLibraryStore();
    localLibrary.replaceLibrary({
      folders: [{ id: 'folder-1', path: 'D:/Music', addedAt: Date.now() }],
      tracks: [
        {
          id: 'local-track-1',
          path: 'D:/Music/song.mp3',
          fileName: 'song.mp3',
          title: 'Local Song',
          artist: 'Local Artist',
          album: 'Local Album',
          durationSec: 215,
          coverPath: null,
          lyricPath: null,
          fileSize: 2048,
          modifiedAt: 100,
          createdAt: 90,
          updatedAt: 100,
        },
      ],
      lastScanAt: Date.now(),
    });

    const store = useLibraryStore();
    const playlistId = store.createPlaylist('Local Playlist');
    const localTrack = localLibrary.tracks[0].track;

    store.addTrackToPlaylist(playlistId!, localTrack);

    expect(store.playlists[0].tracks).toHaveLength(1);
    expect(store.playlists[0].tracks[0]).toMatchObject({
      id: 'local-track-1',
      source: 'local',
      name: 'Local Song',
      artist: 'Local Artist',
      durationSec: 215,
    });
  });
});

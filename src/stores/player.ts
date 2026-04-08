import { defineStore } from 'pinia';
import { convertFileSrc } from '@tauri-apps/api/core';
import { computed, ref, watch } from 'vue';
import {
  AUTO_SWITCH_SOURCE_VALUES,
  musicApi,
  getArtistNames,
  getAlbumName,
  toStr,
  type SearchResult, type MusicSource,
} from '@/api/music';
import { useLibraryStore } from '@/stores/library';
import { useMediaStore } from '@/stores/media';
import { readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';
import { isProductionLoggingEnabled, sanitizeRemoteMediaUrl } from '@/utils/security';

export type PlayMode = 'sequence' | 'random' | 'single';

const BITRATES = [128, 192, 320, 999] as const;
const PLAYBACK_RATES = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0] as const;
const BITRATE_STORAGE_KEY = 'fashion:bitrate';
const HISTORY_STORAGE_KEY = 'fashion:history';
const SESSION_STORAGE_KEY = 'fashion:session';
const PLAYBACK_RATE_STORAGE_KEY = 'fashion:playbackRate';
const MINI_PLAYER_STORAGE_KEY = 'fashion:mini-player';
const PLAYER_STORAGE_VERSION = 1;
const HISTORY_STORAGE_VERSION = 2;
const SESSION_STORAGE_VERSION = 1;
const PLAYBACK_RATE_STORAGE_VERSION = 1;
const MINI_PLAYER_STORAGE_VERSION = 1;
const MAX_HISTORY_ITEMS = 100;

export type PlaybackRate = (typeof PLAYBACK_RATES)[number];

export type Bitrate = (typeof BITRATES)[number];

export interface Track {
  id: string;
  name: string;
  artist: string;
  album: string;
  pic_id: string;
  lyric_id: string;
  source: string;
  url?: string;
  coverUrl?: string;
  durationSec?: number;
}

export interface HistoryItem extends Track {
  historyId: string;
  playedAt: number;
  finishedAt: number | null;
  lastPosition: number;
  durationSnapshot: number;
  completed: boolean;
}

export interface MiniPlayerStateSnapshot {
  currentTrack: Track | null;
  isPlaying: boolean;
  duration: number;
  currentTime: number;
  playMode: PlayMode;
}

interface PlayerSession {
  queue: Track[];
  currentIndex: number;
  currentTrack: Track | null;
  currentTime: number;
  volume: number;
  playMode: PlayMode;
  preferredBitrate: Bitrate;
}

interface MiniPlayerSession {
  visible: boolean;
}

function shouldUseAnonymousCors(source: string, localPath?: string) {
  return !localPath && source !== 'gequbao';
}

function resolvePlayableAudioUrl(url?: string, localPath?: string) {
  if (localPath) {
    return convertFileSrc(localPath);
  }
  return sanitizeRemoteMediaUrl(url);
}

function shouldDebugPlayback(track: Pick<Track, 'source'> | null | undefined) {
  return track?.source === 'gequbao';
}

function logPlaybackDebug(stage: string, payload?: unknown) {
  if (!isProductionLoggingEnabled()) return;
  if (payload === undefined) {
    console.log('[gequbao-debug]', stage);
    return;
  }
  console.log('[gequbao-debug]', stage, payload);
}

function logPlaybackError(stage: string, payload?: unknown) {
  if (!isProductionLoggingEnabled()) return;
  if (payload === undefined) {
    console.error('[gequbao-debug]', stage);
    return;
  }
  console.error('[gequbao-debug]', stage, payload);
}

function trackIdentity(track: Pick<Track, 'id' | 'source'>): string {
  return `${track.source}:${track.id}`;
}

function normalizeHistory(items: HistoryItem[]): HistoryItem[] {
  const seen = new Set<string>();
  const normalized: HistoryItem[] = [];

  for (const item of items) {
    const key = trackIdentity(item);
    if (seen.has(key)) continue;
    seen.add(key);
    normalized.push(item);
    if (normalized.length >= MAX_HISTORY_ITEMS) break;
  }

  return normalized;
}

function parseLrc(lrc: string): { time: number; text: string }[] {
  if (!lrc) return [];
  const result: { time: number; text: string }[] = [];
  const lines = lrc.split('\n');

  for (const line of lines) {
    const text = line.replace(/\[[\d:.]+\]/g, '').trim();
    if (!text) continue;

    const reg = /\[(\d{2}):(\d{2})[.:]([\d]{2,3})\]/g;
    let match: RegExpExecArray | null;
    while ((match = reg.exec(line)) !== null) {
      const m = parseInt(match[1]);
      const s = parseInt(match[2]);
      const ms = parseInt(match[3].padEnd(3, '0'));
      result.push({ time: m * 60 + s + ms / 1000, text });
    }
  }

  return result.sort((a, b) => a.time - b.time);
}

function isTrack(value: unknown): value is Track {
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

function isBitrate(value: unknown): value is Bitrate {
  return typeof value === 'number' && BITRATES.includes(value as Bitrate);
}

function isPlaybackRate(value: unknown): value is PlaybackRate {
  return typeof value === 'number' && (PLAYBACK_RATES as readonly number[]).includes(value);
}

function isPlayMode(value: unknown): value is PlayMode {
  return value === 'sequence' || value === 'random' || value === 'single';
}

function isHistoryItem(value: unknown): value is HistoryItem {
  if (!isTrack(value)) return false;
  const item = value as unknown as Record<string, unknown>;
  return (
    typeof item.historyId === 'string' &&
    typeof item.playedAt === 'number' &&
    (typeof item.finishedAt === 'number' || item.finishedAt === null) &&
    typeof item.lastPosition === 'number' &&
    typeof item.durationSnapshot === 'number' &&
    typeof item.completed === 'boolean'
  );
}

function isPlayerSession(value: unknown): value is PlayerSession {
  if (!value || typeof value !== 'object') return false;
  const session = value as Record<string, unknown>;
  return (
    Array.isArray(session.queue) &&
    session.queue.every(isTrack) &&
    typeof session.currentIndex === 'number' &&
    (session.currentTrack === null || isTrack(session.currentTrack)) &&
    typeof session.currentTime === 'number' &&
    typeof session.volume === 'number' &&
    isPlayMode(session.playMode) &&
    isBitrate(session.preferredBitrate)
  );
}

function readInitialBitrate(): Bitrate {
  return readVersionedStorage<Bitrate>(BITRATE_STORAGE_KEY, PLAYER_STORAGE_VERSION, {
    fallback: 320,
    validate: isBitrate,
    migrateLegacy: (raw) => {
      const value = Number(raw);
      return isBitrate(value) ? value : null;
    },
  });
}

function readInitialHistory(): HistoryItem[] {
  const stored = readVersionedStorage<HistoryItem[]>(HISTORY_STORAGE_KEY, HISTORY_STORAGE_VERSION, {
    fallback: [],
    validate: (value): value is HistoryItem[] => Array.isArray(value) && value.every(isHistoryItem),
    migrateLegacy: (raw) => {
      try {
        const parsed = JSON.parse(raw);
        if (!Array.isArray(parsed)) return null;
        const migrated = parsed
          .filter(isTrack)
          .map((track, index) => ({
            ...track,
            historyId: `legacy-${track.source}-${track.id}-${index}`,
            playedAt: Date.now() - index * 1000,
            finishedAt: null,
            lastPosition: 0,
            durationSnapshot: 0,
            completed: false,
          }));
        return normalizeHistory(migrated);
      } catch {
        return null;
      }
    },
  });

  return normalizeHistory(stored);
}

function readInitialSession(preferredBitrate: Bitrate): PlayerSession {
  return readVersionedStorage<PlayerSession>(SESSION_STORAGE_KEY, SESSION_STORAGE_VERSION, {
    fallback: {
      queue: [],
      currentIndex: -1,
      currentTrack: null,
      currentTime: 0,
      volume: 0.8,
      playMode: 'sequence',
      preferredBitrate,
    },
    validate: isPlayerSession,
  });
}

function readInitialPlaybackRate(): PlaybackRate {
  return readVersionedStorage<PlaybackRate>(PLAYBACK_RATE_STORAGE_KEY, PLAYBACK_RATE_STORAGE_VERSION, {
    fallback: 1.0,
    validate: isPlaybackRate,
    migrateLegacy: (raw) => {
      const parsed = Number(raw);
      return isPlaybackRate(parsed) ? parsed : null;
    },
  });
}

function readMiniPlayerSession(): MiniPlayerSession {
  return readVersionedStorage<MiniPlayerSession>(MINI_PLAYER_STORAGE_KEY, MINI_PLAYER_STORAGE_VERSION, {
    fallback: { visible: false },
    validate: (value): value is MiniPlayerSession => (
      !!value
      && typeof value === 'object'
      && typeof (value as MiniPlayerSession).visible === 'boolean'
    ),
  });
}

export const usePlayerStore = defineStore('player', () => {
  const media = useMediaStore();
  const library = useLibraryStore();
  const initialBitrate = readInitialBitrate();
  const initialSession = readInitialSession(initialBitrate);

  const audio = new Audio();
  audio.preload = 'auto';

  const queue = ref<Track[]>(initialSession.queue.map((track) => media.attachTrackCover(track)));
  const currentIndex = ref(initialSession.currentIndex);
  const currentTrack = ref<Track | null>(
    initialSession.currentTrack ? media.attachTrackCover(initialSession.currentTrack) : null,
  );
  const isPlaying = ref(false);
  const duration = ref(0);
  const currentTime = ref(initialSession.currentTime);
  const playbackTimeUpdatedAt = ref(Date.now());
  const volume = ref(initialSession.volume);
  const playMode = ref<PlayMode>(initialSession.playMode);
  const lyricLines = ref<{ time: number; text: string }[]>([]);
  const tlyricLines = ref<{ time: number; text: string }[]>([]);
  const currentLyricIndex = ref(-1);
  const loading = ref(false);
  const error = ref('');
  const showLyric = ref(false);
  const preferredBitrate = ref<Bitrate>(initialSession.preferredBitrate);
  const history = ref<HistoryItem[]>(readInitialHistory().map((item) => media.attachTrackCover(item)));

  const playbackRate = ref<PlaybackRate>(readInitialPlaybackRate());
  const showMiniPlayer = ref(readMiniPlayerSession().visible);
  const sleepTimerEndTime = ref<number | null>(null);
  const sleepTimerRemaining = ref<number | null>(null);

  let loadToken = 0;
  let isResettingAudio = false;
  let currentHistoryId: string | null = null;
  let lastSessionPersistAt = 0;
  let pendingRestoreTime = initialSession.currentTime;
  let autoSwitchAttempt = 0;
  let sleepTimerHandle: ReturnType<typeof setInterval> | null = null;

  audio.volume = volume.value;
  audio.playbackRate = playbackRate.value;

  const currentLyricText = computed(() =>
    currentLyricIndex.value >= 0 && lyricLines.value[currentLyricIndex.value]
      ? lyricLines.value[currentLyricIndex.value].text
      : ''
  );

  function persistBitrate() {
    writeVersionedStorage(BITRATE_STORAGE_KEY, PLAYER_STORAGE_VERSION, preferredBitrate.value);
  }

  function persistHistory() {
    const normalized = normalizeHistory(history.value);
    const changed =
      normalized.length !== history.value.length ||
      normalized.some((item, index) => item.historyId !== history.value[index]?.historyId);

    if (changed) {
      history.value = normalized;
      return;
    }

    writeVersionedStorage(HISTORY_STORAGE_KEY, HISTORY_STORAGE_VERSION, normalized);
  }

  function persistSession(force = false) {
    const now = Date.now();
    if (!force && now - lastSessionPersistAt < 1200) return;
    lastSessionPersistAt = now;

    writeVersionedStorage(SESSION_STORAGE_KEY, SESSION_STORAGE_VERSION, {
      queue: queue.value,
      currentIndex: currentIndex.value,
      currentTrack: currentTrack.value,
      currentTime: currentTime.value,
      volume: volume.value,
      playMode: playMode.value,
      preferredBitrate: preferredBitrate.value,
    } satisfies PlayerSession);
  }

  watch(preferredBitrate, persistBitrate, { immediate: true });
  watch(history, persistHistory, { deep: true, immediate: true });
  watch(showMiniPlayer, (visible) => {
    writeVersionedStorage(MINI_PLAYER_STORAGE_KEY, MINI_PLAYER_STORAGE_VERSION, { visible });
  }, { immediate: true });
  watch([queue, currentIndex, currentTrack, volume, playMode], () => persistSession(true), { deep: true });

  function syncCurrentHistory(completed = false) {
    if (!currentHistoryId) return;

    history.value = history.value.map((item) => {
      if (item.historyId !== currentHistoryId) return item;

      return {
        ...item,
        lastPosition: currentTime.value,
        durationSnapshot: duration.value,
        completed,
        finishedAt: completed ? Date.now() : item.finishedAt,
      };
    });
  }

  function updateMediaSession() {
    if (typeof navigator === 'undefined' || !('mediaSession' in navigator)) return;

    if (!currentTrack.value) {
      navigator.mediaSession.metadata = null;
      navigator.mediaSession.playbackState = 'none';
      return;
    }

    navigator.mediaSession.metadata = new MediaMetadata({
      title: currentTrack.value.name,
      artist: currentTrack.value.artist,
      album: currentTrack.value.album,
      artwork: currentTrack.value.coverUrl
        ? [{ src: currentTrack.value.coverUrl, sizes: '512x512', type: 'image/jpeg' }]
        : [],
    });
    navigator.mediaSession.playbackState = isPlaying.value ? 'playing' : 'paused';
  }

  function registerMediaSessionHandlers() {
    if (typeof navigator === 'undefined' || !('mediaSession' in navigator)) return;

    navigator.mediaSession.setActionHandler('play', () => { void togglePlay(); });
    navigator.mediaSession.setActionHandler('pause', () => { void togglePlay(); });
    navigator.mediaSession.setActionHandler('previoustrack', () => { playPrev(); });
    navigator.mediaSession.setActionHandler('nexttrack', () => { playNext(); });
    navigator.mediaSession.setActionHandler('seekto', (details) => {
      if (typeof details.seekTime === 'number') {
        seek(details.seekTime);
      }
    });
  }

  function syncLyricIndexByTime(time = audio.currentTime) {
    if (!lyricLines.value.length) {
      currentLyricIndex.value = -1;
      return;
    }

    let idx = -1;
    for (let i = 0; i < lyricLines.value.length; i++) {
      if (lyricLines.value[i].time <= time) idx = i;
      else break;
    }
    currentLyricIndex.value = idx;
  }

  function syncPlaybackPosition(time = audio.currentTime) {
    currentTime.value = time;
    playbackTimeUpdatedAt.value = Date.now();
    syncLyricIndexByTime(time);
  }

  audio.addEventListener('timeupdate', () => {
    syncPlaybackPosition(audio.currentTime);

    syncCurrentHistory(false);
    persistSession();
  });

  audio.addEventListener('durationchange', () => {
    duration.value = Number.isNaN(audio.duration) ? 0 : audio.duration;
  });

  audio.addEventListener('loadedmetadata', () => {
    if (pendingRestoreTime > 0) {
      audio.currentTime = Math.min(pendingRestoreTime, audio.duration || pendingRestoreTime);
      syncPlaybackPosition(audio.currentTime);
      pendingRestoreTime = 0;
      persistSession(true);
    }
  });

  audio.addEventListener('loadstart', () => {
    if (!audio.currentSrc) return;
    loading.value = true;
    error.value = '';
    if (shouldDebugPlayback(currentTrack.value)) {
      logPlaybackDebug('audio-loadstart', {
        currentSrc: audio.currentSrc,
        crossOrigin: audio.crossOrigin,
        networkState: audio.networkState,
        readyState: audio.readyState,
      });
    }
  });

  audio.addEventListener('canplay', () => {
    if (!audio.currentSrc) return;
    loading.value = false;
    error.value = '';
    if (shouldDebugPlayback(currentTrack.value)) {
      logPlaybackDebug('audio-canplay', {
        currentSrc: audio.currentSrc,
        duration: audio.duration,
        networkState: audio.networkState,
        readyState: audio.readyState,
      });
    }
  });

  audio.addEventListener('ended', () => {
    loading.value = false;
    syncPlaybackPosition(audio.currentTime);
    syncCurrentHistory(true);
    playNext();
  });

  audio.addEventListener('play', () => {
    isPlaying.value = true;
    updateMediaSession();
  });

  audio.addEventListener('playing', () => {
    isPlaying.value = true;
    loading.value = false;
    error.value = '';
    if (shouldDebugPlayback(currentTrack.value)) {
      logPlaybackDebug('audio-playing', {
        currentSrc: audio.currentSrc,
        currentTime: audio.currentTime,
        networkState: audio.networkState,
        readyState: audio.readyState,
      });
    }
    updateMediaSession();
  });

  audio.addEventListener('pause', () => {
    isPlaying.value = false;
    syncPlaybackPosition(audio.currentTime);
    syncCurrentHistory(false);
    persistSession(true);
    updateMediaSession();
  });

  audio.addEventListener('waiting', () => {
    if (currentTrack.value && audio.currentSrc) {
      loading.value = true;
    }
  });

  audio.addEventListener('error', () => {
    if (isResettingAudio || !audio.currentSrc || audio.networkState === HTMLMediaElement.NETWORK_EMPTY) {
      return;
    }
    if (shouldDebugPlayback(currentTrack.value)) {
      logPlaybackError('audio-error', {
        currentSrc: audio.currentSrc,
        networkState: audio.networkState,
        readyState: audio.readyState,
        errorCode: audio.error?.code ?? null,
        errorMessage: audio.error?.message ?? null,
        track: currentTrack.value,
      });
    }
    loading.value = false;
    isPlaying.value = false;
    updateMediaSession();
    void tryAutoSwitch();
  });

  function resetAudio() {
    isResettingAudio = true;
    audio.pause();
    audio.removeAttribute('src');
    audio.load();
    isResettingAudio = false;
    isPlaying.value = false;
    duration.value = 0;
    syncPlaybackPosition(0);
  }

  function cancelPendingLoad() {
    loadToken++;
  }

  async function tryAutoSwitch() {
    const track = currentTrack.value;
    if (!track) {
      error.value = '音频加载失败，请重试';
      return;
    }

    const candidates = AUTO_SWITCH_SOURCE_VALUES.filter((s) => s !== track.source);
    if (!candidates.length) {
      error.value = '当前没有可用的自动换源候选源';
      return;
    }
    const source = candidates[autoSwitchAttempt % candidates.length];
    autoSwitchAttempt++;

    if (autoSwitchAttempt > candidates.length) {
      autoSwitchAttempt = 0;
      error.value = '所有音源均不可用，请稍后重试';
      return;
    }

    error.value = `正在尝试切换音源: ${source}...`;
    loading.value = true;

    try {
      const keyword = `${track.name} ${track.artist}`;
      const results = await musicApi.search(source, keyword, 10, 1);
      if (!results.length) throw new Error('no results');

      // 找最佳匹配（名称最接近的）
      const best = results.find(
        (r) => r.name.toLowerCase().includes(track.name.toLowerCase())
      ) ?? results[0];

      const switched: Track = {
        ...fromSearchResult(best),
        coverUrl: track.coverUrl,
      };

      // 替换队列中当前曲目
      if (currentIndex.value >= 0 && currentIndex.value < queue.value.length) {
        queue.value[currentIndex.value] = switched;
      }

      error.value = '';
      autoSwitchAttempt = 0;
      void loadTrack(switched, currentIndex.value, { autoplay: true });
    } catch {
      error.value = `${source} 换源失败，请手动重试`;
      loading.value = false;
    }
  }

  function setPlaybackRate(rate: PlaybackRate) {
    if (!(PLAYBACK_RATES as readonly number[]).includes(rate)) return;
    playbackRate.value = rate;
    audio.playbackRate = rate;
    writeVersionedStorage(PLAYBACK_RATE_STORAGE_KEY, PLAYBACK_RATE_STORAGE_VERSION, rate);
  }

  function setSleepTimer(minutes: number | null) {
    // 清除现有定时器
    if (sleepTimerHandle !== null) {
      clearInterval(sleepTimerHandle);
      sleepTimerHandle = null;
    }
    sleepTimerEndTime.value = null;
    sleepTimerRemaining.value = null;

    if (minutes === null || minutes <= 0) return;

    const endTime = Date.now() + minutes * 60 * 1000;
    sleepTimerEndTime.value = endTime;
    sleepTimerRemaining.value = minutes * 60;

    sleepTimerHandle = setInterval(() => {
      const remaining = Math.round((sleepTimerEndTime.value! - Date.now()) / 1000);
      if (remaining <= 0) {
        sleepTimerRemaining.value = 0;
        clearInterval(sleepTimerHandle!);
        sleepTimerHandle = null;
        sleepTimerEndTime.value = null;
        sleepTimerRemaining.value = null;
        audio.pause();
      } else {
        sleepTimerRemaining.value = remaining;
      }
    }, 1000);
  }

  function fromSearchResult(r: SearchResult): Track {
    return media.attachTrackCover({
      id: toStr(r.id),
      name: r.name,
      artist: getArtistNames(r.artist),
      album: getAlbumName(r.album),
      pic_id: toStr(r.pic_id),
      lyric_id: toStr(r.lyric_id),
      source: r.source,
    });
  }

  function syncTrackCover(track: Pick<Track, 'id' | 'source'>, coverUrl: string) {
    let queueChanged = false;
    const nextQueue = queue.value.map((item) => {
      if (item.id !== track.id || item.source !== track.source || item.coverUrl === coverUrl) {
        return item;
      }

      queueChanged = true;
      return { ...item, coverUrl };
    });

    if (queueChanged) {
      queue.value = nextQueue;
    }

    if (currentTrack.value?.id === track.id && currentTrack.value?.source === track.source && currentTrack.value.coverUrl !== coverUrl) {
      currentTrack.value = { ...currentTrack.value, coverUrl };
      updateMediaSession();
    }

    let historyChanged = false;
    const nextHistory = history.value.map((item) => {
      if (item.id !== track.id || item.source !== track.source || item.coverUrl === coverUrl) {
        return item;
      }

      historyChanged = true;
      return { ...item, coverUrl };
    });

    if (historyChanged) {
      history.value = nextHistory;
    }
  }

  function pushHistory(track: Track) {
    const nextTrack = media.attachTrackCover(track);
    const now = Date.now();
    const historyId = `${nextTrack.source}-${nextTrack.id}-${now}`;
    currentHistoryId = historyId;

    const nextItem: HistoryItem = {
      ...nextTrack,
      historyId,
      playedAt: now,
      finishedAt: null,
      lastPosition: 0,
      durationSnapshot: 0,
      completed: false,
    };

    history.value = normalizeHistory([nextItem, ...history.value]);
  }

  function playHistory(item: HistoryItem) {
    const { historyId: _historyId, playedAt: _playedAt, finishedAt: _finishedAt, lastPosition: _lastPosition, durationSnapshot: _durationSnapshot, completed: _completed, ...track } = item;
    addToQueue(media.attachTrackCover(track), true);
  }

  function removeHistory(historyId: string) {
    history.value = history.value.filter((item) => item.historyId !== historyId);
  }

  function clearHistory() {
    history.value = [];
  }

  async function loadTrack(
    track: Track,
    index: number,
    options?: { autoplay?: boolean; startTime?: number; recordHistory?: boolean },
  ) {
    const autoplay = options?.autoplay ?? true;
    const startTime = options?.startTime ?? 0;
    const recordHistory = options?.recordHistory ?? autoplay;

    const token = ++loadToken;
    loading.value = true;
    error.value = '';
    autoSwitchAttempt = 0;
    syncCurrentHistory(false);
    track = media.attachTrackCover(track);
    if (index >= 0 && index < queue.value.length) {
      queue.value[index] = media.attachTrackCover(queue.value[index] ?? track);
      if (!queue.value[index].coverUrl && track.coverUrl) {
        queue.value[index] = { ...queue.value[index], coverUrl: track.coverUrl };
      }
      track = queue.value[index];
    }
    currentTrack.value = track;
    currentIndex.value = index;
    pendingRestoreTime = startTime;
    resetAudio();
    lyricLines.value = [];
    tlyricLines.value = [];
    currentLyricIndex.value = -1;

    try {
      if (!track.coverUrl) {
        try {
          const coverUrl = await media.ensureTrackCover(track);
          if (token !== loadToken) return;
          if (coverUrl) {
            syncTrackCover(track, coverUrl);
            library.syncTrackCover(track, coverUrl);
            track = media.attachTrackCover({ ...track, coverUrl });
            currentTrack.value = track;
          }
        } catch {}
      }

      let playbackUrl = '';
      let localPath: string | undefined;

      if (track.source === 'local') {
        localPath = track.url;
        playbackUrl = resolvePlayableAudioUrl(undefined, localPath);
        if (!playbackUrl) throw new Error('无法获取本地文件播放路径');
      } else {
        const res = await musicApi.getMusicUrl(track.source, track.id, preferredBitrate.value);
        if (token !== loadToken) return;
        playbackUrl = resolvePlayableAudioUrl(res.url, res.localPath);
        localPath = res.localPath;
        if (!playbackUrl) throw new Error('无法获取播放链接');
      }

      audio.crossOrigin = shouldUseAnonymousCors(track.source, localPath) ? 'anonymous' : null;
      if (shouldDebugPlayback(track)) {
        logPlaybackDebug('load-track-resolved', {
          track,
          apiUrl: playbackUrl,
          localPath: localPath ?? null,
          playbackUrl,
          crossOrigin: audio.crossOrigin,
        });
      }
      audio.src = playbackUrl;
      audio.load();

      if (autoplay) {
        await audio.play();
        if (token !== loadToken) return;
        if (shouldDebugPlayback(track)) {
          logPlaybackDebug('audio-play-invoked', {
            currentSrc: audio.currentSrc,
            paused: audio.paused,
            networkState: audio.networkState,
            readyState: audio.readyState,
          });
        }
      }

      if (recordHistory) {
        pushHistory(track);
      } else {
        currentHistoryId = null;
      }

      void musicApi.getLyric(track.source, track.lyric_id || track.id)
        .then(async (lyr) => {
          if (token !== loadToken) return;
          if (lyr.lyric) {
            lyricLines.value = parseLrc(lyr.lyric);
            if (lyr.tlyric) tlyricLines.value = parseLrc(lyr.tlyric);
            syncLyricIndexByTime(currentTime.value);
          } else {
            // 主源无歌词，尝试 lrclib 辅助歌词
            try {
              const aux = await musicApi.getAuxLyric(
                track.artist, track.name, track.album,
                duration.value > 0 ? Math.round(duration.value) : undefined,
              );
              if (token !== loadToken) return;
              if (aux.lyric) {
                lyricLines.value = parseLrc(aux.lyric);
                syncLyricIndexByTime(currentTime.value);
              }
            } catch {}
          }
        })
        .catch(() => {});

      updateMediaSession();
      persistSession(true);
    } catch (e: unknown) {
      if (token !== loadToken) return;
      if (shouldDebugPlayback(track)) {
        logPlaybackError('load-track-failed', {
          track,
          error: e instanceof Error ? e.message : e,
          currentSrc: audio.currentSrc,
          networkState: audio.networkState,
          readyState: audio.readyState,
        });
      }
      error.value = e instanceof Error ? e.message : '播放失败';
      loading.value = false;
    } finally {
      if (token === loadToken && !isPlaying.value) {
        loading.value = false;
      }
    }
  }

  function addToQueue(track: Track, playNow = false) {
    const nextTrack = media.attachTrackCover(track);
    const idx = queue.value.findIndex((item) => item.id === nextTrack.id && item.source === nextTrack.source);
    if (idx >= 0) {
      if (nextTrack.coverUrl && queue.value[idx]?.coverUrl !== nextTrack.coverUrl) {
        syncTrackCover(nextTrack, nextTrack.coverUrl);
        library.syncTrackCover(nextTrack, nextTrack.coverUrl);
      }
      if (playNow) void loadTrack(queue.value[idx], idx, { autoplay: true });
      return;
    }

    queue.value.push(nextTrack);
    if (playNow) void loadTrack(nextTrack, queue.value.length - 1, { autoplay: true });
  }

  function playTrack(index: number) {
    if (index < 0 || index >= queue.value.length) return;
    void loadTrack(queue.value[index], index, { autoplay: true });
  }

  function playNext() {
    if (!queue.value.length) return;
    if (playMode.value === 'single') {
      audio.currentTime = 0;
      void audio.play().catch((e: unknown) => {
        error.value = e instanceof Error ? e.message : '播放失败';
      });
      return;
    }
    if (playMode.value === 'random') {
      playTrack(Math.floor(Math.random() * queue.value.length));
      return;
    }
    playTrack((currentIndex.value + 1) % queue.value.length);
  }

  function playPrev() {
    if (!queue.value.length) return;
    if (playMode.value === 'random') {
      playTrack(Math.floor(Math.random() * queue.value.length));
      return;
    }
    playTrack((currentIndex.value - 1 + queue.value.length) % queue.value.length);
  }

  async function togglePlay() {
    if (!currentTrack.value) return;

    try {
      if (audio.paused) {
        error.value = '';
        await audio.play();
      } else {
        audio.pause();
      }
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '播放失败';
    }
  }

  function seek(time: number) {
    audio.currentTime = time;
    syncPlaybackPosition(time);
    syncCurrentHistory(false);
    persistSession(true);
  }

  function setVolume(v: number) {
    volume.value = v;
    audio.volume = v;
    persistSession(true);
  }

  function setPreferredBitrate(v: Bitrate) {
    preferredBitrate.value = BITRATES.includes(v) ? v : 320;
  }

  function togglePlayMode() {
    const modes: PlayMode[] = ['sequence', 'random', 'single'];
    const i = modes.indexOf(playMode.value);
    playMode.value = modes[(i + 1) % modes.length];
  }

  function removeFromQueue(index: number) {
    queue.value.splice(index, 1);
    if (index === currentIndex.value) {
      if (!queue.value.length) {
        cancelPendingLoad();
        currentTrack.value = null;
        currentIndex.value = -1;
        error.value = '';
        lyricLines.value = [];
        tlyricLines.value = [];
        currentLyricIndex.value = -1;
        currentHistoryId = null;
        resetAudio();
      } else {
        playTrack(Math.min(index, queue.value.length - 1));
      }
    } else if (index < currentIndex.value) {
      currentIndex.value--;
    }
  }

  function clearQueue() {
    cancelPendingLoad();
    queue.value = [];
    currentIndex.value = -1;
    currentTrack.value = null;
    error.value = '';
    lyricLines.value = [];
    tlyricLines.value = [];
    currentLyricIndex.value = -1;
    loading.value = false;
    currentHistoryId = null;
    resetAudio();
    persistSession(true);
  }

  function setMiniPlayerVisible(visible: boolean) {
    showMiniPlayer.value = visible;
  }

  function applyMiniPlayerStateSnapshot(snapshot: MiniPlayerStateSnapshot) {
    currentTrack.value = snapshot.currentTrack ? media.attachTrackCover(snapshot.currentTrack) : null;
    isPlaying.value = snapshot.isPlaying;
    duration.value = snapshot.duration;
    currentTime.value = snapshot.currentTime;
    playMode.value = snapshot.playMode;
  }

  function getMiniPlayerStateSnapshot(): MiniPlayerStateSnapshot {
    return {
      currentTrack: currentTrack.value,
      isPlaying: isPlaying.value,
      duration: duration.value,
      currentTime: currentTime.value,
      playMode: playMode.value,
    };
  }

  function hydrateSession() {
    if (!initialSession.currentTrack || initialSession.currentIndex < 0 || !initialSession.queue.length) {
      registerMediaSessionHandlers();
      updateMediaSession();
      return;
    }

    void loadTrack(initialSession.currentTrack, initialSession.currentIndex, {
      autoplay: false,
      startTime: initialSession.currentTime,
      recordHistory: false,
    });
    registerMediaSessionHandlers();
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('beforeunload', () => {
      syncCurrentHistory(false);
      persistSession(true);
    });
  }

  hydrateSession();

  return {
    queue,
    currentIndex,
    currentTrack,
    isPlaying,
    duration,
    currentTime,
    playbackTimeUpdatedAt,
    volume,
    playMode,
    preferredBitrate,
    playbackRate,
    showMiniPlayer,
    sleepTimerEndTime,
    sleepTimerRemaining,
    history,
    lyricLines,
    tlyricLines,
    currentLyricIndex,
    currentLyricText,
    loading,
    error,
    showLyric,
    fromSearchResult,
    syncTrackCover,
    addToQueue,
    playTrack,
    playNext,
    playPrev,
    togglePlay,
    seek,
    setVolume,
    setPreferredBitrate,
    setPlaybackRate,
    setSleepTimer,
    togglePlayMode,
    playHistory,
    removeHistory,
    clearHistory,
    setMiniPlayerVisible,
    applyMiniPlayerStateSnapshot,
    getMiniPlayerStateSnapshot,
    removeFromQueue,
    clearQueue,
    PLAYBACK_RATES,
  };
});

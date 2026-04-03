import { invoke } from '@tauri-apps/api/core';
import { readCache, writeCache } from '@/utils/cache';

export type MusicSource =
  | 'netease'
  | 'kuwo'
  | 'joox'
  | 'bilibili'
  | 'tencent'
  | 'migu'
  | 'kugou'
  | 'ximalaya'
  | 'apple'
  | 'ytmusic'
  | 'spotify'
  | 'tidal'
  | 'qobuz'
  | 'deezer'
  | 'aisearch';

export type SearchKind = 'track' | 'album';

export type MusicSourceState = 'available' | 'limited' | 'disabled';

export interface MusicSourceMeta {
  value: MusicSource;
  label: string;
  visible: boolean;
  selectable: boolean;
  defaultEnabled: boolean;
  autoSwitch: boolean;
  state: MusicSourceState;
  note: string;
}

export const SOURCE_CATALOG: MusicSourceMeta[] = [
  {
    value: 'netease',
    label: '网易云',
    visible: true,
    selectable: true,
    defaultEnabled: true,
    autoSwitch: true,
    state: 'available',
    note: '当前 GD Studio 接口最稳定。',
  },
  {
    value: 'kuwo',
    label: '酷我',
    visible: true,
    selectable: true,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'limited',
    note: '搜索可用，但播放直链经常为空。',
  },
  {
    value: 'joox',
    label: 'JOOX',
    visible: true,
    selectable: true,
    defaultEnabled: true,
    autoSwitch: true,
    state: 'available',
    note: '当前 HK 节点可用。',
  },
  {
    value: 'bilibili',
    label: 'Bilibili',
    visible: true,
    selectable: true,
    defaultEnabled: true,
    autoSwitch: true,
    state: 'available',
    note: '当前可播放，但码率是动态的。',
  },
  {
    value: 'tencent',
    label: 'QQ 音乐',
    visible: true,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: '当前后端返回 source unsupported。',
  },
  {
    value: 'migu',
    label: '咪咕',
    visible: false,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: 'CN 节点当前返回 526。',
  },
  {
    value: 'kugou',
    label: '酷狗',
    visible: false,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: 'CN 节点当前返回 526。',
  },
  {
    value: 'ximalaya',
    label: '喜马拉雅',
    visible: false,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: 'CN 节点当前返回 526。',
  },
  {
    value: 'apple',
    label: 'Apple Music',
    visible: true,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: '当前后端返回 source unsupported。',
  },
  {
    value: 'ytmusic',
    label: 'YouTube Music',
    visible: true,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: 'US 节点当前返回 Invalid request。',
  },
  {
    value: 'spotify',
    label: 'Spotify',
    visible: true,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: '当前后端返回 source unsupported。',
  },
  {
    value: 'tidal',
    label: 'Tidal',
    visible: true,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: '当前后端返回 source unsupported。',
  },
  {
    value: 'qobuz',
    label: 'Qobuz',
    visible: true,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: 'US 节点当前返回 Invalid request。',
  },
  {
    value: 'deezer',
    label: 'Deezer',
    visible: false,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: '当前网页版已不再展示，接口也不可用。',
  },
  {
    value: 'aisearch',
    label: 'AI 搜索',
    visible: false,
    selectable: false,
    defaultEnabled: false,
    autoSwitch: false,
    state: 'disabled',
    note: '当前网页版已不再展示该入口。',
  },
];

export const SOURCES = SOURCE_CATALOG.filter((item) => item.visible);

export const DEFAULT_ENABLED_SOURCES = SOURCE_CATALOG
  .filter((item) => item.defaultEnabled)
  .map((item) => item.value);

export const AUTO_SWITCH_SOURCE_VALUES = SOURCE_CATALOG
  .filter((item) => item.autoSwitch)
  .map((item) => item.value);

const SOURCE_META_MAP = new Map(SOURCE_CATALOG.map((item) => [item.value, item] as const));

export function getSourceMeta(source: string) {
  return SOURCE_META_MAP.get(source as MusicSource);
}

export function isSelectableSource(source: string): source is MusicSource {
  return getSourceMeta(source)?.selectable ?? false;
}

export interface SearchResult {
  id: string | number;
  name: string;
  artist: Array<string | { name: string }>;
  album: string | { name: string };
  pic_id: string | number;
  url_id: string | number;
  lyric_id: string | number;
  source: string;
}

export interface MusicUrl {
  url?: string;
  br?: unknown;
  size?: unknown;
}

export interface PicUrl {
  url?: string;
}

export const DEFAULT_PIC_SIZE = 500;

export interface LyricResult {
  lyric?: string;
  tlyric?: string;
}

export interface AuxLyricResult {
  lyric?: string;
  tlyric?: string;
  source: 'aux';
}

export interface RecommendPlaylist {
  id: string | number;
  name: string;
  cover?: string;
  source: string;
}

export interface PlaylistDetail {
  [key: string]: unknown;
}

export interface UserPlaylists {
  [key: string]: unknown;
}

export function getArtistNames(artist: SearchResult['artist']): string {
  if (!artist?.length) return 'Unknown Artist';
  return artist
    .map((item) => (typeof item === 'string' ? item : item.name ?? ''))
    .filter(Boolean)
    .join(' / ');
}

export function getAlbumName(album: SearchResult['album']): string {
  if (!album) return 'Unknown Album';
  return typeof album === 'string' ? album : album.name ?? 'Unknown Album';
}

export function toStr(id: unknown): string {
  if (id === null || id === undefined) return '';
  return String(id);
}

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

function isSearchResult(value: unknown): value is SearchResult {
  return (
    isObject(value) &&
    (typeof value.id === 'string' || typeof value.id === 'number') &&
    typeof value.name === 'string' &&
    Array.isArray(value.artist) &&
    typeof value.source === 'string'
  );
}

function isSearchResultList(value: unknown): value is SearchResult[] {
  return Array.isArray(value) && value.every(isSearchResult);
}

function isMusicUrl(value: unknown): value is MusicUrl {
  return isObject(value) && (!('url' in value) || typeof value.url === 'string' || value.url === undefined);
}

function isPicUrl(value: unknown): value is PicUrl {
  return isObject(value) && (!('url' in value) || typeof value.url === 'string' || value.url === undefined);
}

function isLyricResult(value: unknown): value is LyricResult {
  return (
    isObject(value) &&
    (!('lyric' in value) || typeof value.lyric === 'string' || value.lyric === undefined) &&
    (!('tlyric' in value) || typeof value.tlyric === 'string' || value.tlyric === undefined)
  );
}

function isGenericObject(value: unknown): value is Record<string, unknown> {
  return isObject(value);
}

function resolveSearchSource(source: MusicSource, kind: SearchKind): string {
  return kind === 'album' ? `${source}_album` : source;
}

const inFlightRequests = new Map<string, Promise<unknown>>();

async function invokeWithCache<T>(
  cacheKey: string,
  ttlMs: number,
  fetcher: () => Promise<T>,
  validate?: (candidate: unknown) => candidate is T,
): Promise<T> {
  const cached = readCache(cacheKey, validate);
  if (cached && !cached.stale) {
    return cached.value;
  }

  const pending = inFlightRequests.get(cacheKey) as Promise<T> | undefined;
  if (pending) {
    try {
      return await pending;
    } catch (error) {
      if (cached) return cached.value;
      throw error;
    }
  }

  const request = (async () => {
    try {
      const fresh = await fetcher();
      writeCache(cacheKey, fresh, ttlMs);
      return fresh;
    } finally {
      inFlightRequests.delete(cacheKey);
    }
  })();

  inFlightRequests.set(cacheKey, request);

  try {
    return await request;
  } catch (error) {
    if (cached) return cached.value;
    throw error;
  }
}

function keyOf(...parts: Array<string | number>) {
  return `fashion:cache:${parts.join(':')}`;
}

function picCacheKey(source: string, id: string, size = DEFAULT_PIC_SIZE) {
  return keyOf('pic', source, id, size);
}

export function readCachedPicUrl(source: string, id: string, size = DEFAULT_PIC_SIZE): string | null {
  const normalizedId = toStr(id);
  if (!normalizedId) return null;

  const cached = readCache(picCacheKey(source, normalizedId, size), isPicUrl);
  return cached?.value.url ?? null;
}

export const musicApi = {
  search: (source: MusicSource, keyword: string, count = 30, page = 1, kind: SearchKind = 'track') => {
    const querySource = resolveSearchSource(source, kind);
    return invokeWithCache(
      keyOf('search', querySource, kind, encodeURIComponent(keyword.trim()), count, page),
      1000 * 60 * 60 * 6,
      () => invoke<SearchResult[]>('search_music', { source: querySource, keyword, count, page }),
      isSearchResultList,
    );
  },
  getMusicUrl: (source: string, id: string, br = 320) =>
    invokeWithCache(
      keyOf('url', source, id, br),
      1000 * 60 * 10,
      () => invoke<MusicUrl>('get_music_url', { source, id, br }),
      isMusicUrl,
    ),
  getPicUrl: (source: string, id: string, size = DEFAULT_PIC_SIZE) =>
    invokeWithCache(
      picCacheKey(source, id, size),
      1000 * 60 * 60 * 24 * 30,
      () => invoke<PicUrl>('get_pic_url', { source, id, size }),
      isPicUrl,
    ),
  getLyric: (source: string, id: string) =>
    invokeWithCache(
      keyOf('lyric', source, id),
      1000 * 60 * 60 * 24 * 30,
      () => invoke<LyricResult>('get_lyric', { source, id }),
      isLyricResult,
    ),
  getPlaylistDetail: (id: string, source: MusicSource = 'netease', limit = 50, offset = 0) =>
    invokeWithCache(
      keyOf('playlist', source, id, limit, offset),
      1000 * 60 * 30,
      () => invoke<PlaylistDetail>('get_playlist_detail', { id, source, limit, offset }),
      isGenericObject,
    ),
  getUserPlaylists: (uid: string) =>
    invokeWithCache(
      keyOf('userlist', uid),
      1000 * 60 * 30,
      () => invoke<UserPlaylists>('get_user_playlists', { uid }),
      isGenericObject,
    ),
  getAuxLyric: (artist: string, title: string, albumName?: string, duration?: number) =>
    invokeWithCache(
      keyOf('aux_lyric', artist, title, albumName ?? '', duration ?? 0),
      1000 * 60 * 60 * 24 * 30,
      () => invoke<AuxLyricResult>('get_aux_lyric', { artist, title, albumName, duration }),
      (value): value is AuxLyricResult => isObject(value) && 'source' in value,
    ),
  getRecommendPlaylist: (id: string, source: MusicSource = 'netease', limit = 50, offset = 0) =>
    invokeWithCache(
      keyOf('recommend', source, id, limit, offset),
      1000 * 60 * 30,
      () => invoke<PlaylistDetail>('get_recommend_playlist', { id, source, limit, offset }),
      isGenericObject,
    ),
};

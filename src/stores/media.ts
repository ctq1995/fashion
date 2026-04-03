import { defineStore } from 'pinia';
import { reactive } from 'vue';
import { DEFAULT_PIC_SIZE, musicApi, readCachedPicUrl, toStr } from '@/api/music';

interface CoverTarget {
  source: string;
  pic_id?: string | number;
  id?: string | number;
  coverUrl?: string;
}

function resolveCoverId(target: CoverTarget) {
  return toStr(target.pic_id) || toStr(target.id);
}

function normalizeCoverUrl(url: string | null | undefined) {
  const normalized = typeof url === 'string' ? url.trim() : '';
  return normalized || null;
}

export const useMediaStore = defineStore('media', () => {
  const coverUrls = reactive<Record<string, string>>({});
  const inFlightCovers = new Map<string, Promise<string | null>>();

  function coverCacheKey(source: string, picId: string) {
    return `${source}:${picId}`;
  }

  function coverKeyForTrack(target: CoverTarget) {
    const picId = resolveCoverId(target);
    return picId ? coverCacheKey(target.source, picId) : '';
  }

  function rememberCover(source: string, picId: string, url: string | null | undefined) {
    const normalized = normalizeCoverUrl(url);
    if (!picId || !normalized) return null;

    const key = coverCacheKey(source, picId);
    if (coverUrls[key] !== normalized) {
      coverUrls[key] = normalized;
    }
    return normalized;
  }

  function primeTrackCover(target: CoverTarget) {
    const picId = resolveCoverId(target);
    if (!picId) return null;

    return (
      rememberCover(target.source, picId, target.coverUrl) ??
      rememberCover(target.source, picId, readCachedPicUrl(target.source, picId, DEFAULT_PIC_SIZE))
    );
  }

  function getTrackCoverUrl(target: CoverTarget) {
    const key = coverKeyForTrack(target);
    if (key && coverUrls[key]) return coverUrls[key];
    return normalizeCoverUrl(target.coverUrl);
  }

  function attachTrackCover<T extends CoverTarget>(target: T): T {
    const coverUrl = primeTrackCover(target) ?? getTrackCoverUrl(target);
    if (!coverUrl || target.coverUrl === coverUrl) return target;
    return { ...target, coverUrl } as T;
  }

  async function ensureTrackCover(target: CoverTarget): Promise<string | null> {
    const coverUrl = primeTrackCover(target) ?? getTrackCoverUrl(target);
    if (coverUrl) return coverUrl;

    const picId = resolveCoverId(target);
    if (!picId) return null;

    const key = coverCacheKey(target.source, picId);
    const pending = inFlightCovers.get(key);
    if (pending) {
      return pending;
    }

    const request = (async () => {
      try {
        const pic = await musicApi.getPicUrl(target.source, picId, DEFAULT_PIC_SIZE);
        return rememberCover(target.source, picId, pic.url);
      } finally {
        inFlightCovers.delete(key);
      }
    })();

    inFlightCovers.set(key, request);
    return request;
  }

  function clearRuntimeCoverCache() {
    inFlightCovers.clear();

    for (const key of Object.keys(coverUrls)) {
      delete coverUrls[key];
    }
  }

  return {
    coverUrls,
    coverCacheKey,
    coverKeyForTrack,
    rememberCover,
    primeTrackCover,
    getTrackCoverUrl,
    attachTrackCover,
    ensureTrackCover,
    clearRuntimeCoverCache,
  };
});

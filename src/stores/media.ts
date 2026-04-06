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

function coverFailureKey(source: string, url: string) {
  return `${source}:${url}`;
}

export const useMediaStore = defineStore('media', () => {
  const coverUrls = reactive<Record<string, string>>({});
  const failedCoverUrls = reactive<Record<string, true>>({});
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
    if (failedCoverUrls[coverFailureKey(source, normalized)]) return null;

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

    const normalized = normalizeCoverUrl(target.coverUrl);
    if (!normalized) return null;
    if (failedCoverUrls[coverFailureKey(target.source, normalized)]) return null;
    return normalized;
  }

  function markCoverLoadFailed(target: CoverTarget) {
    const normalized = normalizeCoverUrl(target.coverUrl);
    if (!normalized) return;

    failedCoverUrls[coverFailureKey(target.source, normalized)] = true;

    const key = coverKeyForTrack(target);
    if (key && coverUrls[key] === normalized) {
      delete coverUrls[key];
    }
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

    for (const key of Object.keys(failedCoverUrls)) {
      delete failedCoverUrls[key];
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
    markCoverLoadFailed,
    clearRuntimeCoverCache,
  };
});

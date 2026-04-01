import { readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';

const CACHE_VERSION = 1;

interface CacheEntry<T> {
  value: T;
  expiresAt: number;
}

function isCacheEntry<T>(value: unknown, validate?: (candidate: unknown) => candidate is T): value is CacheEntry<T> {
  if (!value || typeof value !== 'object') return false;
  const entry = value as Record<string, unknown>;
  const valueOk = validate ? validate(entry.value) : 'value' in entry;
  return typeof entry.expiresAt === 'number' && valueOk;
}

export function readCache<T>(
  key: string,
  validate?: (candidate: unknown) => candidate is T,
): { value: T; stale: boolean } | null {
  const entry = readVersionedStorage<CacheEntry<T> | null>(key, CACHE_VERSION, {
    fallback: null,
    validate: (value): value is CacheEntry<T> => value !== null && isCacheEntry(value, validate),
  });

  if (!entry) return null;

  return {
    value: entry.value,
    stale: Date.now() > entry.expiresAt,
  };
}

export function writeCache<T>(key: string, value: T, ttlMs: number) {
  writeVersionedStorage<CacheEntry<T>>(key, CACHE_VERSION, {
    value,
    expiresAt: Date.now() + ttlMs,
  });
}

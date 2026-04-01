import { beforeEach, describe, expect, it } from 'vitest';
import { readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';

describe('persistence', () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it('writes and reads versioned values', () => {
    writeVersionedStorage('demo', 1, { foo: 'bar' });

    const result = readVersionedStorage('demo', 1, {
      fallback: { foo: 'fallback' },
      validate: (value): value is { foo: string } =>
        !!value && typeof value === 'object' && typeof (value as { foo?: unknown }).foo === 'string',
    });

    expect(result.foo).toBe('bar');
  });

  it('migrates legacy values', () => {
    window.localStorage.setItem('legacy', '320');

    const result = readVersionedStorage('legacy', 1, {
      fallback: 128,
      validate: (value): value is number => typeof value === 'number',
      migrateLegacy: (raw) => {
        const parsed = Number(raw);
        return Number.isFinite(parsed) ? parsed : null;
      },
    });

    expect(result).toBe(320);
  });
});

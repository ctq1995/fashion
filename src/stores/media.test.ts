import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';

const apiMocks = vi.hoisted(() => ({
  getPicUrl: vi.fn(),
  readCachedPicUrl: vi.fn(),
}));

vi.mock('@/api/music', () => ({
  DEFAULT_PIC_SIZE: 500,
  musicApi: {
    getPicUrl: apiMocks.getPicUrl,
  },
  readCachedPicUrl: apiMocks.readCachedPicUrl,
  toStr: (id: unknown) => (id === null || id === undefined ? '' : String(id)),
}));

import { useMediaStore } from '@/stores/media';

describe('media store', () => {
  const originalImage = globalThis.Image;

  beforeEach(() => {
    setActivePinia(createPinia());
    apiMocks.getPicUrl.mockReset();
    apiMocks.readCachedPicUrl.mockReset();
    globalThis.Image = originalImage;
  });

  it('primes covers from persisted cache before making requests', () => {
    apiMocks.readCachedPicUrl.mockReturnValue('https://cdn.example/cached.jpg');

    const store = useMediaStore();
    const track = { source: 'netease', pic_id: 'pic-1', id: '1' };

    expect(store.primeTrackCover(track)).toBe('https://cdn.example/cached.jpg');
    expect(store.getTrackCoverUrl(track)).toBe('https://cdn.example/cached.jpg');
    expect(apiMocks.getPicUrl).not.toHaveBeenCalled();
  });

  it('dedupes in-flight cover fetches and reuses the resolved url', async () => {
    apiMocks.readCachedPicUrl.mockReturnValue(null);
    apiMocks.getPicUrl.mockResolvedValue({ url: 'https://cdn.example/live.jpg' });

    const store = useMediaStore();
    const track = { source: 'netease', pic_id: 'pic-2', id: '2' };

    const [first, second] = await Promise.all([
      store.ensureTrackCover(track),
      store.ensureTrackCover(track),
    ]);

    expect(first).toBe('https://cdn.example/live.jpg');
    expect(second).toBe('https://cdn.example/live.jpg');
    expect(apiMocks.getPicUrl).toHaveBeenCalledTimes(1);
    expect(store.getTrackCoverUrl(track)).toBe('https://cdn.example/live.jpg');
  });

  it('drops broken cover urls after image load failure', () => {
    apiMocks.readCachedPicUrl.mockReturnValue(null);

    let capturedImage: InstanceType<typeof Image> | { onerror: null | (() => void); src: string } | null = null;
    globalThis.Image = class {
      onerror: null | (() => void) = null;
      src = '';
      constructor() {
        capturedImage = this;
      }
    } as unknown as typeof Image;

    const store = useMediaStore();
    const track = { source: 'netease', pic_id: 'pic-3', id: '3', coverUrl: 'https://cdn.example/broken.jpg' };

    expect(store.getTrackCoverUrl(track)).toBe('https://cdn.example/broken.jpg');

    store.markCoverLoadFailed(track);
    (capturedImage as { onerror?: (() => void) | null } | null)?.onerror?.();

    expect(store.getTrackCoverUrl(track)).toBe(null);
  });
});

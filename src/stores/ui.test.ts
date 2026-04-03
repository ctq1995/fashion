import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { useUiStore } from '@/stores/ui';

describe('ui store', () => {
  beforeEach(() => {
    window.localStorage.clear();
    setActivePinia(createPinia());
  });

  it('persists desktop lyric settings', async () => {
    const store = useUiStore();

    store.setLyricSettings({
      locked: true,
      scrollSpeed: 96,
      baseColor: '#F8F6FF',
      highlightColor: '#FF7A59',
      windowPosition: { x: 128, y: 256 },
    });

    await nextTick();

    setActivePinia(createPinia());
    const restored = useUiStore();

    expect(restored.lyricSettings.locked).toBe(true);
    expect(restored.lyricSettings.scrollSpeed).toBe(96);
    expect(restored.lyricSettings.baseColor).toBe('#F8F6FF');
    expect(restored.lyricSettings.highlightColor).toBe('#FF7A59');
    expect(restored.lyricSettings.windowPosition).toEqual({ x: 128, y: 256 });
  });

  it('migrates older desktop lyric settings envelopes', () => {
    window.localStorage.setItem(
      'fashion:desktop-lyric-settings',
      JSON.stringify({
        version: 1,
        updatedAt: Date.now(),
        data: {
          locked: true,
          scrollSpeed: 96,
          windowPosition: { x: 128, y: 256 },
        },
      }),
    );

    const store = useUiStore();

    expect(store.lyricSettings.locked).toBe(true);
    expect(store.lyricSettings.scrollSpeed).toBe(96);
    expect(store.lyricSettings.baseColor).toBe('#FFFFFF');
    expect(store.lyricSettings.highlightColor).toBe('#16D6A0');
    expect(store.lyricSettings.windowPosition).toEqual({ x: 128, y: 256 });
  });

  it('adds gequbao to previously saved enabled sources', () => {
    window.localStorage.setItem(
      'fashion:enabled-sources',
      JSON.stringify({
        version: 1,
        updatedAt: Date.now(),
        data: ['netease', 'joox'],
      }),
    );

    const store = useUiStore();

    expect(store.enabledToolbarSources).toContain('netease');
    expect(store.enabledToolbarSources).toContain('joox');
    expect(store.enabledToolbarSources).toContain('gequbao');
  });
});

import { describe, expect, it } from 'vitest';
import {
  buildTranslatedLineMap,
  clampLyricProgress,
  resolveLyricRenderTime,
  resolveLyricLineDuration,
  resolveLyricLineProgress,
} from '@/utils/lyrics';

describe('lyrics helpers', () => {
  it('uses the real next-line gap for fast lyric progress and caps long gaps', () => {
    const lines = [
      { time: 0, text: 'a' },
      { time: 0.1, text: 'b' },
      { time: 8.5, text: 'c' },
    ];

    expect(resolveLyricLineDuration(lines, 0)).toBe(0.65);
    expect(resolveLyricLineDuration(lines, 1)).toBe(4.2);
  });

  it('matches translated lines by nearest timestamp', () => {
    const lines = [
      { time: 1, text: 'line-1' },
      { time: 5, text: 'line-2' },
    ];
    const translated = [
      { time: 1.2, text: 'trans-1' },
      { time: 5.1, text: 'trans-2' },
      { time: 9, text: 'ignored' },
    ];

    const map = buildTranslatedLineMap(lines, translated);

    expect(map.get(0)).toBe('trans-1');
    expect(map.get(1)).toBe('trans-2');
    expect(map.size).toBe(2);
  });

  it('clamps lyric progress between 0 and 1', () => {
    expect(clampLyricProgress(-1, 0, 2)).toBe(0);
    expect(clampLyricProgress(1, 0, 2)).toBe(0.5);
    expect(clampLyricProgress(4, 0, 2)).toBe(1);
  });

  it('applies a positive lyric offset as a render delay', () => {
    expect(resolveLyricRenderTime(10, 2000)).toBe(8);
    expect(resolveLyricRenderTime(10, -1000)).toBe(11);
    expect(resolveLyricRenderTime(1, 3000)).toBe(0);
  });

  it('resolves line progress from line timing', () => {
    const lines = [
      { time: 10, text: 'a' },
      { time: 12, text: 'b' },
    ];

    expect(resolveLyricLineProgress(lines, 0, 11)).toBe(0.5);
    expect(resolveLyricLineProgress(lines, 1, 13)).toBe(0.5);
    expect(resolveLyricLineProgress(lines, 0, 12)).toBe(1);
  });
});

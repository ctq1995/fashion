import { describe, expect, it } from 'vitest';
import { formatDuration } from '@/utils/formatters';

describe('formatDuration', () => {
  it('returns placeholder for empty or invalid durations', () => {
    expect(formatDuration(undefined)).toBe('--:--');
    expect(formatDuration(null)).toBe('--:--');
    expect(formatDuration(0)).toBe('--:--');
    expect(formatDuration(Number.NaN)).toBe('--:--');
  });

  it('formats minute-second durations', () => {
    expect(formatDuration(5)).toBe('00:05');
    expect(formatDuration(65)).toBe('01:05');
    expect(formatDuration(3599)).toBe('59:59');
  });

  it('formats hour-minute-second durations', () => {
    expect(formatDuration(3600)).toBe('1:00:00');
    expect(formatDuration(3665)).toBe('1:01:05');
  });
});

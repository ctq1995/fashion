export interface TimedLyricLine {
  time: number;
  text: string;
}

const DEFAULT_LINE_DURATION = 2.4;
const MIN_LINE_DURATION = 0.65;
const MAX_LINE_DURATION = 4.2;
const TRANSLATION_TOLERANCE_SECONDS = 0.8;

export function resolveLyricRenderTime(currentTime: number, lyricOffsetMs = 0) {
  if (!Number.isFinite(currentTime)) {
    return 0;
  }

  const offsetSeconds = Number.isFinite(lyricOffsetMs) ? lyricOffsetMs / 1000 : 0;
  return Math.max(0, currentTime - offsetSeconds);
}

export function clampLyricProgress(currentTime: number, lineTime: number, duration: number) {
  if (!Number.isFinite(currentTime) || !Number.isFinite(lineTime) || !Number.isFinite(duration) || duration <= 0) {
    return 0;
  }

  return Math.min(1, Math.max(0, (currentTime - lineTime) / duration));
}

export function resolveLyricLineDuration(
  lines: TimedLyricLine[],
  index: number,
  minDuration = MIN_LINE_DURATION,
  maxDuration = MAX_LINE_DURATION,
) {
  const current = lines[index];
  if (!current) return DEFAULT_LINE_DURATION;

  const previous = lines[index - 1];
  const next = lines[index + 1];

  if (next && next.time > current.time) {
    return Math.min(maxDuration, Math.max(minDuration, next.time - current.time));
  }

  let duration = DEFAULT_LINE_DURATION;
  if (previous && current.time > previous.time) {
    duration = current.time - previous.time;
  }

  return Math.min(maxDuration, Math.max(minDuration, duration));
}

export function resolveLyricLineProgress(
  lines: TimedLyricLine[],
  index: number,
  currentTime: number,
) {
  const current = lines[index];
  if (!current) return 0;

  return clampLyricProgress(
    currentTime,
    current.time,
    resolveLyricLineDuration(lines, index),
  );
}

export function buildTranslatedLineMap(
  lines: TimedLyricLine[],
  translatedLines: TimedLyricLine[],
  toleranceSeconds = TRANSLATION_TOLERANCE_SECONDS,
) {
  const map = new Map<number, string>();
  if (!lines.length || !translatedLines.length) return map;

  for (let index = 0; index < lines.length; index += 1) {
    const current = lines[index];
    let bestText = '';
    let bestDelta = toleranceSeconds;

    for (const line of translatedLines) {
      const delta = Math.abs(line.time - current.time);
      if (!line.text || delta >= bestDelta) continue;
      bestDelta = delta;
      bestText = line.text;
    }

    if (bestText) {
      map.set(index, bestText);
    }
  }

  return map;
}

export function findTranslatedLine(
  lines: TimedLyricLine[],
  translatedLines: TimedLyricLine[],
  index: number,
) {
  if (index < 0 || index >= lines.length) return '';
  return buildTranslatedLineMap(lines, translatedLines).get(index) ?? '';
}

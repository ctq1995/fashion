export const DESKTOP_LYRIC_WINDOW_LABEL = 'desktop-lyric';
export const DESKTOP_LYRIC_WINDOW_QUERY = 'desktop-lyric';
export const DESKTOP_LYRIC_READY_EVENT = 'desktop-lyric:ready';
export const DESKTOP_LYRIC_CLOSED_EVENT = 'desktop-lyric:closed';
export const DESKTOP_LYRIC_MOVED_EVENT = 'desktop-lyric:moved';
export const DESKTOP_LYRIC_STATE_EVENT = 'desktop-lyric:state';
export const DESKTOP_LYRIC_ACTION_EVENT = 'desktop-lyric:action';
export const DESKTOP_LYRIC_WINDOW_WIDTH = 860;
export const DESKTOP_LYRIC_WINDOW_HEIGHT = 164;

export interface DesktopLyricWindowPosition {
  x: number;
  y: number;
}

export interface DesktopLyricSettings {
  alwaysOnTop: boolean;
  locked: boolean;
  showTranslation: boolean;
  lyricOffsetMs: number;
  fontScale: number;
  backgroundOpacity: number;
  scrollSpeed: number;
  baseColor: string;
  highlightColor: string;
  windowPosition: DesktopLyricWindowPosition | null;
}

export interface DesktopLyricStatePayload {
  trackTitle: string;
  trackArtist: string;
  coverUrl: string | null;
  playbackTime: number;
  playbackUpdatedAt: number;
  currentLine: string;
  currentLineProgress: number | null;
  currentLineTime: number | null;
  currentLineDuration: number | null;
  translatedLine: string;
  nextLine: string;
  hasTrack: boolean;
  hasLyric: boolean;
  isPlaying: boolean;
  settings: DesktopLyricSettings;
}

export interface DesktopLyricActionPayload {
  type: 'toggle-always-on-top';
}

export const DEFAULT_DESKTOP_LYRIC_SETTINGS: DesktopLyricSettings = Object.freeze({
  alwaysOnTop: true,
  locked: false,
  showTranslation: true,
  lyricOffsetMs: 0,
  fontScale: 1,
  backgroundOpacity: 0.32,
  scrollSpeed: 72,
  baseColor: '#FFFFFF',
  highlightColor: '#16D6A0',
  windowPosition: null,
});

const LYRIC_OFFSET_MS_MIN = -5000;
const LYRIC_OFFSET_MS_MAX = 5000;
const FONT_SCALE_MIN = 0.84;
const FONT_SCALE_MAX = 1.48;
const BACKGROUND_OPACITY_MIN = 0.12;
const BACKGROUND_OPACITY_MAX = 0.62;
const SCROLL_SPEED_MIN = 36;
const SCROLL_SPEED_MAX = 120;
const HEX_COLOR_PATTERN = /^#(?:[0-9a-f]{3}|[0-9a-f]{6})$/i;

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function normalizeColor(value: unknown, fallback: string) {
  if (typeof value !== 'string') return fallback;

  const trimmed = value.trim();
  if (!HEX_COLOR_PATTERN.test(trimmed)) return fallback;

  if (trimmed.length === 4) {
    return `#${trimmed
      .slice(1)
      .split('')
      .map((segment) => `${segment}${segment}`)
      .join('')
      .toUpperCase()}`;
  }

  return trimmed.toUpperCase();
}

function normalizeWindowPosition(value: unknown): DesktopLyricWindowPosition | null {
  if (!value || typeof value !== 'object') return null;

  const position = value as Record<string, unknown>;
  if (typeof position.x !== 'number' || typeof position.y !== 'number') {
    return null;
  }

  if (!Number.isFinite(position.x) || !Number.isFinite(position.y)) {
    return null;
  }

  return {
    x: Math.round(position.x),
    y: Math.round(position.y),
  };
}

export function normalizeDesktopLyricSettings(
  value: Partial<DesktopLyricSettings> | null | undefined,
): DesktopLyricSettings {
  const next = value ?? {};

  return {
    alwaysOnTop:
      typeof next.alwaysOnTop === 'boolean'
        ? next.alwaysOnTop
        : DEFAULT_DESKTOP_LYRIC_SETTINGS.alwaysOnTop,
    locked:
      typeof next.locked === 'boolean'
        ? next.locked
        : DEFAULT_DESKTOP_LYRIC_SETTINGS.locked,
    showTranslation:
      typeof next.showTranslation === 'boolean'
        ? next.showTranslation
        : DEFAULT_DESKTOP_LYRIC_SETTINGS.showTranslation,
    lyricOffsetMs:
      typeof next.lyricOffsetMs === 'number' && Number.isFinite(next.lyricOffsetMs)
        ? Math.round(clamp(next.lyricOffsetMs, LYRIC_OFFSET_MS_MIN, LYRIC_OFFSET_MS_MAX))
        : DEFAULT_DESKTOP_LYRIC_SETTINGS.lyricOffsetMs,
    fontScale:
      typeof next.fontScale === 'number' && Number.isFinite(next.fontScale)
        ? clamp(next.fontScale, FONT_SCALE_MIN, FONT_SCALE_MAX)
        : DEFAULT_DESKTOP_LYRIC_SETTINGS.fontScale,
    backgroundOpacity:
      typeof next.backgroundOpacity === 'number' && Number.isFinite(next.backgroundOpacity)
        ? clamp(next.backgroundOpacity, BACKGROUND_OPACITY_MIN, BACKGROUND_OPACITY_MAX)
        : DEFAULT_DESKTOP_LYRIC_SETTINGS.backgroundOpacity,
    scrollSpeed:
      typeof next.scrollSpeed === 'number' && Number.isFinite(next.scrollSpeed)
        ? clamp(next.scrollSpeed, SCROLL_SPEED_MIN, SCROLL_SPEED_MAX)
        : DEFAULT_DESKTOP_LYRIC_SETTINGS.scrollSpeed,
    baseColor: normalizeColor(next.baseColor, DEFAULT_DESKTOP_LYRIC_SETTINGS.baseColor),
    highlightColor: normalizeColor(
      next.highlightColor,
      DEFAULT_DESKTOP_LYRIC_SETTINGS.highlightColor,
    ),
    windowPosition: normalizeWindowPosition(next.windowPosition),
  };
}

function sameWindowPosition(
  left: DesktopLyricWindowPosition | null,
  right: DesktopLyricWindowPosition | null,
) {
  if (left === right) return true;
  if (!left || !right) return false;
  return left.x === right.x && left.y === right.y;
}

export function sameDesktopLyricSettings(left: DesktopLyricSettings, right: DesktopLyricSettings) {
  return (
    left.alwaysOnTop === right.alwaysOnTop &&
    left.locked === right.locked &&
    left.showTranslation === right.showTranslation &&
    left.lyricOffsetMs === right.lyricOffsetMs &&
    left.fontScale === right.fontScale &&
    left.backgroundOpacity === right.backgroundOpacity &&
    left.scrollSpeed === right.scrollSpeed &&
    left.baseColor === right.baseColor &&
    left.highlightColor === right.highlightColor &&
    sameWindowPosition(left.windowPosition, right.windowPosition)
  );
}

export function isDesktopLyricSettings(value: unknown): value is DesktopLyricSettings {
  if (!value || typeof value !== 'object') return false;
  const normalized = normalizeDesktopLyricSettings(value as Partial<DesktopLyricSettings>);
  return sameDesktopLyricSettings(value as DesktopLyricSettings, normalized);
}

export function isDesktopLyricWindowMode(search: string) {
  return new URLSearchParams(search).has(DESKTOP_LYRIC_WINDOW_QUERY);
}

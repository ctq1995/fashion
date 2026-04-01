import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { SOURCES, type MusicSource } from '@/api/music';
import { readVersionedStorage, writeVersionedStorage } from '@/utils/persistence';

export type AppTheme = 'light' | 'dark';

const UI_STORAGE_VERSION = 1;
const THEME_STORAGE_KEY = 'fashion:theme';
const TOOLBAR_SOURCE_KEY = 'fashion:toolbar-source';
const ENABLED_SOURCES_KEY = 'fashion:enabled-sources';
const ALL_SOURCE_VALUES = SOURCES.map((item) => item.value);

function isTheme(value: unknown): value is AppTheme {
  return value === 'light' || value === 'dark';
}

function isMusicSource(value: unknown): value is MusicSource {
  return typeof value === 'string' && ALL_SOURCE_VALUES.includes(value as MusicSource);
}

function sameSources(left: MusicSource[], right: MusicSource[]) {
  return left.length === right.length && left.every((item, index) => item === right[index]);
}

function normalizeEnabledSources(value: MusicSource[]): MusicSource[] {
  const normalized = ALL_SOURCE_VALUES.filter((source) => value.includes(source));
  return normalized.length ? normalized : [...ALL_SOURCE_VALUES];
}

function readInitialTheme(): AppTheme {
  return readVersionedStorage<AppTheme>(THEME_STORAGE_KEY, UI_STORAGE_VERSION, {
    fallback: 'light',
    validate: isTheme,
    migrateLegacy: (raw) => (isTheme(raw) ? raw : null),
  });
}

function readEnabledSources(): MusicSource[] {
  return normalizeEnabledSources(
    readVersionedStorage<MusicSource[]>(ENABLED_SOURCES_KEY, UI_STORAGE_VERSION, {
      fallback: [...ALL_SOURCE_VALUES],
      validate: (value): value is MusicSource[] => Array.isArray(value) && value.every(isMusicSource),
    }),
  );
}

function readStoredToolbarSource(): MusicSource | null {
  return readVersionedStorage<MusicSource | null>(TOOLBAR_SOURCE_KEY, UI_STORAGE_VERSION, {
    fallback: null,
    validate: (value): value is MusicSource | null => value === null || isMusicSource(value),
    migrateLegacy: (raw) => (isMusicSource(raw) ? raw : null),
  });
}

function resolveToolbarSource(source: MusicSource | null, enabledSources: MusicSource[]): MusicSource {
  if (source && enabledSources.includes(source)) return source;
  return enabledSources[0] ?? 'netease';
}

export const useUiStore = defineStore('ui', () => {
  const theme = ref<AppTheme>(readInitialTheme());
  const toolbarSearch = ref('');
  const toolbarSearchNonce = ref(0);
  const enabledToolbarSources = ref<MusicSource[]>(readEnabledSources());
  const toolbarSource = ref<MusicSource>(
    resolveToolbarSource(readStoredToolbarSource(), enabledToolbarSources.value),
  );

  watch(
    theme,
    (value) => {
      writeVersionedStorage(THEME_STORAGE_KEY, UI_STORAGE_VERSION, value);
    },
    { immediate: true },
  );

  watch(
    enabledToolbarSources,
    (value) => {
      const normalized = normalizeEnabledSources(value);
      if (!sameSources(normalized, value)) {
        enabledToolbarSources.value = normalized;
        return;
      }

      if (!normalized.includes(toolbarSource.value)) {
        toolbarSource.value = resolveToolbarSource(toolbarSource.value, normalized);
      }

      writeVersionedStorage(ENABLED_SOURCES_KEY, UI_STORAGE_VERSION, normalized);
    },
    { deep: true, immediate: true },
  );

  watch(
    toolbarSource,
    (value) => {
      const resolved = resolveToolbarSource(value, enabledToolbarSources.value);
      if (resolved !== value) {
        toolbarSource.value = resolved;
        return;
      }

      writeVersionedStorage(TOOLBAR_SOURCE_KEY, UI_STORAGE_VERSION, value);
    },
    { immediate: true },
  );

  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light';
  }

  function setTheme(value: AppTheme) {
    theme.value = value;
  }

  function isSourceEnabled(value: MusicSource) {
    return enabledToolbarSources.value.includes(value);
  }

  function setToolbarSource(value: MusicSource) {
    if (!isSourceEnabled(value)) return;
    toolbarSource.value = value;
  }

  function setSourceEnabled(value: MusicSource, enabled: boolean) {
    const next = new Set(enabledToolbarSources.value);
    if (enabled) next.add(value);
    else next.delete(value);
    enabledToolbarSources.value = normalizeEnabledSources([...next] as MusicSource[]);
  }

  function toggleSourceEnabled(value: MusicSource) {
    if (isSourceEnabled(value) && enabledToolbarSources.value.length === 1) {
      return false;
    }

    setSourceEnabled(value, !isSourceEnabled(value));
    return true;
  }

  function submitToolbarSearch(value: string, source?: MusicSource) {
    toolbarSearch.value = value.trim();
    if (source && isSourceEnabled(source)) {
      toolbarSource.value = source;
    }
    toolbarSearchNonce.value++;
  }

  return {
    theme,
    toolbarSearch,
    toolbarSearchNonce,
    toolbarSource,
    enabledToolbarSources,
    setTheme,
    toggleTheme,
    isSourceEnabled,
    setSourceEnabled,
    toggleSourceEnabled,
    setToolbarSource,
    submitToolbarSearch,
  };
});

<template>
  <Transition name="fade" mode="out-in">
    <SearchPanel
      v-if="activePanel === 'search'"
      key="search"
      :mode="searchMode"
      @open-library="$emit('open-library')"
      @open-history="$emit('open-history')"
    />
    <FavoritesPanel v-else-if="activePanel === 'favorites'" key="favorites" />
    <LocalLibraryPanel v-else-if="activePanel === 'local-library'" key="local-library" />
    <HistoryPanel v-else-if="activePanel === 'history'" key="history" />
    <LyricPanel
      v-else-if="activePanel === 'lyric'"
      key="lyric"
      :fullscreen="lyricFullscreen"
      @toggle-fullscreen="$emit('toggle-fullscreen')"
    />
    <SettingsPanel v-else-if="activePanel === 'settings'" key="settings" />
  </Transition>
</template>

<script setup lang="ts">
import FavoritesPanel from '@/components/FavoritesPanel.vue';
import HistoryPanel from '@/components/HistoryPanel.vue';
import LocalLibraryPanel from '@/components/LocalLibraryPanel.vue';
import LyricPanel from '@/components/LyricPanel.vue';
import SearchPanel from '@/components/SearchPanel.vue';
import SettingsPanel from '@/components/SettingsPanel.vue';

defineProps<{
  activePanel: 'search' | 'favorites' | 'local-library' | 'history' | 'lyric' | 'settings';
  searchMode: 'recommend' | 'discover';
  lyricFullscreen: boolean;
}>();

defineEmits<{
  'open-library': [];
  'open-history': [];
  'toggle-fullscreen': [];
}>();
</script>

<template>
  <nav class="settings-section-nav">
    <button
      v-for="item in items"
      :key="item.id"
      type="button"
      class="app-chip-btn"
      :class="{ active: activeId === item.id }"
      @click="$emit('jump', item.id)"
    >
      {{ item.label }}
    </button>
  </nav>
</template>

<script setup lang="ts">
defineProps<{
  items: ReadonlyArray<{ id: string; label: string }>;
  activeId: string;
}>();

defineEmits<{
  jump: [id: string];
}>();
</script>

<style scoped>
.settings-section-nav {
  position: sticky;
  top: 0;
  z-index: 5;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-bottom: 12px;
  background: linear-gradient(180deg, var(--bg-panel) 70%, transparent);
}
</style>

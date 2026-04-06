<template>
  <div class="shared-song-row" :class="{ 'is-active': active }">
    <slot name="index" />

    <div class="row-cover">
      <slot name="cover" />
    </div>

    <div class="row-meta">
      <div class="row-head">
        <span class="row-title">{{ title }}</span>
        <span v-if="playingLabel" class="playing-tag">{{ playingLabel }}</span>
      </div>
      <span class="row-sub">{{ subtitle }}</span>
      <div v-if="$slots.extra" class="row-extra">
        <slot name="extra" />
      </div>
    </div>

    <span class="row-duration">{{ durationText }}</span>

    <div class="row-actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'SharedSongRow' });

defineProps<{
  title: string;
  subtitle: string;
  durationText: string;
  playingLabel?: string;
  active?: boolean;
}>();
</script>

<style scoped>
.shared-song-row {
  min-height: 58px;
  padding: 8px 10px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.02);
  display: grid;
  grid-template-columns: auto 42px minmax(0, 1fr) auto auto;
  gap: 10px;
  align-items: center;
  transition: var(--transition);
}

.shared-song-row.is-active {
  background: var(--bg-hover);
  border-color: var(--border);
}

.row-cover {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  overflow: hidden;
  background: var(--bg-hover);
  border: 1px solid var(--border);
}

.row-meta {
  min-width: 0;
}

.row-head {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.row-title,
.row-sub,
.row-extra {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-title {
  min-width: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
}

.row-sub {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-muted);
}

.row-extra {
  margin-top: 3px;
  font-size: 11px;
  color: var(--text-muted);
}

.playing-tag {
  flex-shrink: 0;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 10px;
  font-weight: 700;
}

.row-duration {
  min-width: 48px;
  text-align: right;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-secondary);
}

.row-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

@media (max-width: 760px) {
  .shared-song-row {
    grid-template-columns: auto 42px minmax(0, 1fr) auto;
    grid-template-areas:
      'index cover meta actions'
      'index cover extra duration';
    align-items: start;
  }

  .row-meta {
    grid-column: 3;
    grid-row: 1;
  }

  .row-duration {
    grid-column: 4;
    grid-row: 2;
    align-self: center;
  }

  .row-actions {
    grid-column: 4;
    grid-row: 1;
    justify-self: end;
  }

  .row-extra {
    white-space: normal;
  }
}
</style>

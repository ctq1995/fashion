import { describe, expect, it } from 'vitest';
import favoritesSource from '@/components/FavoritesPanel.vue?raw';
import historySource from '@/components/HistoryPanel.vue?raw';
import localLibrarySource from '@/components/LocalLibraryPanel.vue?raw';
import playlistSource from '@/components/PlaylistPanel.vue?raw';
import searchSource from '@/components/SearchPanel.vue?raw';
import sharedSource from '@/components/SharedSongRow.vue?raw';
import appHtmlSource from '../../index.html?raw';

describe('shared song row layout', () => {
  it('defines a shared song row component with duration and action areas', () => {
    expect(sharedSource).toContain("defineOptions({ name: 'SharedSongRow' })");
    expect(sharedSource).toContain('class="row-duration"');
    expect(sharedSource).toContain('class="row-actions"');
    expect(sharedSource).toContain('name="extra"');
  });

  it('uses the shared song row in favorites', () => {
    expect(favoritesSource).toContain("from '@/components/SharedSongRow.vue'");
    expect(favoritesSource).toContain('<SharedSongRow');
  });

  it('uses the shared song row in search', () => {
    expect(searchSource).toContain("from '@/components/SharedSongRow.vue'");
    expect(searchSource).toContain('<SharedSongRow');
  });

  it('uses the shared song row in history', () => {
    expect(historySource).toContain("from '@/components/SharedSongRow.vue'");
    expect(historySource).toContain('<SharedSongRow');
  });

  it('uses the shared song row in playlist', () => {
    expect(playlistSource).toContain("from '@/components/SharedSongRow.vue'");
    expect(playlistSource).toContain('<SharedSongRow');
  });

  it('uses the shared song row in local library', () => {
    expect(localLibrarySource).toContain("from '@/components/SharedSongRow.vue'");
    expect(localLibrarySource).toContain('<SharedSongRow');
  });

  it('renders favorites duration through shared song row props', () => {
    expect(favoritesSource).toContain(':duration-text="formatDuration(track.durationSec ?? null)"');
    expect(favoritesSource).toContain(`:playing-label="isCurrentTrack(track) ? '播放中' : undefined"`);
  });

  it('keeps playlist index slot aligned with the shared row baseline', () => {
    expect(playlistSource).toContain('<template #index>');
    expect(playlistSource).toContain('class="row-index"');
  });

  it('keeps local library aligned with the shared row slots and formatter', () => {
    expect(localLibrarySource).toContain('<template #index>');
    expect(localLibrarySource).toContain('<template #cover>');
    expect(localLibrarySource).not.toContain('<template #extra>');
    expect(localLibrarySource).toContain('<template #actions>');
    expect(localLibrarySource).toContain(`:duration-text="formatDuration(item.record.durationSec)"`);
    expect(localLibrarySource).not.toContain('function formatDuration(');
    expect(localLibrarySource).not.toContain('source-tag');
    expect(localLibrarySource).not.toContain("grid-column: 2 / span 2;");
  });

  it('keeps playlist action buttons visually aligned with favorites button sizing', () => {
    expect(playlistSource).toContain(':deep(.drawer-row .row-actions .app-icon-btn)');
    expect(playlistSource).toContain('width: 42px;');
    expect(playlistSource).toContain('height: 42px;');
    expect(playlistSource).toContain('border-radius: 14px;');
  });

  it('declares a favicon in the app html shell', () => {
    expect(appHtmlSource).toContain('rel="icon"');
  });

  it('aligns search header and empty state with the favorites visual shell', () => {
    expect(searchSource).toContain('class="result-header"');
    expect(searchSource).toContain('border-radius: 18px;');
    expect(searchSource).toContain('background: linear-gradient(135deg, var(--panel-strong), rgba(255, 255, 255, 0.03));');
    expect(searchSource).toContain('border: 1px dashed var(--border);');
  });

  it('aligns local library empty state with the favorites visual shell', () => {
    expect(localLibrarySource).toContain('border: 1px dashed var(--border);');
    expect(localLibrarySource).toContain('background: var(--bg-hover);');
    expect(localLibrarySource).toContain('border-radius: 20px;');
  });

  it('aligns history empty state with the favorites visual shell', () => {
    expect(historySource).toContain('border: 1px dashed var(--border);');
    expect(historySource).toContain('background: var(--bg-hover);');
    expect(historySource).toContain('border-radius: 20px;');
  });

  it('uses the shared duration formatting across all aligned panels', () => {
    expect(favoritesSource).toContain("from '@/utils/formatters'");
    expect(historySource).toContain("from '@/utils/formatters'");
    expect(localLibrarySource).toContain("from '@/utils/formatters'");
    expect(playlistSource).toContain("from '@/utils/formatters'");
    expect(searchSource).toContain("from '@/utils/formatters'");
    expect(favoritesSource).not.toContain('function formatDuration(');
    expect(historySource).not.toContain('function formatDuration(');
    expect(localLibrarySource).not.toContain('function formatDuration(');
    expect(playlistSource).not.toContain('function formatDuration(');
    expect(searchSource).not.toContain('function formatDuration(');
  });

  it('does not render played-at time in history rows', () => {
    expect(historySource).not.toContain('<template #extra>');
    expect(historySource).not.toContain('class="history-extra"');
    expect(historySource).not.toContain('fmtPlayedAt(');
  });
});


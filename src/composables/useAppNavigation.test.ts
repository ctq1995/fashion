import { describe, expect, it } from 'vitest';
import { ref } from 'vue';
import { useAppNavigation } from '@/composables/useAppNavigation';

describe('useAppNavigation', () => {
  it('maps recommend and discover nav targets to search panel and closes playlist drawer', () => {
    const playlistDrawerOpen = ref(true);
    const navigation = useAppNavigation({
      playlistDrawerOpen,
      submitToolbarSearch: () => undefined,
    });

    navigation.navigateTo('discover');

    expect(navigation.activePanel.value).toBe('search');
    expect(navigation.activeNav.value).toBe('discover');
    expect(navigation.searchMode.value).toBe('discover');
    expect(playlistDrawerOpen.value).toBe(false);
  });

  it('tracks panel history for back and forward navigation', () => {
    const navigation = useAppNavigation({
      playlistDrawerOpen: ref(false),
      submitToolbarSearch: () => undefined,
    });

    navigation.navigateTo('favorites');
    navigation.navigateTo('history');

    expect(navigation.canGoBack.value).toBe(true);
    expect(navigation.canGoForward.value).toBe(false);

    navigation.goBack();
    expect(navigation.activePanel.value).toBe('favorites');
    expect(navigation.canGoForward.value).toBe(true);

    navigation.goForward();
    expect(navigation.activePanel.value).toBe('history');
  });

  it('returns from lyric panel to previous panel when history exists', () => {
    const navigation = useAppNavigation({
      playlistDrawerOpen: ref(false),
      submitToolbarSearch: () => undefined,
    });

    navigation.navigateTo('favorites');
    navigation.navigateTo('lyric');

    navigation.toggleLyricPanel();

    expect(navigation.activePanel.value).toBe('favorites');
  });

  it('falls back to recommend search when closing lyric panel without history', () => {
    const navigation = useAppNavigation({
      playlistDrawerOpen: ref(false),
      submitToolbarSearch: () => undefined,
    });

    navigation.toggleLyricPanel();
    expect(navigation.activePanel.value).toBe('lyric');

    navigation.toggleLyricPanel();

    expect(navigation.activePanel.value).toBe('search');
    expect(navigation.activeNav.value).toBe('recommend');
    expect(navigation.searchMode.value).toBe('recommend');
  });

  it('opens search from toolbar submission without mutating nav mode', () => {
    const submitted: string[] = [];
    const navigation = useAppNavigation({
      playlistDrawerOpen: ref(false),
      submitToolbarSearch: (value) => submitted.push(value),
    });

    navigation.navigateTo('favorites');
    navigation.handleToolbarSearch('test');

    expect(submitted).toEqual(['test']);
    expect(navigation.activePanel.value).toBe('search');
    expect(navigation.activeNav.value).toBe('favorites');
  });

  it('resets mobile search entry back to recommend mode and closes drawer', () => {
    const playlistDrawerOpen = ref(true);
    const navigation = useAppNavigation({
      playlistDrawerOpen,
      submitToolbarSearch: () => undefined,
    });

    navigation.navigateTo('discover');
    playlistDrawerOpen.value = true;

    navigation.handleMobileSearch();

    expect(navigation.activePanel.value).toBe('search');
    expect(navigation.activeNav.value).toBe('recommend');
    expect(navigation.searchMode.value).toBe('recommend');
    expect(playlistDrawerOpen.value).toBe(false);
  });
});

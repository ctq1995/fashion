import { computed, ref, type Ref } from 'vue';

export type Panel = 'search' | 'favorites' | 'local-library' | 'history' | 'lyric' | 'settings';
export type NavKey = 'recommend' | 'discover' | 'favorites' | 'local-library' | 'history' | 'settings';
export type SearchMode = 'recommend' | 'discover';

type UseAppNavigationOptions = {
  playlistDrawerOpen: Ref<boolean>;
  submitToolbarSearch: (value: string) => void;
};

export function useAppNavigation({
  playlistDrawerOpen,
  submitToolbarSearch,
}: UseAppNavigationOptions) {
  const activePanel = ref<Panel>('search');
  const activeNav = ref<NavKey>('recommend');
  const searchMode = ref<SearchMode>('recommend');
  const panelHistory = ref<Panel[]>(['search']);
  const historyIndex = ref(0);

  const canGoBack = computed(() => historyIndex.value > 0);
  const canGoForward = computed(() => historyIndex.value < panelHistory.value.length - 1);

  function setPanel(panel: Panel, recordHistory = true) {
    if (activePanel.value === panel && recordHistory) return;

    activePanel.value = panel;

    if (!recordHistory) return;

    panelHistory.value = panelHistory.value.slice(0, historyIndex.value + 1);
    panelHistory.value.push(panel);
    historyIndex.value = panelHistory.value.length - 1;
  }

  function navigateTo(target: string) {
    if (target === 'recommend' || target === 'discover') {
      activeNav.value = target;
      searchMode.value = target;
      setPanel('search');
      playlistDrawerOpen.value = false;
      return;
    }

    const panel = target as Panel;
    setPanel(panel);
    if (panel === 'favorites' || panel === 'local-library' || panel === 'history' || panel === 'settings') {
      activeNav.value = panel;
    }
    playlistDrawerOpen.value = false;
  }

  function goBack() {
    if (!canGoBack.value) return;
    historyIndex.value -= 1;
    setPanel(panelHistory.value[historyIndex.value], false);
  }

  function goForward() {
    if (!canGoForward.value) return;
    historyIndex.value += 1;
    setPanel(panelHistory.value[historyIndex.value], false);
  }

  function handleToolbarSearch(value: string) {
    submitToolbarSearch(value);
    setPanel('search');
  }

  function handleMobileSearch() {
    activeNav.value = 'recommend';
    searchMode.value = 'recommend';
    setPanel('search');
    playlistDrawerOpen.value = false;
  }

  function toggleLyricPanel() {
    if (activePanel.value === 'lyric') {
      if (canGoBack.value) {
        goBack();
        return;
      }
      navigateTo('recommend');
      return;
    }
    navigateTo('lyric');
  }

  return {
    activePanel,
    activeNav,
    searchMode,
    panelHistory,
    historyIndex,
    canGoBack,
    canGoForward,
    setPanel,
    navigateTo,
    goBack,
    goForward,
    handleToolbarSearch,
    handleMobileSearch,
    toggleLyricPanel,
  };
}

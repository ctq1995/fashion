import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import DesktopLyricApp from './DesktopLyricApp.vue';
import MiniPlayerApp from './MiniPlayerApp.vue';
import './styles/global.css';
import { initializePersistence } from '@/utils/persistence';
import { isDesktopLyricWindowMode } from '@/utils/desktopLyric';
import { isMiniPlayerWindowMode } from '@/utils/miniPlayer';
import { initializeRuntimeInfo } from '@/utils/runtime';

async function bootstrap() {
  await initializeRuntimeInfo();
  await initializePersistence();

  const RootComponent =
    typeof window !== 'undefined' && isDesktopLyricWindowMode(window.location.search)
      ? DesktopLyricApp
      : typeof window !== 'undefined' && isMiniPlayerWindowMode(window.location.search)
        ? MiniPlayerApp
        : App;

  const app = createApp(RootComponent);
  app.use(createPinia());
  app.mount('#app');
}

void bootstrap();

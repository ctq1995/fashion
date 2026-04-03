import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import DesktopLyricApp from './DesktopLyricApp.vue';
import './styles/global.css';
import { initializePersistence } from '@/utils/persistence';
import { isDesktopLyricWindowMode } from '@/utils/desktopLyric';

async function bootstrap() {
  await initializePersistence();

  const RootComponent =
    typeof window !== 'undefined' && isDesktopLyricWindowMode(window.location.search)
      ? DesktopLyricApp
      : App;

  const app = createApp(RootComponent);
  app.use(createPinia());
  app.mount('#app');
}

void bootstrap();

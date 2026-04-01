import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './styles/global.css';
import { initializePersistence } from '@/utils/persistence';

async function bootstrap() {
  await initializePersistence();

  const app = createApp(App);
  app.use(createPinia());
  app.mount('#app');
}

void bootstrap();

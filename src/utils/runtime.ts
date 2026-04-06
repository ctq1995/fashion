import { invoke } from '@tauri-apps/api/core';
import { reactive, readonly } from 'vue';

interface BackendRuntimeInfo {
  os: string;
  isMobile: boolean;
}

interface RuntimeState {
  initialized: boolean;
  isTauri: boolean;
  os: string;
  isMobile: boolean;
  isDesktop: boolean;
  supportsWindowControls: boolean;
  supportsDesktopLyricWindow: boolean;
  supportsDirectoryManagement: boolean;
  supportsSystemTray: boolean;
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

function detectMobileUserAgent() {
  if (typeof navigator === 'undefined') return false;
  return /android|iphone|ipad|ipod|mobile/i.test(navigator.userAgent);
}

function applyRuntimeInfo(next: { isTauri: boolean; os: string; isMobile: boolean }) {
  runtime.initialized = true;
  runtime.isTauri = next.isTauri;
  runtime.os = next.os;
  runtime.isMobile = next.isMobile;
  runtime.isDesktop = !next.isMobile;
  runtime.supportsWindowControls = next.isTauri && !next.isMobile;
  runtime.supportsDesktopLyricWindow = next.isTauri && !next.isMobile;
  runtime.supportsDirectoryManagement = next.isTauri && !next.isMobile;
  runtime.supportsSystemTray = next.isTauri && !next.isMobile;
}

const runtime = reactive<RuntimeState>({
  initialized: false,
  isTauri: isTauriRuntime(),
  os: 'unknown',
  isMobile: detectMobileUserAgent(),
  isDesktop: !detectMobileUserAgent(),
  supportsWindowControls: false,
  supportsDesktopLyricWindow: false,
  supportsDirectoryManagement: false,
  supportsSystemTray: false,
});

applyRuntimeInfo({
  isTauri: runtime.isTauri,
  os: runtime.isMobile ? 'mobile-web' : 'web',
  isMobile: runtime.isMobile,
});

export async function initializeRuntimeInfo() {
  if (!isTauriRuntime()) {
    applyRuntimeInfo({
      isTauri: false,
      os: runtime.isMobile ? 'mobile-web' : 'web',
      isMobile: detectMobileUserAgent(),
    });
    return runtime;
  }

  try {
    const info = await invoke<BackendRuntimeInfo>('get_runtime_info');
    applyRuntimeInfo({
      isTauri: true,
      os: info.os,
      isMobile: info.isMobile,
    });
  } catch (error) {
    console.error('initializeRuntimeInfo failed', error);
    applyRuntimeInfo({
      isTauri: true,
      os: detectMobileUserAgent() ? 'android-webview' : 'desktop-webview',
      isMobile: detectMobileUserAgent(),
    });
  }

  return runtime;
}

export function useRuntimeInfo() {
  return readonly(runtime);
}

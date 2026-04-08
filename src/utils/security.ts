const ALLOWED_REMOTE_PROTOCOLS = ['http:', 'https:'] as const;

export function sanitizeRemoteMediaUrl(url?: string) {
  if (!url) return '';

  try {
    const parsed = new URL(url);
    return ALLOWED_REMOTE_PROTOCOLS.includes(parsed.protocol as (typeof ALLOWED_REMOTE_PROTOCOLS)[number])
      ? parsed.toString()
      : '';
  } catch {
    return '';
  }
}

export function isProductionLoggingEnabled() {
  return typeof window !== 'undefined' && 'location' in window && /localhost|127\.0\.0\.1/.test(window.location.host);
}

# Changelog

## v1.3.0-beta.2 - 2026-04-04

- Added `gequbao` as a new music source, including search, lyric/cover parsing, playback URL resolution, and download support.
- Fixed `gequbao` playback by handling Kuwo direct links, local cached audio playback, and source-specific CORS behavior.
- Added persistent audio caching under the user data directory so replay and downloads can reuse local files before requesting remote servers.
- Added cache management in Settings to clear cached audio files and local search, cover, and lyric cache entries.
- Updated the main lyric panel so overlong lines advance with progress-driven scrolling instead of duplicated marquee text floating alongside the highlight.

## v1.2 - 2026-04-03

- Added desktop lyric window support with independent settings and persistence.
- Updated desktop lyric behavior to use progress-driven lyric highlighting instead of marquee-style looping.
- Added desktop lyric color customization, including preset colors, a color palette, custom HEX input, and default reset.
- Removed track title and artist metadata from the desktop lyric display.
- Improved lyric rendering, settings, and related tests/build stability for this release.

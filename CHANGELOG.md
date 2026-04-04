# Changelog

## v1.4.0 - 2026-04-05

- Fixed desktop app shutdown so the desktop lyric child window is destroyed instead of only being hidden, reducing lingering background processes after closing the app.
- Added Tauri-side exit cleanup to destroy remaining child windows when the main window closes or the application exits.
- Refined the mobile APK layout so the header, bottom player, and tab navigation reuse the desktop app's panel, accent, and spacing language.
- Stabilized lyric highlighting by removing whole-row scaling and font jumps, keeping progress rendering inside the lyric text instead of making the active line feel like it floats.
- Refreshed bundled app icons for desktop and mobile targets and skipped Windows MSI packaging on beta tags so GitHub beta builds can stay focused on Linux and Android artifacts.
- Added a Windows Tauri build wrapper that maps prerelease versions such as `1.3.0-beta.6` to an MSI-compatible numeric prerelease during local bundling.

## v1.3.0-beta.6 - 2026-04-04

- Refined the mobile APK layout so the header, bottom player, and tab navigation reuse the desktop app's panel, accent, and spacing language.
- Stabilized lyric highlighting by removing whole-row scaling and font jumps, keeping progress rendering inside the lyric text instead of making the active line feel like it floats.
- Refreshed bundled app icons for desktop and mobile targets and skipped Windows MSI packaging on beta tags so GitHub beta builds can stay focused on Linux and Android artifacts.
- Added a Windows Tauri build wrapper that maps prerelease versions such as `1.3.0-beta.6` to an MSI-compatible numeric prerelease during local bundling.

## v1.3.0-beta.5 - 2026-04-04

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

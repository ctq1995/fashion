# Changelog

## v1.5.1 - 2026-04-26

- Published a patch release from the stable desktop baseline after dropping in-progress regressions from the working tree.
- Synchronized app, Tauri, and Cargo package version metadata to `1.5.1` for GitHub distribution.
- Refreshed the release record so packaged builds and repository tags point to the same patch version.

## v1.5.0 - 2026-04-09

- Hardened the desktop app security baseline by replacing the permissive Tauri CSP with a minimal policy and restricting remote media URLs to supported schemes only.
- Reduced production playback log exposure and improved download feedback with clearer saved-file results in the search list.
- Improved search recovery UX with limited-source hints, retry actions, and quicker source switching from empty/error states.
- Reduced search row action density by moving lower-frequency actions into a more menu.
- Added grouped settings section navigation to improve reachability of audio source, playback, window, and lyric options.

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

export const MINI_PLAYER_WINDOW_LABEL = 'mini-player';
export const MINI_PLAYER_WINDOW_QUERY = 'mini-player=1';
export const MINI_PLAYER_READY_EVENT = 'mini-player:ready';
export const MINI_PLAYER_CLOSED_EVENT = 'mini-player:closed';
export const MINI_PLAYER_HIDE_EVENT = 'mini-player:hide';
export const MINI_PLAYER_TOGGLE_PLAY_EVENT = 'mini-player:toggle-play';
export const MINI_PLAYER_PLAY_PREV_EVENT = 'mini-player:play-prev';
export const MINI_PLAYER_PLAY_NEXT_EVENT = 'mini-player:play-next';
export const MINI_PLAYER_TOGGLE_MODE_EVENT = 'mini-player:toggle-mode';
export const MINI_PLAYER_TOGGLE_DESKTOP_LYRIC_EVENT = 'mini-player:toggle-desktop-lyric';
export const MINI_PLAYER_SEEK_EVENT = 'mini-player:seek';
export const MINI_PLAYER_STATE_EVENT = 'mini-player:state';

export function isMiniPlayerWindowMode(search: string) {
  return new URLSearchParams(search).get('mini-player') === '1';
}

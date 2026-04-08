import { describe, expect, it } from 'vitest';
import miniPlayerSource from '@/MiniPlayerApp.vue?raw';
import tauriLibSource from '../../src-tauri/src/lib.rs?raw';
import commandsSource from '../../src-tauri/src/commands.rs?raw';

describe('mini player edge dock wiring', () => {
  it('binds the drag handler so dock check can run after dragging', () => {
    expect(miniPlayerSource).toMatch(/@(mouse|pointer)down(?:\.capture)?="startDragging"/);
  });

  it('uses the dedicated rust drag command from the mini player frontend', () => {
    expect(miniPlayerSource).toContain("invoke('mini_player_start_dragging')");
    expect(miniPlayerSource).not.toContain("invoke('mini_player_check_dock_after_drag')");
  });

  it('registers the dedicated rust drag command', () => {
    expect(tauriLibSource).toContain('commands::mini_player_start_dragging');
  });

  it('installs moved-event based mini player dock tracking in rust', () => {
    expect(tauriLibSource).toContain('commands::install_mini_player_dock_tracking');
  });

  it('routes post-drag docking through moved tracking instead of chaining a direct rust check', () => {
    expect(miniPlayerSource).toContain("invoke('mini_player_start_dragging')");
    expect(miniPlayerSource).not.toContain("invoke('mini_player_check_dock_after_drag')");
  });

  it('keeps the mini player window fixed-size in rust startup wiring', () => {
    expect(tauriLibSource).toContain('.inner_size(420.0, 164.0)');
    expect(tauriLibSource).toContain('.min_inner_size(420.0, 164.0)');
    expect(tauriLibSource).toContain('.max_inner_size(420.0, 164.0)');
    expect(tauriLibSource).toContain('.resizable(false)');
  });

  it('adds a dedicated reveal buffer session in rust dock state handling', () => {
    expect(commandsSource).toContain('const MINI_PLAYER_REVEAL_DELAY_MS: u64 = 120;');
    expect(commandsSource).toContain('reveal_session: u64');
    expect(commandsSource).toContain('fn schedule_mini_player_reveal');
  });

  it('uses dock edge specific reveal animation hooks in the mini player UI', () => {
    expect(miniPlayerSource).toContain('data-reveal-from');
    expect(miniPlayerSource).toContain('revealFrom.value = edge;');
    expect(miniPlayerSource).toContain('window.setTimeout(() => {');
  });
});

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::sync::{Arc, Mutex};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Listener, Manager, RunEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
#[cfg(any(target_os = "android", target_os = "ios"))]
use tauri::{Manager, RunEvent, WebviewWindow};
mod app_storage;
mod commands;
#[cfg(windows)]
mod window_chrome;

const MINI_PLAYER_WINDOW_LABEL: &str = "mini-player";
const CLOSE_BEHAVIOR_EVENT: &str = "app:close-behavior";

fn destroy_child_windows<R: tauri::Runtime>(main_window: &WebviewWindow<R>) {
    let app = main_window.app_handle();
    for (label, window) in app.webview_windows() {
        if label != main_window.label() {
            let _ = window.destroy();
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn ensure_mini_player_window(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(MINI_PLAYER_WINDOW_LABEL) {
        return Ok(window);
    }

    let window = WebviewWindowBuilder::new(
        app,
        MINI_PLAYER_WINDOW_LABEL,
        WebviewUrl::App("index.html?mini-player=1".into()),
    )
    .title("Fashion Mini Player")
    .inner_size(420.0, 164.0)
    .min_inner_size(380.0, 156.0)
    .max_inner_size(560.0, 220.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .build()
    .map_err(|error| error.to_string())?;

    if let Some(icon) = app.default_window_icon().cloned() {
        let _ = window.set_icon(icon);
    }

    Ok(window)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn build_system_tray(app: &tauri::AppHandle) -> Result<(), String> {
    let show_item = MenuItem::with_id(app, "tray-show", "显示主窗口", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let play_pause_item = MenuItem::with_id(app, "tray-toggle-play", "播放/暂停", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let prev_item = MenuItem::with_id(app, "tray-prev", "上一首", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let next_item = MenuItem::with_id(app, "tray-next", "下一首", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let quit_item = MenuItem::with_id(app, "tray-quit", "退出应用", true, None::<&str>)
        .map_err(|error| error.to_string())?;

    let menu = Menu::with_items(app, &[&show_item, &play_pause_item, &prev_item, &next_item, &quit_item])
        .map_err(|error| error.to_string())?;

    let tray_icon = app.default_window_icon().cloned();

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "tray-show" => {
                    let _ = app.emit("tray:show-main", ());
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "tray-toggle-play" => {
                    let _ = app.emit("tray:toggle-play", ());
                }
                "tray-prev" => {
                    let _ = app.emit("tray:play-prev", ());
                }
                "tray-next" => {
                    let _ = app.emit("tray:play-next", ());
                }
                "tray-quit" => {
                    let _ = app.emit("tray:exit", ());
                    app.exit(0)
                }
                _ => {}
            }
        });

    if let Some(icon) = tray_icon {
        tray_builder = tray_builder.icon(icon);
    }

    tray_builder.build(app).map_err(|error| error.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            commands::search_music,
            commands::get_music_url,
            commands::get_pic_url,
            commands::get_lyric,
            commands::get_playlist_detail,
            commands::get_user_playlists,
            commands::get_aux_lyric,
            commands::get_recommend_playlist,
            commands::search_once,
            commands::download_music,
            commands::get_runtime_info,
            commands::window_get_state,
            commands::window_toggle_lyric_fullscreen,
            app_storage::load_persistence_bootstrap,
            app_storage::save_persistence_snapshot,
            app_storage::write_persistence_entry,
            app_storage::remove_persistence_entries,
            app_storage::set_data_directory,
            app_storage::set_download_directory,
            app_storage::pick_folder,
            commands::clear_cached_audio_files,
            commands::scan_local_library,
            commands::window_start_dragging,
            commands::window_minimize,
            commands::window_maximize,
            commands::window_close,
            commands::window_hide,
            commands::window_show,
            commands::window_set_always_on_top,
            commands::emit_app_event,
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon().cloned() {
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                    let _ = window.set_icon(icon);

                    #[cfg(any(target_os = "android", target_os = "ios"))]
                    let _ = icon;
                }

                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                {
                    build_system_tray(app.handle())?;
                    let _ = ensure_mini_player_window(app.handle())?;
                }

                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                let close_behavior = Arc::new(Mutex::new(String::from("tray")));

                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                {
                    let close_behavior_state = Arc::clone(&close_behavior);
                    app.listen(CLOSE_BEHAVIOR_EVENT, move |event| {
                        let next = event.payload().trim_matches('"');
                        if next != "tray" && next != "exit" {
                            return;
                        }
                        if let Ok(mut guard) = close_behavior_state.lock() {
                            *guard = next.to_string();
                        }
                    });
                }

                let tracked_window = window.clone();
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                let tracked_close_behavior = Arc::clone(&close_behavior);
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        #[cfg(not(any(target_os = "android", target_os = "ios")))]
                        {
                            let should_hide_to_tray = tracked_close_behavior
                                .lock()
                                .map(|guard| guard.as_str() == "tray")
                                .unwrap_or(true);

                            if should_hide_to_tray {
                                api.prevent_close();
                                let _ = tracked_window.hide();
                                return;
                            }
                        }

                        destroy_child_windows(&tracked_window);
                    }
                });

                #[cfg(windows)]
                window_chrome::install(&window);

                #[cfg(debug_assertions)]
                if std::env::var_os("FASHION_OPEN_DEVTOOLS").is_some() {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |app_handle, event| {
            if matches!(event, RunEvent::Exit | RunEvent::ExitRequested { .. }) {
                if let Some(main_window) = app_handle.get_webview_window("main") {
                    destroy_child_windows(&main_window);
                }
            }
        });
}

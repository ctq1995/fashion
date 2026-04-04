use tauri::{Manager, RunEvent, WebviewWindow};
mod app_storage;
mod commands;
#[cfg(windows)]
mod window_chrome;

fn destroy_child_windows<R: tauri::Runtime>(main_window: &WebviewWindow<R>) {
    let app = main_window.app_handle();
    for (label, window) in app.webview_windows() {
        if label != main_window.label() {
            let _ = window.destroy();
        }
    }
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
            commands::window_start_dragging,
            commands::window_minimize,
            commands::window_maximize,
            commands::window_close,
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon().cloned() {
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                    let _ = window.set_icon(icon);

                    #[cfg(any(target_os = "android", target_os = "ios"))]
                    let _ = icon;
                }

                let tracked_window = window.clone();
                window.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
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

use tauri::Manager;
mod app_storage;
mod commands;
#[cfg(windows)]
mod window_chrome;

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
                    let _ = window.set_icon(icon);
                }

                #[cfg(windows)]
                window_chrome::install(&window);

                #[cfg(debug_assertions)]
                if std::env::var_os("FASHION_OPEN_DEVTOOLS").is_some() {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

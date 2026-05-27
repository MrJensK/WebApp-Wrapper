use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    webview::DownloadEvent,
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};

// ─── Konfigurerbar state ────────────────────────────────────────────────────

struct AppState {
    #[allow(dead_code)]
    target_url: String,
    download_dir: PathBuf,
}

// ─── Tauri-kommandon ────────────────────────────────────────────────────────

/// Returnerar aktuell nedladdningsmapp
#[tauri::command]
fn get_download_dir(state: tauri::State<Mutex<AppState>>) -> String {
    state
        .lock()
        .unwrap()
        .download_dir
        .to_string_lossy()
        .to_string()
}

/// Sätter ny nedladdningsmapp (anropas från frontend/tray)
#[tauri::command]
fn set_download_dir(
    new_dir: String,
    state: tauri::State<Mutex<AppState>>,
) -> Result<String, String> {
    let path = PathBuf::from(&new_dir);
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    state.lock().unwrap().download_dir = path;
    Ok(new_dir)
}

// ─── Entry point ────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ── Standardvärden (ändra här eller via GUI) ──────────────────────────
    let default_url = "https://example.com".to_string();
    let default_download_dir = dirs_next::download_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mini-web-sdk");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Mutex::new(AppState {
            target_url: default_url.clone(),
            download_dir: default_download_dir.clone(),
        }))
        .invoke_handler(tauri::generate_handler![get_download_dir, set_download_dir])
        .setup(move |app| {
            // Skapa nedladdningsmappen om den inte finns
            let _ = std::fs::create_dir_all(&default_download_dir);

            // ── Bygg huvud-webview som pekar mot extern URL ──────────────
            let url = WebviewUrl::External(default_url.parse().expect("Ogiltig URL"));
            let download_dir = default_download_dir.clone();

            WebviewWindowBuilder::new(app, "main", url)
                .title("Mini Web SDK")
                .inner_size(1280.0, 800.0)
                .resizable(true)
                // ── Fånga upp alla nedladdningar ────────────────────────
                .on_download(move |_webview, event| {
                    match event {
                        DownloadEvent::Requested { url, destination } => {
                            let filename = destination
                                .file_name()
                                .map(|n| n.to_owned())
                                .unwrap_or_else(|| {
                                    // Fallback: ta filnamn ur URL:en
                                    let seg = url.path_segments()
                                        .and_then(|s| s.last())
                                        .unwrap_or("download");
                                    std::ffi::OsString::from(seg)
                                });

                            // Peka om till låst mapp
                            *destination = download_dir.join(filename);

                            eprintln!(
                                "[download] {} → {}",
                                url,
                                destination.display()
                            );
                        }
                        DownloadEvent::Finished { url, path, success } => {
                            if success {
                                if let Some(p) = path {
                                    eprintln!("[download] klar: {}", p.display());
                                }
                            } else {
                                eprintln!("[download] misslyckades: {url}");
                            }
                        }
                        _ => {}
                    }
                    true
                })
                .build()?;

            // ── System-tray med snabbmeny ────────────────────────────────
            build_tray(app.handle())?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Fel vid start av Tauri-appen");
}

// ─── Tray-hjälp ─────────────────────────────────────────────────────────────

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_i = MenuItem::with_id(app, "open", "Öppna", true, None::<&str>)?;
    let folder_i = MenuItem::with_id(app, "folder", "Visa nedladdningsmapp", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Avsluta", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_i, &folder_i, &quit_i])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "folder" => {
                let dir = app
                    .state::<Mutex<AppState>>()
                    .lock()
                    .unwrap()
                    .download_dir
                    .clone();
                let _ = open::that(dir);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

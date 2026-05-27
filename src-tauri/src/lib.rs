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
    target_url: String,
    download_dir: PathBuf,
}

// ─── Nätverkskontroll ───────────────────────────────────────────────────────

/// Kontrollerar om en URL är nåbar via en snabb TCP-anslutning (3 s timeout).
/// Returnerar false om DNS-uppslag eller anslutning misslyckas — t.ex. om VPN saknas.
fn check_reachable(url: &str) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let parsed = match url.parse::<url::Url>() {
        Ok(u) => u,
        Err(_) => return false,
    };

    let host = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };

    let port = parsed.port_or_known_default().unwrap_or(443);
    let addr_str = format!("{}:{}", host, port);

    match addr_str.to_socket_addrs() {
        Ok(mut addrs) => addrs
            .next()
            .map(|addr| TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok())
            .unwrap_or(false),
        Err(_) => false,
    }
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

/// Kontrollerar om sidan är nåbar och navigerar dit om den är det.
/// Anropas från offline-sidan när användaren klickar "Försök igen".
#[tauri::command]
fn retry_connection(
    window: tauri::WebviewWindow,
    state: tauri::State<Mutex<AppState>>,
) -> bool {
    let url = state.lock().unwrap().target_url.clone();
    if check_reachable(&url) {
        let _ = window.eval(&format!("window.location.href = '{}'", url));
        true
    } else {
        false
    }
}

// ─── Entry point ────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ── Standardvärden ────────────────────────────────────────────────────
    let default_url = "https://sdkwebbapp.vgregion.se/".to_string();

    let default_download_dir: PathBuf = {
        #[cfg(target_os = "windows")]
        { PathBuf::from(r"T:\SDK-nedladdningar\") }

        #[cfg(target_os = "macos")]
        { PathBuf::from("/Users/Shared/SDK-nedladdningar") }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        { dirs_next::download_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("mini-web-sdk") }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Mutex::new(AppState {
            target_url: default_url.clone(),
            download_dir: default_download_dir.clone(),
        }))
        .invoke_handler(tauri::generate_handler![
            get_download_dir,
            set_download_dir,
            retry_connection
        ])
        .setup(move |app| {
            let _ = std::fs::create_dir_all(&default_download_dir);

            // ── Välj startURL baserat på nätverksåtkomst ─────────────────
            let start_url = if check_reachable(&default_url) {
                WebviewUrl::External(default_url.parse().expect("Ogiltig URL"))
            } else {
                eprintln!("[nätverk] Sidan ej nåbar – visar offline-sida");
                WebviewUrl::App("offline.html".into())
            };

            let download_dir = default_download_dir.clone();

            WebviewWindowBuilder::new(app, "main", start_url)
                .title("SDK - Säker Digital Kommunikation")
                .inner_size(1280.0, 800.0)
                .resizable(true)
                .on_download(move |_webview, event| {
                    match event {
                        DownloadEvent::Requested { url, destination } => {
                            let filename = destination
                                .file_name()
                                .map(|n| n.to_owned())
                                .unwrap_or_else(|| {
                                    let seg = url.path_segments()
                                        .and_then(|s| s.last())
                                        .unwrap_or("download");
                                    std::ffi::OsString::from(seg)
                                });
                            *destination = download_dir.join(filename);
                            eprintln!("[download] {} → {}", url, destination.display());
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

    let icon = app.default_window_icon().cloned();
    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("SDK - Säker Digital Kommunikation");
    if let Some(i) = icon {
        tray = tray.icon(i);
    }
    tray
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
        .build(app)
        .map(|_| ())
}

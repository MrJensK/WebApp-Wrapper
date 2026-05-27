# TODO – Kommande funktioner

## Hög prioritet

### macOS: Låt användaren välja nedladdningsmapp
**Status:** Ej påbörjad  
**Berörda filer:** `src-tauri/src/lib.rs`, tray-meny

På macOS är nedladdningsmappen hårdkodad till `/Users/Shared/SDK-nedladdningar`.
Framtida lösning: låt användaren välja mapp via en mappväljardialog (`tauri-plugin-dialog`)
och spara valet persistent (t.ex. i appens konfigurationsfil via `tauri-plugin-store`).

```rust
// Rough sketch – lägg till i tray-meny under "Välj nedladdningsmapp":
use tauri_plugin_dialog::DialogExt;
app.dialog().file().pick_folder(|folder| {
    if let Some(path) = folder {
        // uppdatera AppState::download_dir
    }
});
```

---

### macOS: Kontrollera och montera nätverksdisk
**Status:** Ej påbörjad  
**Berörda filer:** `src-tauri/src/lib.rs`

Appen ska vid start kontrollera om en specifik nätverksdisk är monterad (t.ex. `T:\` eller
en SMB-share). Om den inte är monterad ska appen försöka montera den automatiskt.

**Att ta reda på:**
- Vilken nätverksadress (SMB/CIFS) ska monteras?
- Kräver monteringen autentisering (Kerberos/domänkonto)?
- Vad ska hända om monteringen misslyckas – varning till användaren?

**macOS-kommando för montering:**
```bash
mount_smbfs //användare@server/share /Volumes/Nätverksdisk
```

**Rust-implementation (grov skiss):**
```rust
fn ensure_network_drive_mounted() -> bool {
    let mount_point = "/Volumes/SDK-nätverksdisk";
    if std::path::Path::new(mount_point).exists() {
        return true; // redan monterad
    }
    // Försök montera
    std::process::Command::new("mount_smbfs")
        .args(["//server/share", mount_point])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
```

---

## Medel prioritet

### Hantera VPN-frånkoppling under pågående session
**Status:** Ej påbörjad  
**Berörda filer:** `src-tauri/src/lib.rs`

Nuvarande implementation kontrollerar nätverksåtkomst **en gång vid start**.
Om användaren tappar VPN-anslutningen medan appen körs visas webbläsarens
standardfelsida istället för vår anpassade offline-sida.

Lösning: använd en bakgrundstråd som periodiskt kontrollerar åtkomst och
navigerar till `offline.html` om sidan inte längre är nåbar:

```rust
// Kontrollera var 30:e sekund i bakgrunden
std::thread::spawn(move || {
    loop {
        std::thread::sleep(Duration::from_secs(30));
        if !check_reachable(&url) {
            let _ = window.eval("window.location.href = '/offline.html'");
        }
    }
});
```

---

### Windows: Kodsignering
**Status:** Väntar på EV-certifikat  
Se README → Kodsignering för instruktioner.

---

### macOS: Kodsignering och notarisering
**Status:** Väntar på Apple Developer-konto  
Se README → Kodsignering för instruktioner.

---

## Låg prioritet

### Inställningsfönster i appen
Möjlighet att ändra nedladdningsmapp och målURL via ett inbyggt inställningsgränssnitt
istället för att behöva redigera `lib.rs` och bygga om.

### Versionsvisning i tray-menyn
Visa appens version i tray-menyn, t.ex. `SDK v1.0.1`.

### Automatisk uppdatering
Tauri v2 har inbyggt stöd för auto-uppdatering via `tauri-plugin-updater`.
Kräver en signerad release och en uppdaterings-endpoint.

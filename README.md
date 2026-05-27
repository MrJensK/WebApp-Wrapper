# SDK Desktop App – Säker Digital Kommunikation (Mac & Windows)

En minimal skrivbordsapplikation byggd med [Tauri v2](https://tauri.app/) som laddar en extern webbadress i en inbyggd webbvy och låser nedladdningar till en förutbestämd mapp per operativsystem.

---

## Innehåll

- [Vad appen gör](#vad-appen-gör)
- [Projektstruktur](#projektstruktur)
- [Förutsättningar](#förutsättningar)
- [Kom igång](#kom-igång)
- [Konfiguration](#konfiguration)
- [Bygga för distribution](#bygga-för-distribution)
- [GitHub Actions CI/CD](#github-actions-cicd)
- [Kodsignering](#kodsignering)
- [Tauri-kommandon (API)](#tauri-kommandon-api)
- [Felsökning](#felsökning)

---

## Vad appen gör

- Öppnar ett fönster med en inbyggd webbläsare som pekar mot `https://sdkwebbapp.vgregion.se/`
- **Kräver VPN eller företagsnätverket** – om sidan inte är nåbar visas en anpassad offline-sida med automatisk återförsök var 15:e sekund
- Fångar upp **alla nedladdningsklick** och sparar filerna i en låst mapp – användaren kan inte välja en annan destination
- Skapar nedladdningsmappen automatiskt om den inte finns
- Har en **system-tray-ikon** med snabbmeny för att öppna appen, visa nedladdningsmappen och avsluta

---

## Projektstruktur

```
.
├── .github/
│   └── workflows/
│       └── build.yml          # CI/CD – bygger för macOS och Windows
├── dist/
│   ├── index.html             # Tomt HTML-skal (krävs av Tauri, innehåller ingen logik)
│   └── offline.html           # Visas när VPN/nätverket saknas
├── src-tauri/
│   ├── capabilities/
│   │   └── default.json       # Tauri v2 säkerhetspermissions
│   ├── icons/                 # App-ikoner i alla storlekar (genereras med npx tauri icon)
│   ├── src/
│   │   ├── main.rs            # Entrypoint – anropar bara lib::run()
│   │   └── lib.rs             # All applogik: webview, nedladdning, tray
│   ├── build.rs               # Tauri build-skript
│   ├── Cargo.toml             # Rust-beroenden
│   └── tauri.conf.json        # App-namn, identifierare, ikoner, bundle-inställningar
├── package.json               # npm-skript och @tauri-apps/cli
└── README.md
```

### Nyckelfilerna

| Fil | Vad du ändrar där |
|---|---|
| `src-tauri/src/lib.rs` | Mål-URL, nedladdningsmappar per OS, tray-meny |
| `src-tauri/tauri.conf.json` | App-namn, bundle-ID, ikonlista |
| `.github/workflows/build.yml` | CI-konfiguration, signeringssecrets |

---

## Förutsättningar

### macOS

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (via Homebrew eller nodejs.org)
brew install node

# Xcode Command Line Tools
xcode-select --install
```

### Windows

```powershell
# Rust
winget install Rustlang.Rustup

# Node.js
winget install OpenJS.NodeJS

# Visual Studio Build Tools (MSVC-toolchain)
winget install Microsoft.VisualStudio.2022.BuildTools
# Välj "Desktop development with C++" i installationsverktyget

# WebView2 – finns förinstallerat på Windows 11, annars:
# https://developer.microsoft.com/microsoft-edge/webview2/
```

---

## Kom igång

```bash
# 1. Klona repot
git clone https://github.com/MrJensK/WebApp-Wrapper.git
cd WebApp-Wrapper

# 2. Installera npm-beroenden (@tauri-apps/cli)
npm install

# 3. Starta i utvecklingsläge med hot-reload
npm run dev
```

> **OBS:** I dev-läge på macOS visar dockan Tauris standardikon och app-namnet i menyraden
> kan skilja sig från det konfigurerade. Det är normalt – rätt namn och ikon syns i byggd app.

---

## Konfiguration

All konfiguration sker i **`src-tauri/src/lib.rs`**.

### Ändra mål-URL

```rust
// rad ~50
let default_url = "https://sdkwebbapp.vgregion.se/".to_string();
```

### Ändra nedladdningsmapp per OS

Mapparna är kompileringstidskonstanter – rätt sökväg bränns in i binären per plattform:

```rust
// rad ~51–60
let default_download_dir: PathBuf = {
    #[cfg(target_os = "windows")]
    { PathBuf::from(r"T:\SDK-nedladdningar\") }

    #[cfg(target_os = "macos")]
    { PathBuf::from("/Users/Shared/SDK-nedladdningar") }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    { dirs_next::download_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("mini-web-sdk") }
};
```

Mappen skapas automatiskt med `create_dir_all` vid appstart om den inte finns.

### Ändra app-namn

Två ställen behöver matcha:

1. **Fönsterrubrik** – `src-tauri/src/lib.rs` rad ~80:
   ```rust
   .title("SDK - Säker Digital Kommunikation")
   ```

2. **OS-namn** (dock, taskbar, installationsfil) – `src-tauri/tauri.conf.json`:
   ```json
   "productName": "SDK - Säker Digital Kommunikation"
   ```

### Byta ikon

Förbered en PNG-fil på minst **1024×1024 px** och kör:

```bash
npx tauri icon din-ikon.png
```

Kommandot skriver över allt i `src-tauri/icons/` med rätt storlekar för alla plattformar.
Inget annat behöver ändras – `tauri.conf.json` pekar redan mot `icons/`-mappen.

---

## Bygga för distribution

```bash
# Bygg för den plattform du kör på
npm run build
```

Installerarna skapas i:

| Plattform | Sökväg |
|---|---|
| macOS (DMG) | `src-tauri/target/release/bundle/dmg/*.dmg` |
| Windows (NSIS exe) | `src-tauri/target/release/bundle/nsis/*.exe` |
| Windows (MSI) | `src-tauri/target/release/bundle/msi/*.msi` |

> Du kan **inte** cross-kompilera – en Windows-binär måste byggas på Windows och en
> macOS-binär på macOS. Använd GitHub Actions (se nedan) för att bygga för båda plattformarna.

---

## GitHub Actions CI/CD

Workflow-filen `.github/workflows/build.yml` bygger automatiskt för:

- `x86_64-pc-windows-msvc` (Windows 64-bit)
- `aarch64-apple-darwin` (Apple Silicon)
- `x86_64-apple-darwin` (Intel Mac)

### Triggas av

```bash
# Ny release-tag
git tag v1.2.0
git push --tags

# Eller manuellt via GitHub: Actions → Build → Run workflow
```

### Hämta färdiga filer

Efter en lyckad körning finns installerarna under **Actions → ditt bygge → Artifacts**:

- `windows-installer` – `.exe` och `.msi`
- `macos-aarch64-apple-darwin` – `.dmg` för Apple Silicon
- `macos-x86_64-apple-darwin` – `.dmg` för Intel Mac

---

## Kodsignering

### macOS

Utan signering visas en Gatekeeper-dialog första gången appen öppnas.
Workflow:en använder **ad-hoc-signering** (`-`) som standard, vilket innebär att
användaren behöver högerklicka → Öppna vid första körning – ingen `xattr`-fix i terminalen krävs.

För att ta bort dialogen helt krävs ett **Apple Developer Program**-konto (~1 300 kr/år)
med notarisering. Lägg då till följande secrets i GitHub-repot
(Settings → Secrets and variables → Actions):

| Secret | Beskrivning |
|---|---|
| `APPLE_CERTIFICATE` | `.p12`-certifikat base64-kodat: `base64 -i cert.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Lösenordet för `.p12`-filen |
| `APPLE_SIGNING_IDENTITY` | T.ex. `Developer ID Application: Namn (TEAMID)` |
| `APPLE_ID` | Din Apple ID-epost |
| `APPLE_PASSWORD` | App-specifikt lösenord från [appleid.apple.com](https://appleid.apple.com) |
| `APPLE_TEAM_ID` | 10-siffrigt team-ID från [developer.apple.com](https://developer.apple.com) |

Workflow:en aktiverar automatiskt riktig signering när secrets finns – inget annat behöver ändras.

### Windows

Utan signering visas en SmartScreen-varning ("Windows skyddade din dator").
Användaren kan klicka **Mer information → Kör ändå**. Varningen försvinner automatiskt
efter att tillräckligt många användare kört filen (ryktesbaserat).

För att ta bort varningen direkt krävs ett **Authenticode EV-certifikat**
från t.ex. DigiCert eller Sectigo (~2 000–4 000 kr/år).

---

## Tauri-kommandon (API)

Dessa kommandon kan anropas från webbvyn via `@tauri-apps/api` om det behövs i framtiden:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Hämta aktuell nedladdningsmapp
const dir = await invoke<string>('get_download_dir');

// Sätt ny nedladdningsmapp (skapar mappen om den inte finns)
await invoke<string>('set_download_dir', { newDir: 'C:\\NyMapp' });
```

---

## Felsökning

### `npm run dev` – "tauri: command not found"
Kör `npm install` först så att `@tauri-apps/cli` installeras lokalt.

### Appen visar blank sida istället för webbplatsen
Kontrollera att `default_url` i `lib.rs` är en giltig och nåbar URL.
Kontrollera även att `dist/index.html` finns – det är ett krav från Tauri även om det är tomt.

### macOS – "Appen är skadad och kan inte öppnas"
Ad-hoc-signering är aktiverad i CI men om du bygger lokalt utan signering:
```bash
xattr -d com.apple.quarantine /Applications/SDK\ -\ Säker\ Digital\ Kommunikation.app
```

### Windows – SmartScreen-varning
Klicka **Mer information → Kör ändå**. Inget tekniskt fel.

### Bygget misslyckas i CI – "Unable to find web assets"
Kontrollera att `dist/index.html` är committad till repot (inte listad i `.gitignore`).

### Nedladdningar sparas på fel ställe
Kontrollera `default_download_dir` i `lib.rs`. Sökvägen är kompileringstidskonstant –
appen måste byggas om efter ändringar.

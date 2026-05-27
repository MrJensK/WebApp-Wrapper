# SDK Desktop App – Säker Digital Kommunikation (Mac & Windows)

En minimal skrivbordsapplikation byggd med [Tauri v2](https://tauri.app/) som laddar en extern webbadress i en inbyggd webbvy och låser nedladdningar till en valbar mapp per operativsystem.

---

## Innehåll

- [Vad appen gör](#vad-appen-gör)
- [Projektstruktur](#projektstruktur)
- [Förutsättningar](#förutsättningar)
- [Kom igång](#kom-igång)
- [Konfiguration](#konfiguration)
- [Nedladdningsmapp](#nedladdningsmapp)
- [Bygga för distribution](#bygga-för-distribution)
- [GitHub Actions CI/CD](#github-actions-cicd)
- [Kodsignering](#kodsignering)
- [Tauri-kommandon (API)](#tauri-kommandon-api)
- [Felsökning](#felsökning)

---

## Vad appen gör

- Öppnar ett fönster med en inbyggd webbläsare som pekar mot `https://sdkwebbapp.vgregion.se/`
- **Kräver VPN eller företagsnätverket** – om sidan inte är nåbar visas en anpassad offline-sida med automatisk återförsök var 15:e sekund
- Fångar upp **alla nedladdningsklick** och sparar filerna i en låst mapp
- Användaren kan **välja nedladdningsmapp** via tray-menyn – valet sparas och används vid nästa start
- Skapar nedladdningsmappen automatiskt om den inte finns
- Har en **system-tray-ikon** med snabbmeny

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
│   │   └── lib.rs             # All applogik: webview, nedladdning, tray, konfiguration
│   ├── build.rs               # Tauri build-skript
│   ├── Cargo.toml             # Rust-beroenden
│   └── tauri.conf.json        # App-namn, identifierare, ikoner, bundle-inställningar
├── package.json               # npm-skript och @tauri-apps/cli
├── TODO.md                    # Kommande funktioner
└── README.md
```

### Nyckelfilerna

| Fil | Vad du ändrar där |
|---|---|
| `src-tauri/src/lib.rs` | Mål-URL, standard nedladdningsmapp per OS, tray-meny |
| `src-tauri/tauri.conf.json` | App-namn, bundle-ID, ikonlista |
| `.github/workflows/build.yml` | CI-konfiguration, signeringssecrets |
| `dist/offline.html` | Texten och utseendet på offline-sidan |

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

### Ändra mål-URL

I [src-tauri/src/lib.rs](src-tauri/src/lib.rs), rad ~95:

```rust
let default_url = "https://sdkwebbapp.vgregion.se/".to_string();
```

### Ändra app-namn

Två ställen behöver matcha:

1. **Fönsterrubrik** – `lib.rs`:
   ```rust
   .title("SDK - Säker Digital Kommunikation")
   ```
2. **OS-namn** (dock, taskbar, installationsfil) – `tauri.conf.json`:
   ```json
   "productName": "SDK - Säker Digital Kommunikation"
   ```

### Byta ikon

Förbered en PNG-fil på minst **1024×1024 px** och kör:

```bash
npx tauri icon din-ikon.png
```

Kommandot skriver över allt i `src-tauri/icons/` automatiskt.

---

## Nedladdningsmapp

### Standardvärden (kompileringstid)

Hårdkodade per plattform i `lib.rs` – används om inget annat valts av användaren:

| Plattform | Standardmapp |
|---|---|
| Windows | `T:\SDK-nedladdningar\` |
| macOS | `/Users/Shared/SDK-nedladdningar` |

### Användaren väljer mapp (runtime)

Högerklicka på tray-ikonen → **"Välj nedladdningsmapp…"** öppnar en native mappväljardialog.
Valet sparas omedelbart och gäller även nästa gång appen startas.

Sparas i:

| Plattform | Sökväg |
|---|---|
| macOS | `~/Library/Application Support/mini-web-sdk/config.json` |
| Windows | `%APPDATA%\mini-web-sdk\config.json` |

För att återställa till standardmappen: ta bort `config.json` och starta om appen.

### Prioritetsordning

```
Sparad config.json  →  Kompileringstids-standard
```

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

> Du kan **inte** cross-kompilera. Använd GitHub Actions för att bygga för båda plattformarna.

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

Filerna laddas upp direkt till GitHub Release och är nedladdningsbara under taggen.

---

## Installation för slutanvändare

### macOS – installationsskript (rekommenderat)

Kör detta i terminalen – skriptet laddar ner senaste version, installerar till `/Applications`
och tar automatiskt bort quarantine-flaggan:

```bash
curl -fsSL https://raw.githubusercontent.com/MrJensK/WebApp-Wrapper/main/install-mac.sh | bash
```

Skriptet detekterar automatiskt om datorn är Apple Silicon eller Intel.

### macOS – manuell installation

1. Ladda ner rätt `.dmg` från [Releases](https://github.com/MrJensK/WebApp-Wrapper/releases)
2. Öppna DMG:en och dra appen till `/Applications`
3. Kör i terminalen:
```bash
xattr -rd com.apple.quarantine "/Applications/SDK - Säker Digital Kommunikation.app"
```

> **Varför behövs detta?** macOS lägger automatiskt en karantänflagga på alla filer som laddas
> ned från internet. Utan ett Apple Developer-konto och notarisering (se nedan) måste flaggan
> tas bort manuellt. Installationsskriptet gör detta automatiskt.

### Windows

1. Ladda ner `.exe`-filen från [Releases](https://github.com/MrJensK/WebApp-Wrapper/releases)
2. Kör installationsfilen
3. Klicka **Mer information → Kör ändå** om SmartScreen-varning visas

---

## Kodsignering

### macOS

Utan signering visas en Gatekeeper-dialog första gången. Workflow:en använder **ad-hoc-signering**
– användaren behöver högerklicka → Öppna vid första körning.

För att ta bort dialogen helt krävs ett **Apple Developer Program**-konto (~1 300 kr/år).
Lägg till följande secrets i GitHub (Settings → Secrets and variables → Actions):

| Secret | Beskrivning |
|---|---|
| `APPLE_CERTIFICATE` | `.p12`-certifikat base64-kodat: `base64 -i cert.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Lösenordet för `.p12`-filen |
| `APPLE_SIGNING_IDENTITY` | T.ex. `Developer ID Application: Namn (TEAMID)` |
| `APPLE_ID` | Din Apple ID-epost |
| `APPLE_PASSWORD` | App-specifikt lösenord från [appleid.apple.com](https://appleid.apple.com) |
| `APPLE_TEAM_ID` | 10-siffrigt team-ID från [developer.apple.com](https://developer.apple.com) |

### Windows

Utan signering visas SmartScreen-varning. Klicka **Mer information → Kör ändå**.
Varningen försvinner automatiskt efter att tillräckligt många kört filen.

För att ta bort varningen direkt krävs ett **Authenticode EV-certifikat** (~2 000–4 000 kr/år).

---

## Tauri-kommandon (API)

Tillgängliga via `invoke` från webbvyn:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Hämta aktuell nedladdningsmapp
const dir = await invoke<string>('get_download_dir');

// Sätt ny nedladdningsmapp programmatiskt (skapar + sparar)
await invoke<string>('set_download_dir', { newDir: 'C:\\NyMapp' });

// Kontrollera nätverksåtkomst och navigera till appen om möjligt
const ok = await invoke<boolean>('retry_connection');
```

---

## Felsökning

### `npm run dev` – "tauri: command not found"
Kör `npm install` först så att `@tauri-apps/cli` installeras lokalt.

### Appen visar offline-sida fast VPN är ansluten
Kontrollera att `default_url` i `lib.rs` är korrekt. Stäng och öppna appen igen –
nätverkskontrollen sker vid start. Klicka "Försök igen" i offline-sidan för att kontrollera på nytt.

### macOS – "Appen är skadad och kan inte öppnas"
Utan Apple Developer-konto + notarisering visar macOS alltid detta för nedladdade appar.
Använd installationsskriptet (se ovan) eller kör:
```bash
xattr -rd com.apple.quarantine "/Applications/SDK - Säker Digital Kommunikation.app"
```

### Windows – SmartScreen-varning
Klicka **Mer information → Kör ändå**. Inget tekniskt fel.

### Nedladdningar sparas på fel ställe
Välj rätt mapp via tray-menyn → **"Välj nedladdningsmapp…"**.
Eller ta bort `config.json` (se sökväg ovan) för att återgå till standardmappen.

### Bygget misslyckas i CI – "Unable to find web assets"
Kontrollera att `dist/index.html` är committad till repot (inte i `.gitignore`).

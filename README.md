# Mask — Persona/Compartmentalization Manager

Cross-platform persona manager for security researchers. Isolates browser identities with per-profile proxies, fingerprint hardening, WireGuard automation, and encrypted export. Supports Firefox, Chromium, and Brave with optional Podman/Docker container isolation.

---

## Threat Model

Mask is designed to defend against **identity correlation** across sessions:

- A target (person, organization, or platform) with access to browser fingerprinting, cookies, or behavioral analytics should not be able to link your "Researcher A" activity to your "Journalist B" activity.
- An adversary monitoring egress traffic at the network level should see different source IPs for different personas.
- A misconfigured browser leaking WebRTC, DNS, or timezone data should be caught before the persona is used operationally.

### What Mask protects against

| Threat | Mitigation |
|--------|------------|
| Cookie / session bleed between identities | Separate browser profile per persona (Firefox `--profile`, Chromium `--user-data-dir`) |
| Browser fingerprint correlation (UA, canvas, timezone, locale) | Per-persona `user.js` injection (Firefox) or `Preferences` JSON (Chromium/Brave) |
| WebRTC IP leak | Disabled by default — Firefox via `user.js`, Chromium/Brave via `--webrtc-ip-handling-policy` |
| DNS leak through proxy | SOCKS5 remote DNS enforced in browser prefs |
| Network-level IP correlation | Per-persona proxy or WireGuard interface |
| Timezone/locale mismatch with exit node | Leak Check panel flags mismatches before use |
| OS-level process bleed | Optional Podman/Docker container isolation per persona |
| Audit trail absence | Local SQLite log of all launches, switches, and checks |

### What Mask does NOT protect against

- **OS-level isolation (without containers)**: All personas share the same kernel by default. For high-risk use cases, enable container mode or use [Whonix](https://www.whonix.org/) / [Qubes OS](https://www.qubes-os.org/).
- **Tor-level anonymity**: Mask routes through user-configured proxies/VPNs. For multi-hop anonymity, configure Tor Browser manually or use Tails.
- **Physical OPSEC**: Typing patterns, screen content, and document metadata are out of scope.
- **VPN/proxy trustworthiness**: Mask configures routing — it does not vet your provider.
- **Extension isolation**: Third-party extensions installed into a persona's profile may reduce fingerprint resistance or exfiltrate data.

---

## Data Storage

All data is local. Zero telemetry, no Mask servers.

| What | Where |
|------|-------|
| Persona configs, audit log | `~/.local/share/mask/mask.db` (Linux) / `~/Library/Application Support/mask/mask.db` (macOS) / `%APPDATA%\mask\mask.db` (Windows) |
| Firefox profiles | `<data-dir>/profiles/<persona-id>/` |
| Chromium/Brave profiles | `<data-dir>/chromium-profiles/<persona-id>/` |
| Generated `user.js` | Regenerated on every Firefox launch from the database |
| In-browser extension | Auto-installed to each profile on first launch |

---

## Features

### Persona Management
Create, edit, and delete personas. Each has:
- Name, description, color tag
- Browser selection (Firefox, Chromium, Brave)
- Isolation mode (profile-only, Podman, Docker)
- Proxy assignment (SOCKS5 or HTTP) or WireGuard config path
- Timezone, locale, and user-agent string
- Free-text notes for account lists, use-case reminders, etc.

### Browser Profile Isolation
Each persona launches its browser with a fully isolated profile directory:
- **Firefox**: `--profile <dir> --new-instance` with injected `user.js`
- **Chromium/Brave**: `--user-data-dir <dir>` with generated `Default/Preferences`

This gives completely separate cookies, cache, history, saved passwords, and extensions per persona.

### Container Isolation
Set a persona's isolation mode to **Podman** or **Docker** to launch the browser inside a container:
- `--network=host` ensures proxy/VPN routing applies
- X11 socket and profile directory are volume-mounted into the container
- Proxy env vars (`http_proxy`, `https_proxy`, etc.) injected automatically
- "Launch Container" / "Stop Container" controls on the persona card

Requires `podman` or `docker` in PATH and a browser image (e.g. `docker.io/jlesage/firefox`).

### WireGuard Automation
If a persona has a `.conf` path or interface name set, a WireGuard toggle appears on the card:
- **Up**: calls `wg-quick up <config>` via `pkexec` (polkit GUI dialog) or `sudo` as fallback
- **Down**: calls `wg-quick down <config>`
- Status is reflected live on the card

### Network Routing
Per-persona SOCKS5 or HTTP proxy is configured directly in the browser profile. The proxy is enforced at the browser level — other apps on your system are unaffected.

### Fingerprint Hardening
The generated `user.js` (Firefox) and `Preferences` (Chromium/Brave) set:
- `privacy.resistFingerprinting = true` (Firefox)
- Custom user-agent string
- Locale/language headers matching your locale setting
- WebRTC disabled
- Battery API, geolocation, disk cache disabled

### In-Browser Persona Indicator
A lightweight browser extension is auto-installed into every profile on first launch. It shows the active persona name and color in the browser toolbar so you always know which identity is active.

- **Firefox**: installed as an unsigned extension (requires `xpinstall.signatures.required = false`, set automatically in `user.js`)
- **Chromium/Brave**: loaded via `--load-extension`

### Persona Export / Import
Export one or all personas to an encrypted file for backup or transfer:
- Encrypted with **AES-256-GCM**, key derived via **PBKDF2-SHA256** (200,000 iterations)
- Output is a single base64-encoded blob — copy to clipboard or save as `.enc` file
- Import decrypts and inserts personas, skipping any with duplicate IDs

The encryption password is never stored anywhere.

### Mask Check (Leak Detection)
Before using a persona operationally, run the Mask Check from the persona card:
1. **Egress IP** — what IP the world sees (via ipinfo.io through the proxy)
2. **DNS routing** — confirms DNS resolves through the proxy, not your ISP
3. **Timezone consistency** — flags if your configured timezone differs from the IP's geolocation timezone
4. **WebRTC** — advisory note to verify at [browserleaks.com](https://browserleaks.com) (cannot be checked server-side)

### Audit Log
Every persona launch, WireGuard toggle, container start/stop, and leak check is recorded in the local SQLite database with timestamps.

---

## Setup

### Prerequisites

**Linux (Arch/Manjaro/CachyOS)**
```bash
sudo pacman -S webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg
```

**Linux (Ubuntu/Debian)**
```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

**macOS**: Install Xcode Command Line Tools.

**Windows**: Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

**All platforms**: Install [Rust](https://rustup.rs/) and Node.js 18+.

### Build & Run

```bash
npm install
npm run tauri dev       # development (hot-reload)
npm run tauri build     # production binary
```

The built binary is at `src-tauri/target/release/mask` (Linux/macOS) or `src-tauri/target/release/mask.exe` (Windows).

### Linux: Wayland / HiDPI

If the window renders blank on a Wayland compositor, force XWayland and disable GPU buffer allocation:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 GDK_BACKEND=x11 npm run tauri dev
```

### Proxy Setup Examples

**SSH SOCKS5 tunnel**
```bash
ssh -D 1080 -N user@your-server.example.com
# Set persona proxy: socks5://127.0.0.1:1080
```

**Tor**
```bash
systemctl start tor
# Set persona proxy: socks5://127.0.0.1:9050
```

**Mullvad / any WireGuard VPN**
```bash
# Point Mask at your .conf file: /etc/wireguard/mullvad-us.conf
# Use the WireGuard toggle on the persona card to bring it up/down
```

---

## Architecture

```
src/                    React + TypeScript frontend
  api/                  Tauri invoke() wrappers
  components/           PersonaCard, PersonaForm, LeakCheckPanel, AuditLog
  pages/                PersonasPage, ExportImportPage
  types/                Shared TypeScript types

src-tauri/src/
  db.rs                 SQLite open/migrate/log helpers
  models.rs             Persona, WgStatus, ExportPayload, AuditEntry
  commands/
    persona.rs          CRUD Tauri commands
    browser.rs          Firefox + Chromium/Brave launch, user.js, extension injection
    wireguard.rs        wg_up / wg_down / wg_status via pkexec/sudo
    export_import.rs    AES-256-GCM export/import
    container.rs        Podman/Docker launch and stop
    leak_check.rs       Async leak check
    audit.rs            Audit log query
  lib.rs                Tauri app entry point, invoke_handler registration
```

---

## Security Notes

- Proxy passwords are stored **unencrypted** in SQLite. Use disk encryption (LUKS, FileVault, BitLocker) to protect the database at rest.
- The `user.js` file is **regenerated on every launch** — manual edits to a Firefox profile's `user.js` will be overwritten.
- The export encryption password is never stored. If you lose it, the export cannot be recovered.
- Running multiple personas simultaneously on the same machine increases correlation risk through timing, network behavior, and shared system resources. Use container mode to reduce this.
- WireGuard bring-up requires `pkexec` or `sudo`. The polkit dialog will prompt for your password.
 
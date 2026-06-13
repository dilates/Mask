# Mask — Persona/Compartmentalization Manager

A cross-platform desktop app for security researchers, OSINT investigators, and privacy-conscious users who need to maintain multiple isolated online identities without cross-contamination.

---

## Threat Model

Mask is designed to defend against **identity correlation** across sessions:

- A target (person, organization, or platform) with access to browser fingerprinting, cookies, or behavioral analytics should not be able to link your "Researcher A" activity to your "Journalist B" activity.
- An adversary monitoring egress traffic at the network level should see different source IPs for different personas.
- A misconfigured browser leaking WebRTC, DNS, or timezone data should be caught before the persona is used operationally.

### What Mask protects against

| Threat | Mitigation |
|--------|------------|
| Cookie / session bleed between identities | Fully separate Firefox profiles per persona |
| Browser fingerprint correlation (UA, canvas, timezone, locale) | Per-persona `user.js` injection into Firefox profile |
| WebRTC IP leak | Disables WebRTC in Firefox profile by default |
| DNS leak through proxy | SOCKS5 remote DNS enforced in Firefox prefs |
| Network-level IP correlation | Per-persona proxy/VPN assignment |
| Timezone/locale mismatch with exit node | Leak Check panel flags mismatches before use |
| Audit trail absence | Local SQLite log of all switches and checks |

### What Mask does NOT protect against

- **OS-level isolation**: All personas run on the same OS kernel. A compromised browser or extension could observe other processes. For high-risk use cases (journalism in authoritarian countries, law enforcement investigations), use [Whonix](https://www.whonix.org/) or [Qubes OS](https://www.qubes-os.org/) instead.
- **Tor-level anonymity**: Mask routes through user-configured proxies/VPNs. It does not provide Tor's multi-hop onion routing. For high-anonymity requirements, configure the Tor Browser manually or use Tails.
- **Physical OPSEC**: Side-channels like typing patterns, screen content visible to cameras, or metadata in documents are out of scope.
- **VPN/proxy trustworthiness**: Mask configures and verifies routing — it does not vet your VPN provider. You are trusting your proxy/VPN not to log.
- **Extension isolation**: If you install extensions into a persona's Firefox profile, those extensions may exfiltrate data or reduce fingerprint resistance.

---

## Data Storage

All data is local. Nothing is transmitted to Mask servers (there are none).

| What | Where |
|------|-------|
| Persona configs, audit log | `~/.local/share/mask/mask.db` (Linux) / `~/Library/Application Support/mask/mask.db` (macOS) / `%APPDATA%\mask\mask.db` (Windows) |
| Firefox profiles per persona | `<data-dir>/profiles/<persona-id>/` |
| Generated `user.js` | `<data-dir>/profiles/<persona-id>/user.js` (regenerated on each launch) |

You can back up the entire `<data-dir>` directory to preserve all personas and browser history.

---

## Features

### Persona Management
Create, edit, and delete personas. Each has:
- Name, description, color tag
- Proxy assignment (SOCKS5 or HTTP) or VPN config path
- Timezone, locale, and user-agent string (injected into Firefox via `user.js`)
- Free-text notes for account lists, use-case reminders, etc.

### Browser Profile Isolation
Each persona launches Firefox with its own `--profile` directory. This gives completely separate:
- Cookies and session storage
- Cache and history
- Saved passwords and extensions
- All Firefox preferences

### Network Routing
Per-persona SOCKS5 or HTTP proxy is configured directly in the Firefox profile via `user.js`. The proxy configuration is enforced at the browser level — other apps on your system are unaffected.

For WireGuard/VPN configs: Mask stores the path to your `.conf` file and reminds you to bring up the interface before launching. Automated VPN bring-up (`wg-quick up`) is on the roadmap.

### Fingerprint Hardening
The generated `user.js` sets:
- `privacy.resistFingerprinting = true` (canvas, WebGL, audio fingerprint randomization)
- Custom user-agent string
- `intl.accept_languages` matching your locale setting
- WebRTC disabled
- Battery API, geolocation, disk cache disabled

### Mask Check (Leak Detection)
Before using a persona operationally, run the Mask Check from the persona card. It verifies:
1. **Egress IP** — what IP the world sees (via ipinfo.io through the proxy)
2. **DNS routing** — confirms DNS resolves through the proxy, not your ISP
3. **Timezone consistency** — flags if your configured timezone differs from the IP's geolocation timezone
4. **WebRTC** — advises manual verification via browserleaks.com (cannot be checked server-side)

### Audit Log
Every persona switch, browser launch, and leak check is recorded in the local SQLite database with timestamps. Never transmitted anywhere.

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
# Install JS dependencies
npm install

# Development (hot-reload)
npm run tauri dev

# Production build
npm run tauri build
```

The built binary is in `src-tauri/target/release/` (Linux/macOS) or `src-tauri/target/release/mask.exe` (Windows).

### Proxy Setup Examples

**Mullvad SOCKS5 proxy** (without installing Mullvad app):
- Connect Mullvad app, then in Mask set proxy to `socks5://127.0.0.1:1080`

**SSH SOCKS5 tunnel**:
```bash
ssh -D 1080 -N user@your-server.example.com
```
Then set persona proxy to `socks5://127.0.0.1:1080`.

**Tor**:
```bash
# Install tor, start it (default SOCKS5 on port 9050)
systemctl start tor
```
Set persona proxy to `socks5://127.0.0.1:9050`.

---

## Architecture

```
src/                    React + TypeScript frontend
  api/                  Tauri invoke() wrappers
  components/           UI components
  pages/                Page-level components
  types/                Shared TypeScript types

src-tauri/
  src/
    db.rs               SQLite open/migrate/log helpers
    models.rs           Shared Rust types (Persona, AuditEntry, etc.)
    commands/
      persona.rs        CRUD Tauri commands
      browser.rs        Firefox launch + user.js generation
      leak_check.rs     Async leak check commands
      audit.rs          Audit log query commands
    lib.rs              Tauri app entry point
```

---

## Limitations & Roadmap

**Current limitations:**
- Firefox only (Chromium profile isolation via `--user-data-dir` is planned)
- VPN bring-up (`wg-quick up/down`) is not automated — must be done manually
- WebRTC leak check is advisory only (requires in-browser verification)
- No import/export of persona configs

**Planned:**
- Automated WireGuard interface management
- Chromium/Brave support
- Persona export/import (encrypted JSON)
- Browser extension for in-browser persona indicator
- Container-based isolation mode (Podman/Docker per persona)

---

## Security Notes

- Proxy passwords are stored **unencrypted** in SQLite. Use a disk-encryption solution (LUKS, FileVault, BitLocker) to protect the database at rest.
- The `user.js` file is **regenerated on every launch** from the database — any manual edits to the Firefox profile's `user.js` will be overwritten.
- Running multiple personas simultaneously on the same machine is possible but increases correlation risk through timing, network behavior, and shared system resources.

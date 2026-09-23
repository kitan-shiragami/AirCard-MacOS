# AirCard 🎴

[![Build Releases](https://github.com/kitan-shiragami/AirCard-MacOS/actions/workflows/build.yml/badge.svg)](https://github.com/kitan-shiragami/AirCard-MacOS/actions/workflows/build.yml)

> **Apple Wallet Card Skinner & Lockscreen Passcode Themer for iOS 18+ (No Jailbreak Required)**  
> Native desktop client for Windows, macOS, and Linux, written in Rust. Powered by the `airlift` AirTraffic sync exploit.

---

## Features
- 🎨 **Custom Card Skins:** Assign custom artwork, textures, or bank logos to Apple Pay and Apple Cash cards.
- 🔢 **Lock Screen Passcode Themes (.passthm):** Apply custom keypad button artwork from popular Cowabunga & Nugget `.passthm` themes directly to iOS lockscreen.
- ⚡ **100% Native & Lightweight:** Native `aircard.exe` on Windows and `aircard` on macOS/Linux. No Python, no Flet, no webview, no bloated runtimes.
- 🪟 **Material Design 3 Interface:** Clean, modern dark theme built with `egui` and `eframe`.
- 📱 **Zero-Hassle Card Detection:** Tap any card in your iPhone's Wallet app while connected to detect its hash in real-time via `syslog_relay`.
- 📶 **USB & WiFi Transport:** Scan card events and apply Wallet or passcode assets through USB or a paired local WiFi connection.
- 🔄 **Safe & Reversible:** Complete Books state snapshot and automatic restore engine — preserves original device state.
- 🚀 **Zero Jailbreak:** Utilizes Apple's built-in AirTraffic sync conduit without modifying system partitions or disabling security.

---

## Platform support and requirements

| Platform | Desktop previews and theme tools | iPhone scanning and applying changes | Runtime requirement |
| --- | --- | --- | --- |
| Windows 10/11 x64 | Yes | Native Apple backend over USB or paired WiFi | 64-bit iTunes / Apple Mobile Device Support |
| macOS Intel / Apple Silicon | Yes | Native Apple backend over USB or paired WiFi | Built-in CoreFoundation, MobileDevice, and AirTrafficHost frameworks |
| Linux x64 (X11 / Wayland) | Yes | Unavailable | Graphical desktop; an XDG desktop portal for file dialogs |

**Linux support currently covers the desktop application, image preparation, and theme previews.**
Apple's proprietary AirTrafficHost framework is not available on Linux; device scanning and applying
skins/themes are disabled there with an explanation in the app. Installing `usbmuxd` or
`libimobiledevice` alone does not supply the missing AirTraffic backend.

For device operations on Windows/macOS, connect your iPhone with a Lightning or USB-C cable
for the initial pairing, unlock it, and trust the computer. On macOS, also accept the device
in Finder. No iTunes installation is needed on macOS. For WiFi mode, enable WiFi sync and keep
the computer and iPhone on the same local network. Apple private framework compatibility and
actual device operations depend on the installed macOS/iOS versions.

---

## ⚠️ Windows Troubleshooting & Driver Repair (If Nothing Works)

> [!TIP]
> **iPhone not detected, AirTraffic sync hangs, or operation fails?**  
> Corrupted or conflicting Apple USB drivers on Windows are the #1 root cause.
> 1. Download and install **[3uTools](https://www.3u.com/)**.
> 2. **Disconnect your iPhone** from your PC.
> 3. In 3uTools, go to **Toolbox ➔ Repair Driver**.
> 4. Click **Repair Now** and wait for the Apple driver reinstallation to finish.
> 5. Reconnect your unlocked iPhone, tap **Trust**, and launch **AirCard**.

---

## Installation

### Pre-built binaries

Builds produced by the release workflow have these names:

- Windows x64: `aircard.exe`
- macOS Intel: `aircard-macos-x64.dmg`
- macOS Apple Silicon: `aircard-macos-arm64.dmg`
- Linux x64: `aircard-linux-x64.tar.gz` (desktop features only)

Check [Releases](https://github.com/kitan-shiragami/AirCard-MacOS/releases) for available builds.
Older releases may contain only the Windows executable or macOS tar archives.
On macOS, open the DMG for your processor, drag **AirCard.app** to **Applications**,
then eject the disk image and open AirCard from Applications.
On Linux, extract the archive and run `./aircard`; on Windows, run `aircard.exe`.

The macOS app currently uses an ad-hoc signature. It is **not Developer ID signed or
notarized by Apple**, so downloaded builds may be blocked by Gatekeeper. Public distribution
with a verified developer identity requires Developer ID signing and Apple notarization.

---

## WiFi Connection Setup
1. Connect the iPhone by USB for the initial pairing.
2. In Finder on macOS, or Apple Devices/iTunes on Windows, enable **Show this iPhone when on Wi-Fi** / **Sync with this iPhone over Wi-Fi**.
3. Apply the setting, then keep the iPhone and PC on the same local network.
4. In AirCard, click **Refresh** and confirm the device shows a **WiFi** transport.
5. Disconnect the cable, click **Refresh** again, and select **WiFi only**. Use **Auto (USB preferred)** when automatic fallback is desired.

If both transports are available, **Auto** uses USB first and falls back to WiFi. For a guaranteed end-to-end WiFi route, disconnect the USB cable, click **Refresh**, and then choose **WiFi only**. This is required because Apple's AirTraffic API selects its route by UDID rather than accepting a transport parameter.

---

## How to Customize Apple Wallet Cards
1. Connect your iPhone through USB or paired WiFi and ensure it is unlocked.
2. In AirCard, stay on the **Wallet** tab and click **Scan**.
3. On your iPhone:
   - Open **Apple Wallet** (or double-click the Side/Power button).
   - Tap the card you want to customize.
   - AirCard intercepts and saves the card hash automatically. Click **Stop**.
4. Click **Choose Image...** to pick your artwork (PNG, JPG, or WebP).
5. In **Wallet Preview**, drag the image to reposition it and adjust **Zoom** to resize it. Use **Position X / Y** for precise offsets. **Fit Image** shows the entire image with black margins; **Center** recenters it, and **Reset / Fill** restores the default centered crop.
6. Click **Apply Card Skin**. The current framing is used for both device artwork and **Export PNG**, at `1536 × 969`.
7. Force-close the **Wallet** app on your iPhone from the App Switcher (swipe up from bottom, then swipe Wallet away) and reopen Wallet to see your new card!

---

## How to Apply Lockscreen Passcode Themes (.passthm)
1. Switch to the **Passcode** tab in AirCard.
2. Click **Choose .passthm...** and select any `.passthm` package (Cowabunga or Nugget).
3. Select your target iOS version cache:
   - **Auto (TelephonyUI-10)** — iOS 18+ (Default)
   - **TelephonyUI-9** — iOS 16 - 17
   - **TelephonyUI-8** — Legacy iOS
4. Click **Apply Passcode Theme**.
5. Lock your iPhone screen or open Phone dialer to see your new custom passcode keypad buttons!

> [!IMPORTANT]
> **Turn OFF Bold Text:**  
> On your iPhone, go to **Settings ➔ Display & Brightness** and make sure **Bold Text** is turned **OFF**. If Bold Text is enabled, iOS ignores cached dialer button graphics and renders system vector fonts instead.

---

## Building from Source

Install the current stable [Rust toolchain](https://rustup.rs/) for your host platform:

- **Windows:** MSVC toolchain and Visual Studio C++ Build Tools.
- **macOS:** Xcode Command Line Tools (`xcode-select --install`). Rust builds for the
  installed toolchain architecture; use `aarch64-apple-darwin` on Apple Silicon or
  `x86_64-apple-darwin` on Intel.
- **Linux (Ubuntu/Debian):** install the native build dependencies below. File dialogs
  use the desktop's XDG portal (for example, `xdg-desktop-portal-gtk` on GTK desktops).

```sh
sudo apt-get install build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev libssl-dev
```

The same build commands work on all three platforms:

```sh
git clone https://github.com/kitan-shiragami/AirCard-MacOS.git
cd AirCard-MacOS
cargo test --locked
cargo build --release --locked
```

Output: `target/release/aircard.exe` on Windows, `target/release/aircard` on macOS/Linux.
CI builds and tests Windows x64, Linux x64, macOS Intel, and macOS Apple Silicon separately.

### Build a macOS DMG

Run on macOS with Python 3, Rust, and Xcode Command Line Tools installed:

```sh
# Build for the Mac's native hardware, even when the terminal runs under Rosetta
python3 scripts/package-macos.py

# Explicitly build an Apple Silicon Release and DMG
rustup target add aarch64-apple-darwin
python3 scripts/package-macos.py --target aarch64-apple-darwin

# Or package an already-built binary (also used by CI)
python3 scripts/package-macos.py --binary target/aarch64-apple-darwin/release/aircard
```

Install the Rust target for the desired architecture first with `rustup target add`.
The build uses an explicit target, so ARM64 binaries are under
`target/aarch64-apple-darwin/release/aircard`, and Intel binaries are under
`target/x86_64-apple-darwin/release/aircard`.
The script checks the executable's architecture and writes `dist/aircard-macos-x64.dmg`
or `dist/aircard-macos-arm64.dmg`. A universal binary produces `aircard-macos-universal.dmg`.
Each image includes **AirCard.app**, an **Applications** shortcut, and installation instructions.
The script validates the app's ad-hoc signature and the compressed disk image before publishing
the local artifact. It does not perform Developer ID signing or notarization.

### Runtime checks and saved data

Hardware-dependent tests are ignored by default. To check the macOS framework loader
without accessing a phone:

```sh
cargo test native_frameworks_and_plist_round_trip -- --ignored
```

Saved card data is stored in:

- Windows: `%LOCALAPPDATA%\AirCard\cards.json`
- macOS: `~/Library/Application Support/AirCard/cards.json`
- Linux: `$XDG_DATA_HOME/AirCard/cards.json`, or `~/.local/share/AirCard/cards.json`.

---

## Contributors
- **[@Lumid-Off](https://github.com/Lumid-Off)** (Windows Native Rust Port & Maintainer) — [GitHub](https://github.com/Lumid-Off) · [Twitter / X](https://x.com/LumidOff)
- **[@mak5er](https://github.com/mak5er)** (Original macOS App & Exploit Research) — [GitHub](https://github.com/mak5er) · [Twitter / X](https://x.com/mak5er)
- **[AirLift](https://github.com/0xjohnnydev/airlift)** by **[0xjohnny (@0xjohnnydev)](https://github.com/0xjohnnydev)**: Original AirTraffic/ATAirlock sandbox escape and proof of concept underlying `AirliftFFI`.

## Credits
- Core exploit based on `airlift` (AirTraffic sync escape).
- Theme format inspired by [Cowabunga](https://github.com/leminlimez/Cowabunga) and [Nugget](https://github.com/leminlimez/Nugget).

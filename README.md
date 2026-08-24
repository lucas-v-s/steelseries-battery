# steelseries-battery

A tiny Windows system tray app that shows the battery level of your SteelSeries wireless mouse and headset, without opening SteelSeries GG.

It talks directly to the USB dongle over raw HID, bypassing GG entirely.

![tray icon](https://img.shields.io/badge/platform-Windows-blue)

## Supported devices

- SteelSeries Rival 3 Wireless (mouse)
- SteelSeries Arctis Nova Pro Wireless (headset)

Other SteelSeries 2.4GHz devices aren't supported out of the box, but adding one is usually a small change — see [Adding another device](#adding-another-device).

## Install

### Option A: download

Grab `steelseries-battery-windows.zip` from the [Releases page](../../releases), unzip it, and run `steelseries-battery.exe`.

To have it start automatically at login, run this once in PowerShell (adjust the path if you moved the exe):

```powershell
New-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "SteelSeriesBattery" -Value "`"$PWD\steelseries-battery.exe`"" -PropertyType String -Force
```

### Option B: build from source

Requires [Rust](https://rustup.rs) and, on Windows, the MSVC build tools (`winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools"`).

```bash
git clone git@github.com:lucas-v-s/steelseries-battery.git
cd steelseries-battery
cargo build --release
```

The binary is at `target/release/steelseries-battery.exe`.

## Usage

The tray icon shows two bars (mouse | headset), colored green/yellow/red by level. Hover for the exact percentages. Right-click for **Refresh now** or **Quit**. It polls every 60 seconds.

## Adding another device

1. Plug the device's dongle in and run `cargo run --example probe`. It lists every SteelSeries HID interface on your machine with its product ID, interface number, and usage page — note the one for your device.
2. Find the command bytes for that interface: the mouse/headset protocol tables in [rivalcfg](https://github.com/flozz/rivalcfg/tree/master/rivalcfg/devices) (mice) and [arctis-usb-finder](https://github.com/richrace/arctis-usb-finder/blob/main/src/headphone_list.ts) (headsets) already document many models. If yours isn't listed, [aarol.dev's writeup](https://aarol.dev/posts/arctis-hid/) walks through sniffing it yourself with Wireshark + USBPcap while SteelSeries GG is running.
3. Add the product ID, interface number, write command, and response byte offsets to `src/battery.rs`, following the existing `read_mouse_battery` / `read_headset_battery` functions as a template.

## License

MIT

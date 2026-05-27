# Changelog

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [SemVer](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-05-27

### Fixed
- AirPods 4 (ANC) Bluetooth product code is `0x201B`, not `0x2026`. The
  former mapping pointed at Beats Solo Buds; the new one is cross-checked
  against The Apple Wiki and OpenPods. AirPods 4 (non-ANC) keeps no code
  for now — no public PID has surfaced. `0x2025` (Beats Solo 4) and the
  old `0x2026` (Beats Solo Buds) now resolve to `Unknown` instead of being
  mis-identified as AirPods 4 variants.

### Added
- `AirPods Pro (3rd gen)` (`0x2027`) and `AirPods Max (2nd gen)`
  (`0x202D`) — model variants plus capability matrix entries.
- `PODCTL_ADAPTER=hciN` env override for multi-adapter hosts. Without it
  the first `hci*` from `/sys/class/bluetooth` is used, same as before.
- `podctl install` and `podctl uninstall` gracefully skip the systemd
  steps on hosts without `systemctl` (Artix, Devuan, Void, embedded
  rootfs), printing a clear note rather than failing.

### Changed
- All shell-outs to `pactl`, `bluetoothctl`, `dbus-send` and `systemctl`
  now run with `LC_ALL=C`. Defends parsers against gettext-translated
  output on non-English locales (e.g. German "ja"/"nein" instead of
  "yes"/"no" for `pactl get-sink-mute`).
- `INSTALL.md`: optional BlueZ `DeviceID` hint documented for users who
  want their host to advertise as an Apple device (some buds expose more
  features once they think they are paired to a Mac).

## [0.1.0] - 2026-05-20

First public release.

### Added
- `podctl` CLI with status, battery, listening modes, conversation awareness,
  ear detection, mic selection, one-bud ANC, AutoANC strength, chime,
  rename, connect / disconnect / pair / unpair / list / auto-connect.
- `podctld` daemon with Unix-socket IPC at `$XDG_RUNTIME_DIR/podctl.sock`
  (0600). Live `Event` stream via `podctl watch`.
- Standalone fallback: audio and BlueZ verbs work without the daemon.
- Apple Accessory Protocol over L2CAP PSM 0x1001, from scratch — no
  external bluetooth crate.
- `podctl-tray` (StatusNotifierItem) with battery tooltip and quick-action
  menu.
- `podctl-popup` case-open bubble with three backends (wlr-layer-shell,
  X11 override-redirect, GNOME notification fallback).
- `podctl install` / `podctl uninstall` — XDG-compliant user install, shell
  completions (bash/zsh/fish), man pages, optional systemd-user service.
- `podctl debug` with default DSGVO redaction (MAC OUI only, custom names
  masked, `$HOME` → `~`).
- `podctl meter` software RMS / peak dBFS meter via `parec`.

### Known limitations
- Spatial audio, loud-sound reduction, per-bud press actions and tone
  on press: AAP setting IDs not yet pinned down — the daemon returns
  a clear "not implemented for this device" error.
- Find My, Personalized Spatial Audio, Hearing Test and Announce
  Notifications via Siri are Apple-only and cannot be implemented on
  Linux.

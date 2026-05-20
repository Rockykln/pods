# Changelog

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [SemVer](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

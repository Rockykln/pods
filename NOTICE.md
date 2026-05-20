# NOTICE

`podctl` is licensed under MIT OR Apache-2.0 at your option. See
[LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

## Protocol knowledge

The Apple Accessory Protocol opcodes used here were cross-referenced
against [LibrePods](https://github.com/kavishdevar/librepods) (GPL-3.0).
No GPL code is included in this crate — only the protocol facts (opcode
numbers, frame layouts), which are not copyrightable.

## Bundled assets

- `Noto Sans` font (UI text in `podctl-popup`) — © Google Inc.,
  [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).
  The full licence is reproduced at `assets/fonts/LICENSE-NotoSans.txt`.
  No modifications.

## Runtime tools

`podctl` shells out to the standard Linux toolchain at runtime:
`bluetoothctl` (BlueZ, GPL-2.0+), `dbus-send` (dbus, AFL-2.1 or GPL-2+),
`pactl` / `parec` (PulseAudio, LGPL-2.1+), `systemctl` (systemd,
LGPL-2.1+). These are invoked as separate processes; their licences do
not propagate into this binary.

## Third-party Rust crates

Generated from `Cargo.lock` at release time — run
`cargo about generate about.hbs > about.html` (or `cargo-license`) for
the full list with versions and licences.

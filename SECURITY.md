# Security policy

## Supported versions

Only the most recent release receives security fixes. Older `0.x`
versions are unsupported.

## Reporting a vulnerability

Do **not** open a public issue for security bugs — especially anything
touching the L2CAP / AAP code path, the Unix socket, or the install
flow. Email **[report@rockykln.com](mailto:report@rockykln.com)** or
file a [GitHub private security advisory](https://github.com/Rockykln/podctl/security/advisories/new).

Include in your report:

- the affected component (`podctl`, `podctld`, `podctl-tray`, `podctl-popup`)
- the version (`podctl --version`)
- a short reproducer
- the impact you observed

You will get an acknowledgement within seven days. Public disclosure
happens after a fix is in a tagged release, coordinated with the
reporter.

## Scope

In scope:

- crashes or memory-safety issues in the `unsafe` blocks (L2CAP socket,
  `wl_shm` memfd, libc FFI shims)
- privilege escalation via the daemon socket, the install script, or
  the systemd-user units
- BlueZ / D-Bus interaction that lets another local user influence the
  daemon
- malformed AAP frames from a paired device that crash the daemon

Out of scope:

- vulnerabilities in `bluetoothctl`, `pactl`, `parec`, BlueZ, PipeWire,
  PulseAudio or systemd themselves — report those upstream
- denial of service that requires already being root or already being
  the same user
- issues only reachable by modifying the source and rebuilding

---
name: Bug report
about: Something doesn't work as documented
labels: bug
---

**What happened**

A short description.

**What you expected**

What the docs / `podctl --help` said would happen.

**Reproducer**

```
podctl <verb> ...
```

**Debug report**

Paste the output of `podctl debug` (redacted by default — safe to share).
If the bug only reproduces with the unredacted variant, run
`podctl debug --no-redact` locally and only quote the relevant section.

```
<podctl debug output here>
```

**Environment**

- podctl version: `podctl --version`
- distro / kernel: (the `[system]` block from `podctl debug` covers this)
- desktop: KDE / Hyprland / GNOME / ...
- AirPods model: (Pro 2 USB-C, Pro 1, 4 ANC, ...)

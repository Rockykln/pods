<!--
  Keep the subject short and imperative. The body answers WHY — the
  diff already shows WHAT.
-->

## Summary

One or two sentences.

## Why

What problem does this solve? Link the issue if there is one.

## How tested

- [ ] `cargo test --all-targets` green
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] manually verified against a real device (model + setting)

## Notes for the reviewer

Anything non-obvious: protocol assumptions, fallbacks, edge cases.

# Handoff

Running notes for [weirdnet-themes](https://github.com/WeirdBrains/weirdnet-themes), newest first.

## 2026-11-22: release 0.2.0

- What changed: `00b5ce5` (CI green) released as 0.2.0 with `scripts/weirdnet_release.py` (commit `release: v0.2.0`, annotated tag [v0.2.0](https://github.com/WeirdBrains/weirdnet-themes/releases/tag/v0.2.0)), published to [crates.io](https://crates.io/crates/weirdnet-themes/0.2.0); docs at [docs.rs](https://docs.rs/weirdnet-themes/0.2.0). Breaking, hence the minor bump: see [CHANGELOG.md](CHANGELOG.md).
- Checks: the release script's checks from `release.toml` (fmt, clippy `--all-features -D warnings`, `cargo test --all-features`, `cargo doc -D warnings`), then the crates.io upload.
- Not tested: as in the entry below.
- Left: nothing for this release.
- Next: Bragi moves to 0.2.0 and drops its local theme workarounds.

## 2026-11-22: Bragi theme, optional roles, NO_COLOR, theme editing helpers (main, unreleased)

- What changed (`475757b`): the built-in [Bragi](https://github.com/WeirdBrains/bragi) theme;
  optional roles `surface`, `code_bg`, `diff_add_bg`, `diff_remove_bg`, `link` and `dim`,
  derived from the required roles when a theme leaves them out; `NO_COLOR` / `TERM=dumb` as
  `ColorDepth::None` plus `ColorDepth::parse` / `detect_with`; 16-colour backgrounds and body
  text left to the terminal; `terminal-background` off by default and `Mode::System` probing
  only through `resolve_variant_with` and `detect::Probe`; `UserDir` and editor/switcher helpers
  (`entries`, `duplicate`, `rename`, `delete`, `import`, `export`, `cycle`, `set_role`). Theme
  names are now checked before any file access. `22850f7` adds the vendored release script and
  `release.toml` so the next release uses the shared flow. See [CHANGELOG.md](CHANGELOG.md).
- Checks: `cargo fmt --check`, `cargo clippy --all-targets -D warnings` with default and all
  features, `cargo test` (default, all features, no default features), `cargo doc -D warnings`,
  `cargo package --locked`, `weirdnet_release.py validate`.
  [cargo-semver-checks](https://crates.io/crates/cargo-semver-checks) against 0.1.1 reports
  breaking changes (a default feature removed, a `ColorDepth` variant, `system_variant`'s
  parameter, `Theme` non-exhaustive), so the next release is 0.2.0.
- Not tested: a real OSC 11 query and the OS probes (no terminal or desktop session in tests),
  Windows and macOS config folders, and a Bragi build against this branch.
- Left: release 0.2.0 once m3talsmith approves the version; update the themes guide on
  [net.weirdbrains.com](https://net.weirdbrains.com) with the release.
- Next: Bragi drops its local NO_COLOR, depth, 16-colour and extra-role code and builds its
  theme editor and switcher on the new helpers.

## 2026-11-21: v0.1.1, public-content patch release

- What changed: v0.1.1 republishes the 0.1 line with the public-content rules applied (Weird Brains LLC as the copyright holder, m3talsmith as the author, no machine details), on top of `170d5d7`. No code or API changes.
- Commits: `170d5d7` and the `release: v0.1.1` commit tagged [v0.1.1](https://github.com/WeirdBrains/weirdnet-themes/releases/tag/v0.1.1).
- Checks: cargo fmt, clippy, test and package; the packaged crate was scanned for private terms before publishing.
- Not tested: nothing beyond the crate's own test suite.
- Left: earlier versions on this line are yanked on [crates.io](https://crates.io/crates/weirdnet-themes/versions).
- Next: nothing for this release.


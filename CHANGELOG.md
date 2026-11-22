# Changelog

All notable changes to weirdnet-themes are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-11-22

### Added

- Built-in `bragi` theme.
- Optional roles `surface`, `code_bg`, `diff_add_bg`, `diff_remove_bg`, `link` and `dim`, derived from the required roles when a theme leaves them out (`OPTIONAL_ROLES`, `VariantDef::color`).
- `NO_COLOR` and `TERM=dumb` support: `ColorDepth::None`, `no_color`, and `ColorDepth::parse` / `detect_with` for an app's own colour setting.
- Theme editing and switching helpers: `UserDir` (per-user themes folder), `entries`, `exists`, `is_builtin`, `unique_name`, `duplicate`, `rename`, `delete`, `import`, `export`, `cycle`, `VariantDef::set_role` / `clear_role`, and `mix`.
- `resolve_variant_with` and `detect::Probe` to opt in to the terminal and OS light/dark probes; `COLORFGBG` hint (`detect::env_variant`).
- ratatui: `Theme` gains the optional roles and its `depth`; `to_bg_color`.

### Changed

- **Breaking:** the `terminal-background` feature is no longer on by default.
- **Breaking:** `Mode::System` no longer queries the terminal or runs an OS subprocess unless asked (`resolve_variant_with`); `detect::system_variant` takes a `Probe`.
- **Breaking:** `ColorDepth` has a `None` variant; the ratatui `Theme` has new fields and is `#[non_exhaustive]`.
- In 16 colours, backgrounds and body text use the terminal's own defaults instead of the nearest ANSI colour.
- Theme names are checked before any file is read or written.

## [0.1.1] - 2026-11-21

### Changed

- Package metadata and docs follow the WeirdBrains public-content rules (Weird Brains LLC as the copyright holder, m3talsmith as the author, no machine details). No code or API changes.

## [0.1.0]

- First release.

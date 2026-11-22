# weirdnet-themes

A theme format for terminal and GUI front ends, from [Weird Brains](https://weirdbrains.com):
one TOML file per theme with `[meta]`, `[dark]` and `[light]` tables, 21 required semantic
colour roles plus 6 optional ones (palette names or `#rrggbb`), eight built-in palettes, WCAG
contrast checks and lightness-inverted variant generation.

- **Built-ins.** `tokyo-night` (the default), `catppuccin`, `gruvbox`, `dracula`, `nord`, `one`,
  `solarized` and `bragi`.
- **Roles.** `ROLES` must all be set. `OPTIONAL_ROLES` (`surface`, `code_bg`, `diff_add_bg`,
  `diff_remove_bg`, `link`, `dim`) may be; `VariantDef::color(role)` derives the ones a theme
  leaves out from its required roles (surface is `bg` nudged toward `fg`, code blocks use the
  surface, diff backgrounds tint `bg` toward `up` / `down`, links use `info`, dim text is `muted`
  toward `bg`), so older theme files load unchanged. `VariantDef::resolve(role)` is the raw value.
- **Storage is yours.** `load`, `save`, `list`, `materialize` and `reset` work through the
  `ThemeDirs` trait (a themes folder plus how to create folders and write files). `UserDir` is
  the per-user default: `UserDir::for_app("myapp")` is `<config dir>/myapp/themes`, where the
  config dir comes from [dirs](https://crates.io/crates/dirs) (`$XDG_CONFIG_HOME` or
  `~/.config` on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows). It
  writes atomically, with private permissions on Unix.
- **Editors and switchers.** `entries` (name, display name, built-in or not, load error),
  `exists`, `is_builtin`, `unique_name`, `duplicate`, `rename`, `delete` (a user theme, or a
  user's edited copy of a built-in), `import`, `export`, `cycle` (next or previous theme), and
  `VariantDef::set_role` / `clear_role`. A user file with a built-in's name overrides it.
- **Mode.** `Mode::{Light, Dark, System}`. `resolve_variant` maps it to a `Variant` without
  probing anything: `System` reads the `COLORFGBG` hint, else dark. `resolve_variant_with(mode,
  detect::Probe::ALL)` also asks the terminal background (OSC 11, with the `terminal-background`
  feature) and the OS preference (the freedesktop portal, `defaults` or `reg`).
- **Colour depth.** `ColorDepth::detect` honours [`NO_COLOR`](https://no-color.org) and
  `TERM=dumb` (`ColorDepth::None`), then `COLORTERM` and `TERM`. `ColorDepth::detect_with` takes
  an app's explicit choice (`truecolor`, `256`, `16`, `none`), which wins over `NO_COLOR`.
- **ratatui** (feature `ratatui`). `Theme::from_variant_depth` and `to_color` / `to_bg_color`
  give ratatui colours by depth. In 16 colours, backgrounds and body text are `Color::Reset`
  (the terminal's own), not the nearest ANSI colour; with no colour everything is
  `Color::Reset` and `Theme::has_color` is false, so style with bold, dim and reverse instead.
- **Colours elsewhere.** `to_hex` gives `#rrggbb` (what a Dart or web front end consumes), and
  `mix` blends two colours.
- **Errors.** `ThemeError` (`Invalid`, `Io`, `TomlSer`).

```rust
use weirdnet_themes::{ColorDepth, UserDir, entries, load, resolve_variant, Mode};

let t = weirdnet_themes::builtin("nord").unwrap();
assert_eq!(weirdnet_themes::to_hex(t.dark.resolve("bg").unwrap()), "#2e3440");
assert_eq!(t.dark.color("link"), t.dark.resolve("info"));

if let Some(dir) = UserDir::for_app("myapp") {
    let names: Vec<String> = entries(&dir).unwrap().into_iter().map(|e| e.name).collect();
    let theme = load(&dir, &names[0]).unwrap();
    let _colours = theme.variant(resolve_variant(Mode::System));
    let _depth = ColorDepth::detect();
}
```

See [CHANGELOG.md](CHANGELOG.md) for changes and [HANDOFF.md](HANDOFF.md) for work in progress.

## License

BSD-3-Clause, copyright [Weird Brains](https://weirdbrains.com) LLC. See [LICENSE](LICENSE).

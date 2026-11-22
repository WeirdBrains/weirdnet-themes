//! Terminal rendering of a theme: colour depth always; ratatui colours with the `ratatui`
//! feature (the only part of the crate that depends on ratatui).
#[cfg(feature = "ratatui")]
use super::{VariantDef, dist, nearest_256};
#[cfg(feature = "ratatui")]
use ratatui::style::Color;

/// Terminal color depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorDepth {
    TrueColor,
    Ansi256,
    /// The 16 ANSI colours. Backgrounds and body text use the terminal's own defaults.
    Ansi16,
    /// No colour at all ([`NO_COLOR`](https://no-color.org), `TERM=dumb`): text attributes
    /// (bold, dim, reverse) only.
    None,
}

impl ColorDepth {
    /// From the environment: [`NO_COLOR`](https://no-color.org) set and not empty, or
    /// `TERM=dumb`, is [`None`](ColorDepth::None); then `COLORTERM`, then `TERM`.
    pub fn detect() -> Self {
        if no_color() {
            return ColorDepth::None;
        }
        let ct = std::env::var("COLORTERM").unwrap_or_default().to_ascii_lowercase();
        if ct.contains("truecolor") || ct.contains("24bit") {
            return ColorDepth::TrueColor;
        }
        let term = std::env::var("TERM").unwrap_or_default();
        if term == "dumb" {
            ColorDepth::None
        } else if term == "linux" || term.contains("16color") || term == "xterm" || term == "vt100" {
            ColorDepth::Ansi16
        } else if term.contains("256") {
            ColorDepth::Ansi256
        } else {
            ColorDepth::TrueColor
        }
    }

    /// An app's explicit choice (a `--color` flag or setting) when it parses, else
    /// [`detect`](ColorDepth::detect). Per [no-color.org](https://no-color.org), an explicit
    /// choice overrides `NO_COLOR`.
    pub fn detect_with(choice: Option<&str>) -> Self {
        choice.and_then(Self::parse).unwrap_or_else(Self::detect)
    }

    /// `truecolor` / `24bit` / `24`, `256`, `16`, `none` / `off` / `0` (any case).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "truecolor" | "24bit" | "24" => Some(ColorDepth::TrueColor),
            "256" => Some(ColorDepth::Ansi256),
            "16" => Some(ColorDepth::Ansi16),
            "none" | "off" | "0" => Some(ColorDepth::None),
            _ => None,
        }
    }

    /// Whether there is any colour.
    pub fn has_color(self) -> bool {
        self != ColorDepth::None
    }
}

/// Whether [`NO_COLOR`](https://no-color.org) is set to a non-empty value.
pub fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
}

#[cfg(feature = "ratatui")]
const ANSI16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (128, 0, 0),
    (0, 128, 0),
    (128, 128, 0),
    (0, 0, 128),
    (128, 0, 128),
    (0, 128, 128),
    (192, 192, 192),
    (128, 128, 128),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (0, 0, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];

/// A foreground colour at `depth`: RGB, the nearest xterm-256 or ANSI-16 index, or
/// `Color::Reset` with no colour.
#[cfg(feature = "ratatui")]
pub fn to_color(c: (u8, u8, u8), depth: ColorDepth) -> Color {
    match depth {
        ColorDepth::TrueColor => Color::Rgb(c.0, c.1, c.2),
        ColorDepth::Ansi256 => Color::Indexed(nearest_256(c)),
        ColorDepth::Ansi16 => {
            let i = ANSI16.iter().enumerate().min_by_key(|(_, a)| dist(c, **a)).map(|(i, _)| i).unwrap_or(7);
            Color::Indexed(i as u8)
        }
        ColorDepth::None => Color::Reset,
    }
}

/// A background (or body text) colour at `depth`: like [`to_color`], but `Color::Reset` (the
/// terminal's own default) in 16 colours and with no colour, since the nearest ANSI colour is
/// rarely the theme's and usually black.
#[cfg(feature = "ratatui")]
pub fn to_bg_color(c: (u8, u8, u8), depth: ColorDepth) -> Color {
    match depth {
        ColorDepth::Ansi16 | ColorDepth::None => Color::Reset,
        _ => to_color(c, depth),
    }
}

#[cfg(feature = "ratatui")]
/// A resolved variant ready for rendering. `bg`, `fg`, `surface`, `code_bg` and the diff
/// backgrounds go through [`to_bg_color`]; everything else through [`to_color`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Theme {
    /// The depth the colours were made for.
    pub depth: ColorDepth,
    pub bg: Color,
    pub fg: Color,
    pub muted: Color,
    pub border: Color,
    pub border_focus: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub accent: Color,
    pub header: Color,
    pub up: Color,
    pub down: Color,
    pub warning: Color,
    pub error: Color,
    pub success: Color,
    pub info: Color,
    pub charts: [Color; 6],
    pub surface: Color,
    pub code_bg: Color,
    pub diff_add_bg: Color,
    pub diff_remove_bg: Color,
    pub link: Color,
    pub dim: Color,
}

#[cfg(feature = "ratatui")]
impl Theme {
    /// At the detected depth ([`ColorDepth::detect`]).
    pub fn from_variant(v: &VariantDef) -> Self {
        Self::from_variant_depth(v, ColorDepth::detect())
    }

    pub fn from_variant_depth(v: &VariantDef, depth: ColorDepth) -> Self {
        let rgb = |r: &str| v.color(r).unwrap_or((128, 128, 128));
        let c = |r: &str| to_color(rgb(r), depth);
        let g = |r: &str| to_bg_color(rgb(r), depth);
        Theme {
            depth,
            bg: g("bg"),
            fg: g("fg"),
            muted: c("muted"),
            border: c("border"),
            border_focus: c("border_focus"),
            selection_bg: c("selection_bg"),
            selection_fg: c("selection_fg"),
            accent: c("accent"),
            header: c("header"),
            up: c("up"),
            down: c("down"),
            warning: c("warning"),
            error: c("error"),
            success: c("success"),
            info: c("info"),
            charts: [c("chart_1"), c("chart_2"), c("chart_3"), c("chart_4"), c("chart_5"), c("chart_6")],
            surface: g("surface"),
            code_bg: g("code_bg"),
            diff_add_bg: g("diff_add_bg"),
            diff_remove_bg: g("diff_remove_bg"),
            link: c("link"),
            dim: c("dim"),
        }
    }

    /// Whether there is any colour (style with modifiers instead when not).
    pub fn has_color(&self) -> bool {
        self.depth.has_color()
    }
}

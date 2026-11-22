//! System light/dark detection.
//!
//! [`system_variant`] runs only what a [`Probe`] turns on: the terminal background (OSC 11,
//! which also needs the `terminal-background` feature) and the OS preference (the freedesktop
//! portal over `busctl` or `gdbus` on Linux, `defaults` on macOS, `reg` on Windows). The
//! `COLORFGBG` environment hint is always read, since it costs nothing. Else dark.
use super::Variant;
use std::io::IsTerminal;
use std::process::Command;

/// Which probes `Mode::System` may run. [`Probe::NONE`] (the default) runs none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub struct Probe {
    /// Ask the terminal for its background (OSC 11). Needs the `terminal-background` feature
    /// and a terminal on stdin and stdout.
    pub terminal: bool,
    /// Ask the OS for its light or dark preference (runs a short subprocess).
    pub os: bool,
}

impl Probe {
    /// No probes: environment hints only.
    pub const NONE: Probe = Probe { terminal: false, os: false };
    /// Every probe.
    pub const ALL: Probe = Probe { terminal: true, os: true };

    /// The same probes, with the terminal query on or off.
    pub const fn terminal(mut self, on: bool) -> Self {
        self.terminal = on;
        self
    }

    /// The same probes, with the OS query on or off.
    pub const fn os(mut self, on: bool) -> Self {
        self.os = on;
        self
    }
}

/// Light or dark for `Mode::System`: the terminal (when probed), `COLORFGBG`, the OS (when
/// probed), else dark.
pub fn system_variant(probe: Probe) -> Variant {
    probe.terminal.then(terminal_variant).flatten().or_else(env_variant).or_else(|| probe.os.then(os_variant).flatten()).unwrap_or(Variant::Dark)
}

/// The `COLORFGBG` hint (`fg;bg` ANSI indexes, set by rxvt, Konsole and others): background 7
/// or 9 to 15 is light, other numbers dark. `None` when unset or unreadable.
pub fn env_variant() -> Option<Variant> {
    parse_colorfgbg(&std::env::var("COLORFGBG").ok()?)
}

/// Parses a `COLORFGBG` value (`15;0`, `0;default;15`).
pub fn parse_colorfgbg(s: &str) -> Option<Variant> {
    let bg: u8 = s.rsplit(';').next()?.trim().parse().ok()?;
    match bg {
        7 | 9..=15 => Some(Variant::Light),
        0..=6 | 8 => Some(Variant::Dark),
        _ => None,
    }
}

/// Query the terminal background (OSC 11) when attached to a terminal. This writes to the
/// terminal and waits briefly for its answer.
/// Without the `terminal-background` feature this is always `None`.
pub fn terminal_variant() -> Option<Variant> {
    if !std::io::stdout().is_terminal() || !std::io::stdin().is_terminal() {
        return None;
    }
    terminal_luma().map(|luma| if luma < 0.5 { Variant::Dark } else { Variant::Light })
}

#[cfg(feature = "terminal-background")]
fn terminal_luma() -> Option<f32> {
    terminal_light::luma().ok()
}

#[cfg(not(feature = "terminal-background"))]
fn terminal_luma() -> Option<f32> {
    None
}

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let o = Command::new(cmd).args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).to_string())
}

/// The OS preference. Runs a subprocess; only called when you ask (or through [`Probe::os`]).
pub fn os_variant() -> Option<Variant> {
    if cfg!(target_os = "macos") {
        // Prints "Dark" in dark mode; fails (key absent) in light mode.
        return Some(match run("defaults", &["read", "-g", "AppleInterfaceStyle"]) {
            Some(s) if s.trim() == "Dark" => Variant::Dark,
            _ => Variant::Light,
        });
    }
    if cfg!(windows) {
        let s = run("reg", &["query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "/v", "AppsUseLightTheme"])?;
        return Some(if s.contains("0x0") { Variant::Dark } else { Variant::Light });
    }
    // freedesktop portal: org.freedesktop.appearance color-scheme (1 = dark, 2 = light, 0 = no preference)
    let out = run(
        "busctl",
        &[
            "--user",
            "call",
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
            "ReadOne",
            "ss",
            "org.freedesktop.appearance",
            "color-scheme",
        ],
    )
    .or_else(|| {
        run(
            "gdbus",
            &[
                "call",
                "--session",
                "--dest",
                "org.freedesktop.portal.Desktop",
                "--object-path",
                "/org/freedesktop/portal/desktop",
                "--method",
                "org.freedesktop.portal.Settings.ReadOne",
                "org.freedesktop.appearance",
                "color-scheme",
            ],
        )
    })?;
    parse_portal(&out)
}

/// Parse `v u 1` (busctl) or `(<uint32 1>,)` (gdbus).
pub fn parse_portal(s: &str) -> Option<Variant> {
    let n = s.chars().rev().find(|c| c.is_ascii_digit())?;
    match n {
        '1' => Some(Variant::Dark),
        '2' => Some(Variant::Light),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portal_parsing() {
        assert_eq!(parse_portal("v u 1\n"), Some(Variant::Dark));
        assert_eq!(parse_portal("(<<uint32 2>>,)\n"), Some(Variant::Light));
        assert_eq!(parse_portal("v u 0"), None);
    }

    #[test]
    fn colorfgbg_and_probes() {
        assert_eq!(parse_colorfgbg("15;0"), Some(Variant::Dark));
        assert_eq!(parse_colorfgbg("0;15"), Some(Variant::Light));
        assert_eq!(parse_colorfgbg("0;default;7"), Some(Variant::Light));
        assert_eq!(parse_colorfgbg("0;99"), None);
        assert_eq!(parse_colorfgbg("nope"), None);
        assert_eq!(Probe::default(), Probe::NONE);
        assert_eq!(Probe::NONE.os(true), Probe { terminal: false, os: true });
        assert_eq!(Probe::ALL.terminal(false).os(false), Probe::NONE);
    }
}

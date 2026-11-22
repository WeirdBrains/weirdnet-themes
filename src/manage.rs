//! Theme editing and switching on top of [`ThemeDirs`]: the per-user [`UserDir`], [`entries`]
//! for a switcher, and [`duplicate`], [`rename`], [`delete`], [`import`] and [`export`] for an
//! editor.
use crate::{BUILTINS, Result, THEME_FILE, ThemeDirs, ThemeError, ThemeFile, builtin, list, load, save, valid_name};
use std::path::{Path, PathBuf};

/// The per-user themes folder: `<config dir>/<app>/themes`, with private permissions on Unix
/// (folders `0700`, files `0600`) and atomic writes.
///
/// The config dir is [`dirs::config_dir`]: `$XDG_CONFIG_HOME` or `~/.config` on Linux and the
/// BSDs, `~/Library/Application Support` on macOS, `%APPDATA%` (roaming) on Windows. So
/// `UserDir::for_app("bragi")` is `~/.config/bragi/themes` on Linux.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDir(PathBuf);

impl UserDir {
    /// A themes folder at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// `<config dir>/<app>/themes`. `None` when the platform has no config dir or `app` is not a
    /// plain name (letters, digits, `-`, `_`, `.`; not starting with `.`).
    pub fn for_app(app: &str) -> Option<Self> {
        let plain = !app.is_empty() && !app.starts_with('.') && app.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
        if !plain {
            return None;
        }
        Some(Self(dirs::config_dir()?.join(app).join("themes")))
    }

    /// The folder.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl ThemeDirs for UserDir {
    fn themes_dir(&self) -> PathBuf {
        self.0.clone()
    }

    fn ensure_dir(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(())
    }

    fn write_private(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let tmp = path.with_file_name(format!(".{name}.tmp"));
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    }
}

/// One row of a theme switcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeEntry {
    /// Folder name, what [`load`] takes.
    pub name: String,
    /// `[meta] display_name`, else the name.
    pub display_name: String,
    /// Ships with the crate (see [`BUILTINS`]).
    pub builtin: bool,
    /// A file in the themes folder: a user theme, or a built-in the user has edited.
    pub user_file: bool,
    /// Why the theme does not load, if it does not.
    pub error: Option<String>,
}

/// Whether `name` is a built-in theme.
pub fn is_builtin(name: &str) -> bool {
    BUILTINS.iter().any(|(n, _)| *n == name)
}

fn has_file(paths: &(impl ThemeDirs + ?Sized), name: &str) -> bool {
    valid_name(name).is_ok() && paths.theme_dir(name).join(THEME_FILE).is_file()
}

/// Whether a theme called `name` exists (built-in or in the themes folder).
pub fn exists(paths: &(impl ThemeDirs + ?Sized), name: &str) -> bool {
    is_builtin(name) || has_file(paths, name)
}

/// Every theme in [`list`] order (built-ins first), with display names and load errors, for a
/// switcher or the editor's theme list.
pub fn entries(paths: &(impl ThemeDirs + ?Sized)) -> Result<Vec<ThemeEntry>> {
    Ok(list(paths)?
        .into_iter()
        .map(|name| {
            let (display_name, error) = match load(paths, &name) {
                Ok(t) => (t.display_name(), None),
                Err(e) => (builtin(&name).map(|t| t.display_name()).unwrap_or_else(|| name.clone()), Some(e.to_string())),
            };
            ThemeEntry { builtin: is_builtin(&name), user_file: has_file(paths, &name), name, display_name, error }
        })
        .collect())
}

/// `base` if no theme has that name, else `base-2`, `base-3`, ...
pub fn unique_name(paths: &(impl ThemeDirs + ?Sized), base: &str) -> String {
    if !exists(paths, base) {
        return base.to_string();
    }
    (2..).map(|i| format!("{base}-{i}")).find(|n| !exists(paths, n)).expect("an unused name")
}

fn must_be_free(paths: &(impl ThemeDirs + ?Sized), name: &str) -> Result<()> {
    valid_name(name)?;
    if exists(paths, name) {
        return Err(ThemeError::invalid("theme", format!("`{name}` already exists")));
    }
    Ok(())
}

/// Copies theme `src` (built-in or user) to a new user theme `dest`, or to `<src>-copy` (made
/// unique) when `dest` is `None`. Returns the new name. The copy's display name is cleared.
pub fn duplicate(paths: &(impl ThemeDirs + ?Sized), src: &str, dest: Option<&str>) -> Result<String> {
    let mut t = load(paths, src)?;
    let dest = match dest {
        Some(d) => d.to_string(),
        None => unique_name(paths, &format!("{src}-copy")),
    };
    must_be_free(paths, &dest)?;
    t.meta.name = dest.clone();
    t.meta.display_name = None;
    save(paths, &t)?;
    Ok(dest)
}

/// Renames user theme `from` to `to`, moving its folder (assets included). Built-ins cannot be
/// renamed; [`duplicate`] them instead.
pub fn rename(paths: &(impl ThemeDirs + ?Sized), from: &str, to: &str) -> Result<()> {
    valid_name(from)?;
    if is_builtin(from) {
        return Err(ThemeError::invalid("theme", format!("`{from}` is a built-in theme; duplicate it instead")));
    }
    if !has_file(paths, from) {
        return Err(ThemeError::invalid("theme", format!("`{from}` not found in {}", paths.themes_dir().display())));
    }
    must_be_free(paths, to)?;
    let mut t = load(paths, from)?;
    std::fs::rename(paths.theme_dir(from), paths.theme_dir(to))?;
    t.meta.name = to.to_string();
    save(paths, &t)
}

/// What [`delete`] removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Removed {
    /// A user theme's folder.
    UserTheme,
    /// The user's edited copy of a built-in, which is now back to the shipped version.
    BuiltinOverride,
}

/// Deletes user theme `name` (its whole folder), or the user's edited copy of a built-in.
/// A built-in without a user file cannot be deleted. Switch the app away from `name` first if it
/// is a user theme.
pub fn delete(paths: &(impl ThemeDirs + ?Sized), name: &str) -> Result<Removed> {
    valid_name(name)?;
    if !has_file(paths, name) {
        return Err(if is_builtin(name) {
            ThemeError::invalid("theme", format!("`{name}` is a built-in theme and cannot be deleted"))
        } else {
            ThemeError::invalid("theme", format!("`{name}` not found in {}", paths.themes_dir().display()))
        });
    }
    std::fs::remove_dir_all(paths.theme_dir(name))?;
    Ok(if is_builtin(name) { Removed::BuiltinOverride } else { Removed::UserTheme })
}

/// Imports a theme file (any path, in this crate's format) as a user theme, named `as_name` or
/// its own `[meta] name`. Refuses a name that already exists. Returns the name.
pub fn import(paths: &(impl ThemeDirs + ?Sized), file: &Path, as_name: Option<&str>) -> Result<String> {
    let text = std::fs::read_to_string(file)?;
    let mut t = ThemeFile::parse(&text).map_err(|e| ThemeError::invalid("theme", format!("{}: {e}", file.display())))?;
    if let Some(n) = as_name {
        t.meta.name = n.to_string();
    }
    must_be_free(paths, &t.meta.name)?;
    save(paths, &t)?;
    Ok(t.meta.name)
}

/// Writes theme `name` to `file` as TOML (for sharing).
pub fn export(paths: &(impl ThemeDirs + ?Sized), name: &str, file: &Path) -> Result<()> {
    let t = load(paths, name)?;
    std::fs::write(file, t.to_toml()?)?;
    Ok(())
}

/// The theme after (`forward`) or before `current` in `names`, wrapping around, for a "next
/// theme" key. The first name when `current` is not in the list; `None` for an empty list.
pub fn cycle<'a>(names: &'a [String], current: &str, forward: bool) -> Option<&'a str> {
    let n = names.len();
    if n == 0 {
        return None;
    }
    let i = match names.iter().position(|x| x == current) {
        None => 0,
        Some(i) if forward => (i + 1) % n,
        Some(i) => (i + n - 1) % n,
    };
    Some(names[i].as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PlainDir;

    #[test]
    fn user_dir_for_app_and_private_writes() {
        assert!(UserDir::for_app("").is_none());
        assert!(UserDir::for_app("../x").is_none());
        assert!(UserDir::for_app(".hidden").is_none());
        if let Some(d) = UserDir::for_app("bragi") {
            assert!(d.path().ends_with(Path::new("bragi").join("themes")));
        }
        let tmp = tempfile::tempdir().unwrap();
        let d = UserDir::new(tmp.path().join("themes"));
        save(&d, &builtin("nord").unwrap()).unwrap();
        let f = d.theme_dir("nord").join(THEME_FILE);
        assert_eq!(load(&d, "nord").unwrap(), builtin("nord").unwrap());
        assert!(!d.theme_dir("nord").join(".interface.toml.tmp").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&f).unwrap().permissions().mode() & 0o777, 0o600);
            assert_eq!(std::fs::metadata(d.theme_dir("nord")).unwrap().permissions().mode() & 0o777, 0o700);
        }
        let _ = f;
    }

    #[test]
    fn editor_operations() {
        let tmp = tempfile::tempdir().unwrap();
        let d = PlainDir(tmp.path().join("t"));
        assert!(is_builtin("bragi") && !is_builtin("mine"));
        assert_eq!(duplicate(&d, "nord", None).unwrap(), "nord-copy");
        assert_eq!(duplicate(&d, "nord", None).unwrap(), "nord-copy-2");
        assert_eq!(unique_name(&d, "nord"), "nord-2");
        assert_eq!(duplicate(&d, "nord", Some("nord-copy")).unwrap_err().to_string(), "invalid theme: `nord-copy` already exists");
        assert!(duplicate(&d, "nord", Some("a/b")).is_err());
        let t = load(&d, "nord-copy").unwrap();
        assert_eq!((t.meta.name.as_str(), t.display_name().as_str()), ("nord-copy", "nord-copy"));

        rename(&d, "nord-copy", "mine").unwrap();
        assert!(!exists(&d, "nord-copy") && exists(&d, "mine"));
        assert_eq!(load(&d, "mine").unwrap().meta.name, "mine");
        assert!(rename(&d, "nord", "x").unwrap_err().to_string().contains("built-in"));
        assert!(rename(&d, "mine", "nord").unwrap_err().to_string().contains("already exists"));

        // An edited built-in overrides the shipped one until deleted.
        let mut n = load(&d, "nord").unwrap();
        n.dark.set_role("accent", "#FF0000").unwrap();
        save(&d, &n).unwrap();
        assert_eq!(load(&d, "nord").unwrap().dark.roles["accent"], "#ff0000");
        assert_eq!(delete(&d, "nord").unwrap(), Removed::BuiltinOverride);
        assert_eq!(load(&d, "nord").unwrap(), builtin("nord").unwrap());
        assert!(delete(&d, "nord").unwrap_err().to_string().contains("cannot be deleted"));
        assert_eq!(delete(&d, "mine").unwrap(), Removed::UserTheme);
        assert!(!d.theme_dir("mine").exists());
        assert!(delete(&d, "mine").unwrap_err().to_string().contains("not found"));
        assert!(delete(&d, "..").is_err());

        let file = tmp.path().join("shared.toml");
        export(&d, "bragi", &file).unwrap();
        assert!(import(&d, &file, None).unwrap_err().to_string().contains("already exists"));
        assert_eq!(import(&d, &file, Some("bragi-mine")).unwrap(), "bragi-mine");
        assert_eq!(load(&d, "bragi-mine").unwrap().dark, builtin("bragi").unwrap().dark);
    }

    #[test]
    fn entries_for_a_switcher() {
        let tmp = tempfile::tempdir().unwrap();
        let d = PlainDir(tmp.path().join("t"));
        duplicate(&d, "bragi", Some("zz")).unwrap();
        d.ensure_dir(&d.theme_dir("broken")).unwrap();
        std::fs::write(d.theme_dir("broken").join(THEME_FILE), "nope").unwrap();
        let e = entries(&d).unwrap();
        assert_eq!(e.len(), BUILTINS.len() + 2);
        assert_eq!(e[0], ThemeEntry { name: "tokyo-night".into(), display_name: "Tokyo Night".into(), builtin: true, user_file: false, error: None });
        let bragi = e.iter().find(|x| x.name == "bragi").unwrap();
        assert_eq!(bragi.display_name, "Bragi");
        let broken = e.iter().find(|x| x.name == "broken").unwrap();
        assert!(broken.error.is_some() && broken.user_file && !broken.builtin);
        let zz = e.last().unwrap();
        assert_eq!((zz.name.as_str(), zz.user_file, zz.error.is_none()), ("zz", true, true));

        let names: Vec<String> = e.iter().map(|x| x.name.clone()).collect();
        assert_eq!(cycle(&names, "tokyo-night", true), Some("catppuccin"));
        assert_eq!(cycle(&names, "tokyo-night", false), Some("zz"));
        assert_eq!(cycle(&names, "zz", true), Some("tokyo-night"));
        assert_eq!(cycle(&names, "gone", true), Some("tokyo-night"));
        assert_eq!(cycle(&[], "x", true), None);
    }
}

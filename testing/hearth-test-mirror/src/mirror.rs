use crate::Pin;
use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
};

/// The directory below the cache, holding mirrors by digest.
pub const CACHE: &str = "hearth-test-mirror";
/// One pin's fetched mirror (testing.md, 6.2).
#[derive(Clone, Debug)]
pub struct Mirror {
    root: PathBuf,
    pin: Pin,
}
/// No absolute cache home, or the pinned digest has not been fetched.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Missing {
    Home,
    Unfetched { path: PathBuf },
}
impl Mirror {
    pub fn find(pin: &Pin) -> Result<Self, Missing> {
        let cache = env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| env::var_os("HOME").map(PathBuf::from).filter(|p| p.is_absolute()).map(|p| p.join(".cache")))
            .ok_or(Missing::Home)?;
        Self::find_in(&cache, pin)
    }
    pub fn find_in(cache: &Path, pin: &Pin) -> Result<Self, Missing> {
        let root = cache.join(CACHE).join(&*pin.digest);
        if !root.is_dir() {
            return Err(Missing::Unfetched { path: root });
        }
        Ok(Self { root, pin: pin.clone() })
    }
    #[must_use]
    pub fn pin(&self) -> &Pin {
        &self.pin
    }
    /// The checked, decompressed index; this directory is never served.
    #[must_use]
    pub fn index(&self, component: &str) -> PathBuf {
        self.root.join("checked").join(component).join(format!("binary-{}", self.pin.architecture)).join("Packages")
    }
    /// A regular file below `archive/`, without dot, empty or symbolic-link segments.
    #[must_use]
    pub fn file(&self, path: &Path) -> Option<PathBuf> {
        let text = path.to_str()?;
        if !text.starts_with("archive/") {
            return None;
        }
        let mut file = self.root.clone();
        if fs::symlink_metadata(&file).ok()?.file_type().is_symlink() {
            return None;
        }
        for segment in text.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return None;
            }
            file.push(segment);
            if fs::symlink_metadata(&file).ok()?.file_type().is_symlink() {
                return None;
            }
        }
        file.is_file().then_some(file)
    }
}
impl fmt::Display for Missing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Home => write!(f, "no absolute XDG_CACHE_HOME or HOME; run testing/mirror/fetch with a cache home"),
            Self::Unfetched { path } => write!(f, "{} is not fetched; run testing/mirror/fetch", path.display()),
        }
    }
}

use crate::common::{AppDataType, AppDirsError, AppInfo};
use std::fs;
use std::path::{Path, PathBuf};
use crate::utils;

#[cfg(target_os="macos")]
mod platform {
    mod macos;
    pub use self::macos::*;
}
#[cfg(all(unix, not(target_os="macos"), not(target_os="android")))]
mod platform {
    mod unix;
    pub use self::unix::*;
}
#[cfg(windows)]
mod platform {
    mod windows;
    pub use self::windows::*;
}
#[cfg(not(any(windows, unix, target_os="macos",)))]
mod platform {
    mod unknown;
    pub use self::unknown::*;
}
#[cfg(target_os="redox")]
mod platform {
    mod redox;
    pub use self::redox::*;
}
#[cfg(target_os="android")]
mod platform {
    mod android;
    pub use self::android::*;
}

/// Creates (if necessary) and returns path to **app-specific** data
/// **subdirectory** for provided data type and subdirectory path.
///
/// The `path` parameter should be a valid relative path separated by
/// **forward slashes** (`/`).
///
/// If the directory structure does not exist, this function will recursively
/// create the full hierarchy. Therefore, a result of `Ok` guarantees that the
/// returned path exists.
///
/// If there are multiple possible app dis, this functions picks the first one (see
/// [`get_app_dirs.get_app_dirs.html)).
pub fn app_dir(t: AppDataType, app: &AppInfo, path: &str) -> Result<PathBuf, AppDirsError> {
    let path = get_app_dir(t, app, &path)?;
    match fs::create_dir_all(&path) {
        Ok(..) => Ok(path),
        Err(e) => Err(e.into()),
    }
}

/// Returns (but **does not create**) path to **app-specific** data
/// **subdirectory** for provided data type and subdirectory path.
///
/// The `path` parameter should be a valid relative path separated by
/// **forward slashes** (`/`).
///
/// A result of `Ok` means that we determined where the data SHOULD go, but
/// it DOES NOT guarantee that the directory actually exists. (See
/// [`app_dir`](fn.app_dir.html).)
///
/// If there are multiple possible app dis, this functions picks the first one (see
/// [`get_app_dirs.get_app_dirs.html)).
pub fn get_app_dir(t: AppDataType, app: &AppInfo, path: &str) -> Result<PathBuf, AppDirsError> {
    get_app_dirs(t, app, path).map(|mut v| v.remove(0))
}


/// Returns (but **does not create**) the paths to all possible **app-specific** data
/// **subdirectories** for the provided data type and subdirectory path.
///
/// The `path` parameter should be a valid relative path separated by
/// **forward slashes** (`/`).
///
/// The returned vector contains at least one element.  On some platforms, for example on Unix
/// platforms, it may contain multiple elements.
///
/// A result of `Ok` means that we determined where the data SHOULD go, but
/// it DOES NOT guarantee that the directory actually exists. (See
/// [`app_dir`](fn.app_dir.html).)
pub fn get_app_dirs(t: AppDataType, app: &AppInfo, path: &str) -> Result<Vec<PathBuf>, AppDirsError> {
    let mut subdir = PathBuf::new();
    for component in path.split("/").filter(|s| s.len() > 0) {
        subdir.push(utils::sanitized(component));
    }

    get_app_roots(t, app).map(|v| select_subdirs(v, subdir))
}

/// Creates (if necessary) and returns path to **app-specific** data
/// directory for provided data type.
///
/// If the directory structure does not exist, this function will recursively
/// create the full hierarchy. Therefore, a result of `Ok` guarantees that the
/// returned path exists.
///
/// If there are multiple possible app roots, this functions picks the first one (see
/// [`get_app_roots`](fn.get_app_roots.html)).
pub fn app_root(t: AppDataType, app: &AppInfo) -> Result<PathBuf, AppDirsError> {
    let path = get_app_root(t, app)?;
    match fs::create_dir_all(&path) {
        Ok(..) => Ok(path),
        Err(e) => Err(e.into()),
    }
}

/// Returns (but **does not create**) path to **app-specific** data directory
/// for provided data type.
///
/// A result of `Ok` means that we determined where the data SHOULD go, but
/// it DOES NOT guarantee that the directory actually exists. (See
/// [`app_root`](fn.app_root.html).)
///
/// If there are multiple possible app roots, this functions picks the first one (see
/// [`get_app_roots`](fn.get_app_roots.html)).
pub fn get_app_root(t: AppDataType, app: &AppInfo) -> Result<PathBuf, AppDirsError> {
    get_app_roots(t, app).map(|mut v| v.remove(0))
}

/// Returns (but **does not create**) the paths of all possible **app-specific** data directories
/// for the provided data type.
///
/// The returned vector contains at least one element.  On some platforms, for example on Unix
/// platforms, it may contain multiple elements.
///
/// A result of `Ok` means that we determined where the data SHOULD go, but
/// it DOES NOT guarantee that the directory actually exists. (See
/// [`app_root`](fn.app_root.html).)
pub fn get_app_roots(t: AppDataType, app: &AppInfo) -> Result<Vec<PathBuf>, AppDirsError> {
    if app.author.len() == 0 || app.name.len() == 0 {
        return Err(AppDirsError::InvalidAppInfo);
    }

    let mut subdir = PathBuf::new();
    if platform::USE_AUTHOR {
        subdir.push(utils::sanitized(app.author));
    }
    subdir.push(utils::sanitized(app.name));

    get_data_roots(t).map(|v| select_subdirs(v, subdir))
}

/// Creates (if necessary) and returns path to **top-level** data directory
/// for provided data type.
///
/// If the directory structure does not exist, this function will recursively
/// create the full hierarchy. Therefore, a result of `Ok` guarantees that the
/// returned path exists.
///
/// If there are multiple possible data roots, this functions picks the first one (see
/// [`get_data_roots`](fn.get_data_roots.html)).
pub fn data_root(t: AppDataType) -> Result<PathBuf, AppDirsError> {
    let path = get_data_root(t)?;
    match fs::create_dir_all(&path) {
        Ok(..) => Ok(path),
        Err(e) => Err(e.into()),
    }
}

/// Returns (but **does not create**) path to **top-level** data directory for
/// provided data type.
///
/// A result of `Ok` means that we determined where the data SHOULD go, but
/// it DOES NOT guarantee that the directory actually exists. (See
/// [`data_root`](fn.data_root.html).)
///
/// If there are multiple possible data roots, this functions picks the first one (see
/// [`get_data_roots`](fn.get_data_roots.html)).
pub fn get_data_root(t: AppDataType) -> Result<PathBuf, AppDirsError> {
    get_data_roots(t).map(|mut v| v.remove(0))
}

/// Returns (but **does not create**) the paths of all possible **top-level** data directories for
/// the provided data type.
///
/// The returned vector contains at least one element.  On some platforms, for example on Unix
/// platforms, it may contain multiple elements.
///
/// A result of `Ok` means that we determined where the data SHOULD go, but
/// it DOES NOT guarantee that the directory actually exists. (See
/// [`data_root`](fn.data_root.html).)
pub fn get_data_roots(t: AppDataType) -> Result<Vec<PathBuf>, AppDirsError> {
    let dirs = platform::get_app_dirs(t)?;
    if dirs.is_empty() {
        Err(AppDirsError::NotSupported)
    } else {
        Ok(select_dirs(dirs))
    }
}

/// Select all valid subdirectories of the directories in the given vector.
///
/// If none of the subdirectories exist, this function returns all subdirectories.  Otherwise, only
/// the existing subdirectories are returned.
fn select_subdirs(mut vec: Vec<PathBuf>, subdir: impl AsRef<Path>) -> Vec<PathBuf> {
    for path in vec.iter_mut() {
        path.push(subdir.as_ref());
    }
    select_dirs(vec)
}

/// Select all valid directories from the given vector.
///
/// If the vector contains only one element or if none of the elements of the vector is a
/// directory, it is returned without modification.  Otherwise, only the paths that point to an
/// existing directory are returned.
fn select_dirs(vec: Vec<PathBuf>) -> Vec<PathBuf> {
    if vec.len() > 1 && vec.iter().any(|path| path.is_dir()) {
        vec.into_iter().filter(|path| path.is_dir()).collect()
    } else {
        vec
    }
}

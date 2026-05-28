use fd_lock::RwLock;
use jwalk::WalkDir;
use rayon::prelude::*;
use std::fs::{OpenOptions, copy, create_dir_all, hard_link, rename};
use std::io::{Error, ErrorKind, Result};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkStrategy {
    Reflink,
    Hardlink,
    Copy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Refresh {
    Default,
    Force,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    None,
    Flush,
}

#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
    refresh: Refresh,
    durability: Durability,
    store: Arc<ContentAddressableStore>,
    _temp_dir: Option<Arc<tempfile::TempDir>>,
}

impl Cache {
    pub fn new(
        root: PathBuf,
        refresh: Refresh,
        durability: Durability,
        no_cache: bool,
    ) -> Result<Self> {
        let (actual_root, temp_dir) = if no_cache {
            let t_dir = tempfile::TempDir::new()?;
            (t_dir.path().to_path_buf(), Some(Arc::new(t_dir)))
        } else {
            (root, None)
        };

        create_dir_all(&actual_root)?;
        create_dir_all(actual_root.join("locks"))?;

        let store_dir = actual_root.join("store");
        create_dir_all(&store_dir)?;

        Ok(Self {
            root: actual_root,
            refresh,
            durability,
            store: Arc::new(ContentAddressableStore {
                root: store_dir,
                durability,
                link_strategy: OnceLock::new(),
            }),
            _temp_dir: temp_dir,
        })
    }

    #[inline]
    pub fn store(&self) -> &ContentAddressableStore {
        &self.store
    }

    #[inline]
    pub fn refresh(&self) -> &Refresh {
        &self.refresh
    }

    #[inline]
    pub fn durability(&self) -> Durability {
        self.durability
    }

    pub fn guard_package<F, R>(&self, package_id: &str, shared: bool, op: F) -> Result<R>
    where
        F: FnOnce(&Path) -> Result<R>,
    {
        let package_key = package_key(package_id);
        let lock_path = self.root.join("locks").join(format!("{package_key}.lock"));
        let target_dir = self.store.get_package_path(package_id);

        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)?;

        let mut rw_lock = RwLock::new(file);

        if shared {
            let _guard = rw_lock
                .read()
                .map_err(|e| Error::new(ErrorKind::Other, e))?;
            op(&target_dir)
        } else {
            let _guard = rw_lock
                .write()
                .map_err(|e| Error::new(ErrorKind::Other, e))?;
            op(&target_dir)
        }
    }

    pub fn resolve_package<F>(&self, package_id: &str, fetch_fallback: F) -> Result<PathBuf>
    where
        F: FnOnce(&Path) -> Result<()>,
    {
        validate_package_id(package_id)?;
        let target_dir = self.store.get_package_path(package_id);

        match self.refresh {
            Refresh::Offline => self.guard_package(package_id, true, |dir| {
                if is_package_valid(dir) {
                    Ok(dir.to_path_buf())
                } else {
                    Err(Error::new(
                        ErrorKind::NotFound,
                        format!("Package '{package_id}' not found in offline mode."),
                    ))
                }
            }),

            Refresh::Force | Refresh::Default => {
                if self.refresh == Refresh::Default {
                    let exists =
                        self.guard_package(package_id, true, |dir| Ok(is_package_valid(dir)))?;
                    if exists {
                        return Ok(target_dir);
                    }
                }

                let staging_dir = self
                    .store
                    .root
                    .join(format!(".staging_{}", random_suffix()));
                create_dir_all(&staging_dir)?;

                let commit_result = self.guard_package(package_id, false, |_dir| {
                    if self.refresh == Refresh::Default && is_package_valid(&target_dir) {
                        best_effort_remove(&staging_dir);
                        return Ok(target_dir.clone());
                    }

                    if let Err(e) = fetch_fallback(&staging_dir) {
                        best_effort_remove(&staging_dir);
                        return Err(e);
                    }

                    self.store.commit_package(package_id, &staging_dir)?;
                    Ok(target_dir.clone())
                });

                if commit_result.is_err() && staging_dir.exists() {
                    best_effort_remove(&staging_dir);
                }

                commit_result
            }
        }
    }
}

#[derive(Debug)]
pub struct ContentAddressableStore {
    root: PathBuf,
    durability: Durability,
    link_strategy: OnceLock<LinkStrategy>,
}

impl ContentAddressableStore {
    #[inline]
    pub fn get_package_path(&self, package_id: &str) -> PathBuf {
        self.root.join(package_key(package_id))
    }

    fn get_strategy(&self) -> LinkStrategy {
        *self.link_strategy.get_or_init(|| {
            let probe_src = self.root.join(format!(".probe_src_{}", random_suffix()));
            let probe_dst = self.root.join(format!(".probe_dst_{}", random_suffix()));

            if std::fs::write(&probe_src, b"ok").is_err() {
                return LinkStrategy::Copy;
            }

            let strat = if reflink_copy::reflink(&probe_src, &probe_dst).is_ok() {
                LinkStrategy::Reflink
            } else if hard_link(&probe_src, &probe_dst).is_ok() {
                LinkStrategy::Hardlink
            } else {
                LinkStrategy::Copy
            };

            let _ = std::fs::remove_file(&probe_dst);
            let _ = std::fs::remove_file(&probe_src);

            strat
        })
    }

    pub fn commit_package(&self, package_id: &str, tmp_extracted_dir: &Path) -> Result<()> {
        let final_dest = self.get_package_path(package_id);

        if is_package_valid(&final_dest) {
            best_effort_remove(tmp_extracted_dir);
            return Ok(());
        }

        // TODO: Ask the user explicitly if they want to make the package readonly, in settings or in the initial setup
        // Self::make_readonly_parallel(&final_dest);
        write_completion_marker(tmp_extracted_dir)?;

        if self.durability == Durability::Flush {
            let _ = sync_directory(tmp_extracted_dir);
        }

        if let Err(rename_err) = rename(tmp_extracted_dir, &final_dest) {
            // lost race
            if is_package_valid(&final_dest) {
                best_effort_remove(tmp_extracted_dir);
                return Ok(());
            }

            // if target is corrupt but exists
            let mut old_trash = None;
            if final_dest.exists() {
                let trash_dest = self.root.join(format!(".trash_{}", random_suffix()));
                if rename(&final_dest, &trash_dest).is_ok() {
                    old_trash = Some(trash_dest);
                } else {
                    best_effort_remove(&final_dest);
                }
            }

            // cross-device staging fallback if req
            if is_cross_device_error(&rename_err) {
                let staging_dest = self.root.join(format!(".tmp_{}", random_suffix()));
                if let Err(copy_err) = self.copy_dir_all(tmp_extracted_dir, &staging_dest) {
                    best_effort_remove(&staging_dest);
                    best_effort_remove(tmp_extracted_dir);
                    if let Some(t) = old_trash {
                        best_effort_remove(&t);
                    }
                    return Err(copy_err);
                }

                if let Err(second_rename_err) = rename(&staging_dest, &final_dest) {
                    best_effort_remove(&staging_dest);
                    best_effort_remove(tmp_extracted_dir);
                    if let Some(t) = old_trash {
                        best_effort_remove(&t);
                    }
                    return Err(second_rename_err);
                }
            } else if let Err(retry_rename_err) = rename(tmp_extracted_dir, &final_dest) {
                // retry fast path if not cross-device err
                best_effort_remove(tmp_extracted_dir);
                if let Some(t) = old_trash {
                    best_effort_remove(&t);
                }
                return Err(retry_rename_err);
            }

            if let Some(trash) = old_trash {
                best_effort_remove(&trash);
            }
        }

        if self.durability == Durability::Flush {
            let _ = sync_directory(&final_dest);
            let _ = sync_parent_dir(&final_dest);
        }

        best_effort_remove(tmp_extracted_dir);
        Ok(())
    }

    fn copy_dir_all(&self, src: &Path, dst: &Path) -> Result<()> {
        let (dirs, files, symlinks) = WalkDir::new(src)
            .skip_hidden(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let src_path = entry.path();
                let rel = src_path.strip_prefix(src).unwrap();
                if rel.as_os_str().is_empty() {
                    return None;
                }

                let dst_path = dst.join(rel);
                Some((src_path, dst_path, entry.file_type(), entry.depth()))
            })
            .fold(
                (Vec::new(), Vec::new(), Vec::new()),
                |mut acc, (src_p, dst_p, ft, depth)| {
                    if ft.is_dir() {
                        acc.0.push(dst_p);
                    } else if ft.is_symlink() {
                        acc.2.push((src_p, dst_p, depth.saturating_sub(1) as i32));
                    } else {
                        acc.1.push((src_p, dst_p));
                    }
                    acc
                },
            );

        dirs.into_iter()
            .try_for_each(|dst_p| create_dir_all(dst_p))?;

        let strategy = self.get_strategy();

        files.into_par_iter().try_for_each(|(src_p, dst_p)| {
            match strategy {
                LinkStrategy::Reflink => {
                    if reflink_copy::reflink(&src_p, &dst_p).is_err() {
                        copy(&src_p, &dst_p)?;
                    }
                }

                LinkStrategy::Hardlink => {
                    if hard_link(&src_p, &dst_p).is_err() {
                        copy(&src_p, &dst_p)?;
                    }
                }

                LinkStrategy::Copy => {
                    copy(&src_p, &dst_p)?;
                }
            }
            Ok::<(), Error>(())
        })?;

        symlinks
            .into_par_iter()
            .try_for_each(|(src_p, dst_p, depth)| Self::link_symlink(&src_p, &dst_p, depth))?;

        Ok(())
    }

    #[inline]
    fn link_symlink(src_path: &Path, dst_path: &Path, starting_depth: i32) -> Result<()> {
        let target = std::fs::read_link(src_path)?;
        validate_symlink_target(starting_depth, &target)?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, dst_path)?;

        #[cfg(windows)]
        {
            if src_path.is_dir() {
                std::os::windows::fs::symlink_dir(&target, dst_path)?;
            } else {
                std::os::windows::fs::symlink_file(&target, dst_path)?;
            }
        }
        Ok(())
    }
}

#[inline]
fn is_package_valid(dir: &Path) -> bool {
    dir.join(".complete").exists()
}

#[inline]
fn write_completion_marker(dir: &Path) -> Result<()> {
    std::fs::write(dir.join(".complete"), b"ok")
}

fn validate_package_id(package_id: &str) -> Result<()> {
    if package_id.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Package ID cant be empty",
        ));
    }

    package_id.chars().try_for_each(|c| match c {
        '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid package ID: '{package_id}'"),
        )),
        _ => Ok(()),
    })
}

#[inline]
fn package_key(package_id: &str) -> String {
    blake3::hash(package_id.as_bytes()).to_hex().to_string()
}

#[inline]
fn random_suffix() -> String {
    format!("{:016x}", rand::random::<u64>())
}

fn validate_symlink_target(starting_depth: i32, target: &Path) -> Result<()> {
    target
        .components()
        .try_fold(starting_depth, |depth, comp| match comp {
            Component::Normal(_) => Ok(depth + 1),
            Component::ParentDir => {
                if depth == 0 {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        "Path traversal symlink forbidden",
                    ))
                } else {
                    Ok(depth - 1)
                }
            }
            Component::CurDir => Ok(depth),
            Component::RootDir | Component::Prefix(_) => Err(Error::new(
                ErrorKind::InvalidData,
                "Absolute symlinks forbidden",
            )),
        })
        .map(|_| ())
}

#[inline]
fn best_effort_remove(path: &Path) {
    if !path.exists() {
        return;
    }

    let file_name = path.file_name().unwrap_or_default();
    let trash_name = format!(".trash_{}_{}", file_name.to_string_lossy(), random_suffix());
    let trash_path = path.with_file_name(trash_name);

    let target_path = match rename(path, &trash_path) {
        Ok(_) => trash_path,
        Err(_) => path.to_path_buf(),
    };

    #[cfg(windows)]
    {
        let files: Vec<_> = WalkDir::new(&target_path)
            .skip_hidden(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path())
            .collect();

        files.into_par_iter().for_each(|p| {
            if let Ok(mut perms) = std::fs::metadata(&p).map(|m| m.permissions()) {
                if perms.readonly() {
                    perms.set_readonly(false);
                    let _ = std::fs::set_permissions(&p, perms);
                }
            }
        });
    }

    let _ = std::fs::remove_dir_all(&target_path);
}

fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        let _ = path;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        if let Ok(dir) = std::fs::File::open(path) {
            if let Err(e) = dir.sync_all() {
                if e.raw_os_error() != Some(22) {
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

fn sync_parent_dir(path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        let _ = path;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        if let Some(parent) = path.parent() {
            if let Ok(dir) = std::fs::File::open(parent) {
                if let Err(e) = dir.sync_all() {
                    if e.raw_os_error() != Some(22) {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
}

#[inline]
fn is_cross_device_error(err: &std::io::Error) -> bool {
    if err.kind() == ErrorKind::CrossesDevices {
        return true;
    }
    #[cfg(unix)]
    {
        err.raw_os_error() == Some(18)
    }
    #[cfg(windows)]
    {
        err.raw_os_error() == Some(17)
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

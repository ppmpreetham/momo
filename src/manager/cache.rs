#[derive(Eq, PartialEq)]
pub enum Refresh {
    /// Use cache if valid, fetch if missing.
    Default,
    /// `--force`: Disregard local cache manifests and force redownloading.
    Force,
    /// `--offline`: Strict offline mode. If a package is missing, throw an error immediately.
    Offline,
}

#[derive(Debug, Clone)]
pub struct Cache {
    /// The cache directory.
    root: PathBuf,
    /// The refresh strategy to use when reading from the cache.
    refresh: Refresh,
    /// A temporary cache directory, if the user requested `--no-cache`.
    ///
    /// Included to ensure that the temporary directory exists for the length of the operation, but
    /// is dropped at the end as appropriate.
    temp_dir: Option<Arc<tempfile::TempDir>>,
    /// Ensure that `momo cache` operations don't remove items from the cache that are used by another
    /// momo process.
    lock_file: Option<Arc<LockedFile>>,
    store: Arc<ContentAddressableStore>,
}
// TODO: Make CAS
// struct ContentAddressableStore{
// }

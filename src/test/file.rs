use jwalk::{Parallelism::RayonDefaultPool, WalkDirGeneric};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::byte_slices;

// standard test notations users use
const TEST_EXTENSIONS: &[&[u8]] = byte_slices![
    ".test.js",
    ".test.ts",
    ".test.jsx",
    ".test.tsx",
    ".spec.js",
    ".spec.ts",
    ".spec.jsx",
    ".spec.tsx",
    "_test.js",
    "_test.ts",
    "_test.jsx",
    "_test.tsx",
];

const IGNORED_DIRS: &[&[u8]] = byte_slices!["node_modules", ".git", "dist", "coverage", "target"];

// find test files in proj
// momo test login -> src/utils/login.test.ts
// momo test controllers -> src/controllers/user.test.ts
pub fn find_test_files(root_path: impl AsRef<Path>, cli_filter: Option<&str>) -> Vec<PathBuf> {
    WalkDirGeneric::<((), ())>::new(root_path)
        .skip_hidden(true)
        .parallelism(RayonDefaultPool {
            busy_timeout: Duration::from_secs(1),
        })
        .process_read_dir(|_, _, _, children| {
            children.retain(|res| {
                res.as_ref().map_or(true, |e| {
                    !IGNORED_DIRS.contains(&e.file_name.as_encoded_bytes())
                })
            });
        })
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type.is_file())
        .filter(|entry| {
            let file_name_bytes = entry.file_name.as_os_str().as_encoded_bytes();
            TEST_EXTENSIONS
                .iter()
                .any(|ext| file_name_bytes.ends_with(ext))
        })
        .filter_map(|entry| {
            let path = entry.path();

            if let Some(filter) = cli_filter {
                if !path.to_string_lossy().contains(filter) {
                    return None;
                }
            }

            Some(path)
        })
        .collect()
}

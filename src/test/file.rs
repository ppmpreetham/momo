use jwalk::{Parallelism::RayonDefaultPool, WalkDirGeneric};
use std::ffi::OsStr;
use std::path::Path;

const TEST_EXTENSIONS: &[&[u8]] = &[
    b".test.js",
    b".test.ts",
    b".test.jsx",
    b".test.tsx",
    b".spec.js",
    b".spec.ts",
    b".spec.jsx",
    b".spec.tsx",
    b"_test.js",
    b"_test.ts",
    b"_test.jsx",
    b"_test.tsx",
];

// find test files in proj
// momo test login -> src/utils/login.test.ts
// momo test controllers -> src/controllers/user.test.ts
pub fn find_test_files(root_path: impl AsRef<Path>, cli_filter: Option<&str>) {
    WalkDirGeneric::<()>::new(root_path)
        .skip_hidden(true)
        .parallelism(RayonDefaultPool)
        .process_read_dir(|_, _, _, children| {
            children.retain(|res| {
                res.as_ref()
                    .map_or(true, |e| e.file_name != OsStr::new("node_modules"))
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
        .for_each(|path| {
            println!("Test file: {:?}", path);
        });
}

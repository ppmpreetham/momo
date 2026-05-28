mod file;
mod parse;
mod reporter;
mod types;

use file::find_test_files;
use parse::parse_all_files;

use owo_colors::OwoColorize;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::test::types::{TestFailure, TestReport};

pub fn test(cli_test_filter: Option<&str>) {
    let files = find_test_files(".", cli_test_filter);
    let total_files = files.len();

    println!(
        "{}\n",
        format!("Found {total_files} files. Testing them...",).bright_black()
    );

    let failed_count = AtomicUsize::new(0);
    let errors = Mutex::new(Vec::new());

    parse_all_files(
        files,
        |_path, _ast| {},
        |path, err| {
            failed_count.fetch_add(1, Ordering::Relaxed);
            errors.lock().push(TestFailure {
                path: path.to_path_buf(),
                error: err.to_string(),
            });
        },
    );
    // TODO: do dynamic analysis (using JSC) for files that are statically analyzable.

    let total_failed = failed_count.load(Ordering::Relaxed);
    let total_passed = total_files - total_failed;

    let report = TestReport {
        total_files,
        total_failed,
        total_passed,
        failures: errors.into_inner(),
    };

    reporter::ConsoleReporter::print_summary(&report);
}

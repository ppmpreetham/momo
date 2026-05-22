use std::path::PathBuf;

pub struct TestFailure {
    pub path: PathBuf,
    pub error: String,
}

pub struct TestReport {
    pub total_files: usize,
    pub total_failed: usize,
    pub total_passed: usize,
    pub failures: Vec<TestFailure>,
}

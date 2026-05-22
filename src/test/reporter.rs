use crate::test::types::{TestFailure, TestReport};
use owo_colors::OwoColorize;

pub struct ConsoleReporter;

impl ConsoleReporter {
    pub fn print_summary(report: &TestReport) {
        if !report.failures.is_empty() {
            println!("{}", "FAILURES".bold().red());

            for failure in report.failures.iter().take(5) {
                Self::print_failure(failure);
            }

            if report.failures.len() > 5 {
                let hidden = report.failures.len() - 5;
                println!(
                    "{}",
                    format!("  ... and {hidden} more errors.").bright_black()
                );
            }
            println!();
        }

        println!("{}", "Test Suites:".bold().white());

        let failed_str = match report.total_failed {
            0 => "0 failed".to_string(),
            n => format!("{n} failed").bold().red().to_string(),
        };

        let passed_str = format!("{} passed", report.total_passed);

        println!(
            "  {failed_str}, {}, {} total",
            passed_str.bold().green(),
            report.total_files
        );
        println!("{}", "Ran all test suites.\n".bright_black());
    }
    fn print_failure(failure: &TestFailure) {
        let path_display = match failure.path.parent() {
            Some(p) if !p.as_os_str().is_empty() => failure
                .path
                .display()
                .to_string()
                .bright_black()
                .to_string(),
            _ => failure
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .bold()
                .to_string(),
        };
        println!(
            "{:<2}{} {}",
            "",
            " FAIL ".white().on_red().bold(),
            path_display
        );

        println!("> {}", failure.error.red());
    }
}

use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum VersionFormat {
    /// For Humans
    Text,
    /// For Machines
    Json,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalArgs {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    /// Bundle JS and TS files
    #[command(alias = "b")]
    Build {
        /// Watch files for changes and automatically re-bundle
        #[arg(short, long)]
        watch: bool,

        /// Enable prod minification
        #[arg(short, long)]
        minify: bool,

        /// Entrypoint file paths to begin bundling from
        #[arg(value_name = "ENTRY")]
        entry: Vec<String>,

        /// Directory to write all bundled output files
        #[arg(long, value_name = "DIR", default_value = "dist")]
        outdir: Option<String>,

        /// Single file name to output (only valid with one entrypoint)
        #[arg(long, value_name = "FILE")]
        outfile: Option<String>,

        /// Specify execution environment target architecture (browser, node)
        #[arg(long, default_value = "browser")]
        target: String,

        /// Specify output module resolution standard (esm, cjs)
        #[arg(long, default_value = "esm")]
        format: String,

        /// Exclude specific package identifiers or file modules from compilation
        #[arg(short = 'e', long = "external")]
        external: Vec<String>,
    },

    /// Run benchmarks
    #[command(alias = "be")]
    Bench,

    /// Initialize a new project
    #[command(alias = "create")]
    Init {
        /// Project directory name to instantiate into
        #[arg(value_name = "PATH")]
        path: Option<String>,

        /// auto accept all default values without interactive prompts
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// force initialization n overwrite an existing package.json || package.toml
        #[arg(long)]
        force: bool,

        /// Instantiates an executable runtime project profile (default architecture)
        #[arg(long, conflicts_with = "lib")]
        bin: bool,

        /// Instantiates a pure shareable dependency library layout structure
        #[arg(long, conflicts_with = "bin")]
        lib: bool,

        /// Overrides explicit application registration names (defaults to current dir name)
        #[arg(long, value_name = "NAME")]
        name: Option<String>,

        /// Selects default Version Control Engine configurations (git, hg, none)
        #[arg(long, value_name = "VCS", default_value = "git")]
        vcs: String,

        /// pre-configure a frontend framework ecosystem starter template.
        #[arg(long, value_name = "TEMPLATE")]
        template: Option<String>,
    },

    /// Install dependencies
    #[command(alias = "i")]
    Install,

    /// Run a JS/TS file
    #[command(alias = "r")]
    Run {
        /// The target file path or package script identifier (run/start) to execute
        #[arg(value_name = "TARGET", required = true)]
        target: String,

        /// Additional trailing arguments forwarded directly to the executing script/file
        #[arg(value_name = "ARGS", trailing_var_arg = true)]
        args: Vec<String>,

        /// Watch source files for changes and reload the execution loop instantly
        #[arg(short, long)]
        watch: bool,

        /// Automatically open a debugger inspector listening port (defaults to 127.0.0.1:4040) cuz MOMO is similar to 4040
        #[arg(long, value_name = "PORT")]
        inspect: Option<Option<u16>>,

        /// Evaluate raw script string code passed in directly from the terminal prompt
        #[arg(short = 'e', long = "eval", conflicts_with = "target")]
        eval: Option<String>,

        /// Force strict production or development environment configurations
        #[arg(long, default_value = "development")]
        env: String,
    },

    /// Test JS/TS files
    #[command(alias = "t")]
    Test {
        filter: Option<String>,

        /// run tests without cache
        #[arg(long)]
        no_cache: bool,

        /// show the first n tests
        #[arg(short, long)]
        number: Option<usize>,

        /// show all tests, failed and passed
        #[arg(short, long)]
        all: bool,

        /// Watch files for changes and re-run tests automatically
        #[arg(short, long)]
        watch: bool,

        /// Log tests in a file
        #[arg(short, long)]
        log: Option<String>,

        /// Don't Print code coverage statistics after completing the run
        #[arg(long, default_value = "false")]
        no_coverage: bool,

        /// Set maximum test runtime execution threshold in milliseconds
        #[arg(long, default_value_t = 30000)]
        timeout: u32,

        /// Force async tests to execute concurrently within files
        #[arg(long, default_value = "true")]
        concurrent: bool,

        /// Fail the entire test suite run immediately on the first error
        #[arg(long)]
        bail: bool,

        /// Maximum number of parallel worker threads/processes allowed to run tests concurrently
        #[arg(short, long, value_name = "JOBS")]
        jobs: Option<usize>,

        /// Force tests within a single file to execute sequentially instead of concurrently
        #[arg(long)]
        no_parallel: bool,

        /// Choose the format layout style for terminal test outputs
        #[arg(long, default_value = "line", value_name = "REPORTER")]
        reporter: String,

        /// Only run tests explicitly decorated with skipped or ignored metadata tags
        #[arg(long)]
        ignored: bool,

        /// Suppress passing test logs, only outputting failed stack traces
        #[arg(short, long)]
        quiet: bool,
    },

    /// Update all the globally installed packages
    #[command(alias = "upd")]
    Update,

    /// Upgrade Momo to the latest version    
    #[command(alias = "upg")]
    Upgrade,

    /// Display version information
    Version {
        #[arg(short, long, value_enum, default_value = "text")]
        format: VersionFormat,
    },
}

use rayon::prelude::*;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub type Allocator = oxc_allocator::Allocator;
pub type ParserReturn<'a> = oxc_parser::ParseReturn<'a>;
pub type SourceType = oxc_span::SourceType;
pub type Parser<'a> = oxc_parser::Parser<'a>;

const DEFAULT_BUFFER_CAPACITY: usize = 64 * 1024;
const MAX_BUFFER_CAPACITY: usize = 8 * 1024 * 1024;

pub struct Diagnostic {
    pub message: String,
}

pub struct WorkerContext {
    pub allocator: Allocator,
    pub source: Vec<u8>,
    pub diagnostics: Vec<Diagnostic>,
    pub temp_paths: Vec<PathBuf>,
}

impl WorkerContext {
    #[inline(always)]
    pub fn new() -> Self {
        let mut ctx = Self {
            allocator: Allocator::default(),
            source: Vec::with_capacity(DEFAULT_BUFFER_CAPACITY),
            diagnostics: Vec::with_capacity(32),
            temp_paths: Vec::with_capacity(32),
        };

        ctx.prepare_for_next_file();
        ctx
    }

    #[inline(always)]
    pub fn prepare_for_next_file(&mut self) {
        self.allocator.reset();

        if self.source.capacity() > MAX_BUFFER_CAPACITY {
            self.source = Vec::with_capacity(DEFAULT_BUFFER_CAPACITY);
        } else {
            self.source.clear();
        }

        self.diagnostics.clear();
        self.temp_paths.clear();
    }
}

#[inline(always)]
fn read_and_parse<'a>(path: &Path, ctx: &'a mut WorkerContext) -> Result<ParserReturn<'a>, String> {
    let mut fmt_err = |args: std::fmt::Arguments| -> String {
        ctx.err_buf.clear();
        let _ = std::fmt::write(&mut ctx.err_buf, args);
        ctx.err_buf.clone()
    };

    let mut file =
        File::open(path).map_err(|e| fmt_err(format_args!("Failed opening file: {e}")))?;

    file.read_to_end(&mut ctx.source)
        .map_err(|e| fmt_err(format_args!("Failed reading file: {e}")))?;

    let source = std::str::from_utf8(&ctx.source)
        .map_err(|e| fmt_err(format_args!("Invalid UTF-8: {e}")))?;

    let source_type =
        SourceType::from_path(path).map_err(|_| "Unknown source dialect".to_string())?;

    let parsed = Parser::new(&ctx.allocator, source, source_type).parse();

    if parsed.panicked {
        return Err("Parser panicked".into());
    }

    if !parsed.errors.is_empty() {
        return Err(fmt_err(format_args!(
            "{} syntax errors encountered",
            parsed.errors.len()
        )));
    }

    Ok(parsed)
}

pub fn parse_all_files<S, E>(paths: Vec<PathBuf>, on_success: S, on_error: E)
where
    S: for<'a> Fn(&Path, &ParserReturn<'a>) + Send + Sync,
    E: Fn(&Path, String) + Send + Sync,
{
    paths
        .into_par_iter()
        .for_each_init(WorkerContext::new, |ctx, path| {
            match read_and_parse(&path, ctx) {
                Ok(ast) => {
                    on_success(&path, &ast);
                }
                Err(err) => {
                    on_error(&path, err);
                }
            }

            ctx.prepare_for_next_file();
        });
}
